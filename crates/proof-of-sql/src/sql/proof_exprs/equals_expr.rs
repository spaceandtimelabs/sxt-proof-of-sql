use super::{scale_and_add_subtract_eval, scale_and_subtract, DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
    utils::log,
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable AST expression for an equals expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EqualsExpr {
    pub(crate) lhs: Box<DynProofExpr>,
    pub(crate) rhs: Box<DynProofExpr>,
}

impl EqualsExpr {
    /// Create a new equals expression
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> Self {
        Self { lhs, rhs }
    }
}

impl ProofExpr for EqualsExpr {
    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    #[tracing::instrument(name = "EqualsExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        log::log_memory_usage("Start");

        let lhs_column = self.lhs.result_evaluate(alloc, table);
        let rhs_column = self.rhs.result_evaluate(alloc, table);
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let res = scale_and_subtract(alloc, lhs_column, rhs_column, lhs_scale, rhs_scale, true)
            .expect("Failed to scale and subtract");
        let res = Column::Boolean(result_evaluate_equals_zero(table.num_rows(), alloc, res));

        log::log_memory_usage("End");

        res
    }

    #[tracing::instrument(name = "EqualsExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> Column<'a, S> {
        log::log_memory_usage("Start");

        let lhs_column = self.lhs.prover_evaluate(builder, alloc, table);
        let rhs_column = self.rhs.prover_evaluate(builder, alloc, table);
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let scale_and_subtract_res =
            scale_and_subtract(alloc, lhs_column, rhs_column, lhs_scale, rhs_scale, true)
                .expect("Failed to scale and subtract");
        let res = Column::Boolean(prover_evaluate_equals_zero(
            table.num_rows(),
            builder,
            alloc,
            scale_and_subtract_res,
        ));

        log::log_memory_usage("End");

        res
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<S, ProofError> {
        let lhs_eval = self.lhs.verifier_evaluate(builder, accessor, chi_eval)?;
        let rhs_eval = self.rhs.verifier_evaluate(builder, accessor, chi_eval)?;
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let res = scale_and_add_subtract_eval(lhs_eval, rhs_eval, lhs_scale, rhs_scale, true);
        verifier_evaluate_equals_zero(builder, res, chi_eval)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

#[expect(
    clippy::missing_panics_doc,
    reason = "table_length is guaranteed to match lhs.len()"
)]
pub fn result_evaluate_equals_zero<'a, S: Scalar>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &'a [S],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] == S::zero())
}

pub fn prover_evaluate_equals_zero<'a, S: Scalar>(
    table_length: usize,
    builder: &mut FinalRoundBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: &'a [S],
) -> &'a [bool] {
    // lhs_pseudo_inv
    let lhs_pseudo_inv = alloc.alloc_slice_copy(lhs);
    slice_ops::batch_inversion(lhs_pseudo_inv);

    builder.produce_intermediate_mle(lhs_pseudo_inv as &[_]);

    // selection_not
    let selection_not: &[_] = alloc.alloc_slice_fill_with(table_length, |i| lhs[i] != S::zero());

    // selection
    let selection: &[_] = alloc.alloc_slice_fill_with(table_length, |i| !selection_not[i]);
    builder.produce_intermediate_mle(selection);

    // subpolynomial: selection * lhs
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![(S::one(), vec![Box::new(lhs), Box::new(selection)])],
    );

    // subpolynomial: selection_not - lhs * lhs_pseudo_inv
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (S::one(), vec![Box::new(selection_not)]),
            (
                -S::one(),
                vec![Box::new(lhs), Box::new(lhs_pseudo_inv as &[_])],
            ),
        ],
    );

    selection
}

pub fn verifier_evaluate_equals_zero<S: Scalar>(
    builder: &mut impl VerificationBuilder<S>,
    lhs_eval: S,
    chi_eval: S,
) -> Result<S, ProofError> {
    // consume mle evaluations
    let lhs_pseudo_inv_eval = builder.try_consume_final_round_mle_evaluation()?;
    let selection_eval = builder.try_consume_final_round_mle_evaluation()?;
    let selection_not_eval = chi_eval - selection_eval;

    // subpolynomial: selection * lhs
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        selection_eval * lhs_eval,
        2,
    )?;

    // subpolynomial: selection_not - lhs * lhs_pseudo_inv
    builder.try_produce_sumcheck_subpolynomial_evaluation(
        SumcheckSubpolynomialType::Identity,
        selection_not_eval - lhs_eval * lhs_pseudo_inv_eval,
        2,
    )?;

    Ok(selection_eval)
}
