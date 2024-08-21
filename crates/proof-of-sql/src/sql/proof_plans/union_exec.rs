use super::{
    fold_columns, fold_vals, AliasedProvableExprPlan, ProvableExpr, ProvableExprPlan, TableExpr,
};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            filter_util::filter_columns, Column, ColumnField, ColumnRef, CommitmentAccessor,
            DataAccessor, MetadataAccessor, OwnedTable,
        },
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::proof::{
        CountBuilder, HonestProver, Indexes, ProofBuilder, ProofExpr, ProverEvaluate,
        ProverHonestyMarker, ResultBuilder, SumcheckSubpolynomialType, VerificationBuilder,
    },
};
use bumpalo::Bump;
use core::{iter::repeat_with, marker::PhantomData};
use indexmap::IndexSet;
use num_traits::{One, Zero};
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     <ProofExecutionPlan> UNION ALL <ProofExecutionPlan> ...
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UnionExec<C: Commitment> {
    pub(super) inputs: Vec<Box<dyn ProofExecutionPlan<C>>>,
    pub(super) schema: Vec<ColumnField>,
}

fn tables_compatible(tables: &[TableExpr]) -> bool {
    let first_table = &tables[0];
    tables.iter().all(|table| table == first_table)
}

impl<C: Commitment> UnionExec<C> {
    /// Creates a new union all expression.
    pub fn try_new(
        inputs: Vec<Box<dyn ProofExecutionPlan<C>>>,
        schema: Vec<ColumnField>,
    ) -> ConversionResult<Self> {
        //Check schema compatibility
        inputs
            .iter()
            .all(|input| input.get_column_result_fields() == schema)
            .then(|| Self { inputs, schema })
            .ok_or(ConversionError::SchemaMismatch(
                "Union schema mismatch".to_string(),
            ))
    }
}

impl<C: Commitment> ProofExpr<C> for UnionExec<C>
where
    UnionExec<C>: ProverEvaluate<C::Scalar>,
{
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        let num_inputs = self.inputs.len();
        self.inputs
            .iter()
            .try_for_each(|input| input.count(builder))?;
        builder.count_intermediate_mles(num_inputs + 1);
        builder.count_subpolynomials(num_inputs + 2);
        builder.count_degree(3);
        builder.count_post_result_challenges(2);
        Ok(())
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        self.inputs
            .iter()
            .map(|input| input.get_length(accessor))
            .sum()
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        // TODO: Shall unions always start from 0?
        0
    }

    #[allow(unused_variables)]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<(), ProofError> {
        // 1. selection
        let selection_eval = self.where_clause.verifier_evaluate(builder, accessor)?;
        // 2. columns
        let columns_evals = Vec::from_iter(
            self.aliased_results
                .iter()
                .map(|aliased_expr| aliased_expr.expr.verifier_evaluate(builder, accessor))
                .collect::<Result<Vec<_>, _>>()?,
        );
        // 3. indexes
        let indexes_eval = builder
            .mle_evaluations
            .result_indexes_evaluation
            .ok_or(ProofError::VerificationError("invalid indexes"))?;
        // 4. filtered_columns
        let filtered_columns_evals = Vec::from_iter(
            repeat_with(|| builder.consume_result_mle()).take(self.aliased_results.len()),
        );

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        verify_filter(
            builder,
            alpha,
            beta,
            columns_evals,
            selection_eval,
            filtered_columns_evals,
        )
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.schema.clone()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.inputs
            .iter()
            .flat_map(|input| input.get_column_references())
            .collect()
    }
}

impl<C: Commitment> ProverEvaluate<C::Scalar> for UnionExec<C> {
    #[tracing::instrument(name = "UnionExec::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) {
        // 1. columns
        let columns = self.inputs.iter().map(|input| {
            let mut input_builder = builder.create_sub_builder();
            input.result_evaluate(&mut input_builder, alloc, accessor);
            input_builder
        });
        // 2. columns
        let columns = Vec::from_iter(self.aliased_results.iter().map(|aliased_expr| {
            aliased_expr
                .expr
                .result_evaluate(builder.table_length(), alloc, accessor)
        }));
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);
        // 3. set indexes
        builder.set_result_indexes(Indexes::Dense(0..(self.get_length() as u64)));
        // 4. set filtered_columns
        for col in filtered_columns {
            builder.produce_result_column(col);
        }
        builder.request_post_result_challenges(2);
    }

    #[tracing::instrument(name = "UnionExec::prover_evaluate", level = "debug", skip_all)]
    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) {
        // 1. selection
        let selection_column: Column<'a, C::Scalar> =
            self.where_clause.prover_evaluate(builder, alloc, accessor);
        let selection = selection_column
            .as_boolean()
            .expect("selection is not boolean");

        // 2. columns
        let columns = Vec::from_iter(
            self.aliased_results
                .iter()
                .map(|aliased_expr| aliased_expr.expr.prover_evaluate(builder, alloc, accessor)),
        );
        // Compute filtered_columns and indexes
        let (filtered_columns, result_len) = filter_columns(alloc, &columns, selection);

        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();

        prove_filter::<C::Scalar>(
            builder,
            alloc,
            alpha,
            beta,
            &columns,
            selection,
            &filtered_columns,
            result_len,
        );
    }
}

fn verify_filter<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    alpha: C::Scalar,
    beta: C::Scalar,
    c_evals: Vec<C::Scalar>,
    s_eval: C::Scalar,
    d_evals: Vec<C::Scalar>,
) -> Result<(), ProofError> {
    let one_eval = builder.mle_evaluations.one_evaluation;
    let rand_eval = builder.mle_evaluations.random_evaluation;

    let chi_eval = match builder.mle_evaluations.result_indexes_evaluation {
        Some(eval) => eval,
        None => return Err(ProofError::VerificationError("Result indexes not valid.")),
    };

    let c_fold_eval = alpha * one_eval + fold_vals(beta, &c_evals);
    let d_bar_fold_eval = alpha * one_eval + fold_vals(beta, &d_evals);
    let c_star_eval = builder.consume_intermediate_mle();
    let d_star_eval = builder.consume_intermediate_mle();

    // sum c_star * s - d_star = 0
    builder.produce_sumcheck_subpolynomial_evaluation(&(c_star_eval * s_eval - d_star_eval));

    // c_fold * c_star - 1 = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        &(rand_eval * (c_fold_eval * c_star_eval - one_eval)),
    );

    // d_bar_fold * d_star - chi = 0
    builder.produce_sumcheck_subpolynomial_evaluation(
        &(rand_eval * (d_bar_fold_eval * d_star_eval - chi_eval)),
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn prove_filter<'a, S: Scalar + 'a>(
    builder: &mut ProofBuilder<'a, S>,
    alloc: &'a Bump,
    alpha: S,
    beta: S,
    c: &[Column<S>],
    s: &'a [bool],
    d: &[Column<S>],
    m: usize,
) {
    let n = builder.table_length();
    let chi = alloc.alloc_slice_fill_copy(n, false);
    chi[..m].fill(true);

    let c_fold = alloc.alloc_slice_fill_copy(n, alpha);
    fold_columns(c_fold, One::one(), beta, c);
    let d_bar_fold = alloc.alloc_slice_fill_copy(n, alpha);
    fold_columns(d_bar_fold, One::one(), beta, d);

    let c_star = alloc.alloc_slice_copy(c_fold);
    let d_star = alloc.alloc_slice_copy(d_bar_fold);
    d_star[m..].fill(Zero::zero());
    slice_ops::batch_inversion(c_star);
    slice_ops::batch_inversion(&mut d_star[..m]);

    builder.produce_intermediate_mle(c_star as &[_]);
    builder.produce_intermediate_mle(d_star as &[_]);

    // sum c_star * s - d_star = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::ZeroSum,
        vec![
            (S::one(), vec![Box::new(c_star as &[_]), Box::new(s)]),
            (-S::one(), vec![Box::new(d_star as &[_])]),
        ],
    );

    // c_fold * c_star - 1 = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![Box::new(c_star as &[_]), Box::new(c_fold as &[_])],
            ),
            (-S::one(), vec![]),
        ],
    );

    // d_bar_fold * d_star - chi = 0
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (
                S::one(),
                vec![Box::new(d_star as &[_]), Box::new(d_bar_fold as &[_])],
            ),
            (-S::one(), vec![Box::new(chi as &[_])]),
        ],
    );
}
