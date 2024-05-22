use super::{scale_and_subtract, scale_and_subtract_eval, ProvableExpr, ProvableExprPlan};
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        proof::ProofError,
        scalar::Scalar,
        slice_ops,
    },
    sql::proof::{CountBuilder, ProofBuilder, SumcheckSubpolynomialType, VerificationBuilder},
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable AST expression for an equals expression
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct EqualsExpr<C: Commitment> {
    lhs: Box<ProvableExprPlan<C>>,
    rhs: Box<ProvableExprPlan<C>>,
}

impl<C: Commitment> EqualsExpr<C> {
    /// Create a new equals expression
    pub fn new(lhs: Box<ProvableExprPlan<C>>, rhs: Box<ProvableExprPlan<C>>) -> Self {
        Self { lhs, rhs }
    }
}

impl<C: Commitment> ProvableExpr<C> for EqualsExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        count_equals_zero(builder);
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column = self.lhs.result_evaluate(table_length, alloc, accessor);
        let rhs_column = self.rhs.result_evaluate(table_length, alloc, accessor);
        let res = scale_and_subtract(alloc, lhs_column, rhs_column)
            .expect("Failed to scale and subtract");
        Column::Boolean(result_evaluate_equals_zero(table_length, alloc, res))
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.equals_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column = self.lhs.prover_evaluate(builder, alloc, accessor);
        let rhs_column = self.rhs.prover_evaluate(builder, alloc, accessor);
        let res = scale_and_subtract(alloc, lhs_column, rhs_column)
            .expect("Failed to scale and subtract");
        Column::Boolean(prover_evaluate_equals_zero(builder, alloc, res))
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let lhs_eval = self.lhs.verifier_evaluate(builder, accessor)?;
        let rhs_eval = self.rhs.verifier_evaluate(builder, accessor)?;
        let lhs_scale = self.lhs.data_type().scale();
        let rhs_scale = self.rhs.data_type().scale();
        let res = scale_and_subtract_eval(lhs_eval, rhs_eval, lhs_scale, rhs_scale)
            .expect("Failed to scale and subtract");
        Ok(verifier_evaluate_equals_zero(builder, res))
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}

pub fn result_evaluate_equals_zero<'a, S: Scalar>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &'a [S],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] == S::zero())
}

pub fn prover_evaluate_equals_zero<'a, S: Scalar>(
    builder: &mut ProofBuilder<'a, S>,
    alloc: &'a Bump,
    lhs: &'a [S],
) -> &'a [bool] {
    let table_length = builder.table_length();

    // lhs_pseudo_inv
    let lhs_pseudo_inv = alloc.alloc_slice_copy(lhs);
    slice_ops::batch_inversion(lhs_pseudo_inv);

    builder.produce_intermediate_mle(lhs_pseudo_inv as &[_]);

    // selection_not
    let selection_not: &[_] = alloc.alloc_slice_fill_with(table_length, |i| lhs[i] != S::zero());
    builder.produce_intermediate_mle(selection_not);

    // selection
    let selection: &[_] = alloc.alloc_slice_fill_with(table_length, |i| !selection_not[i]);

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

pub fn verifier_evaluate_equals_zero<C: Commitment>(
    builder: &mut VerificationBuilder<C>,
    lhs_eval: C::Scalar,
) -> C::Scalar {
    // consume mle evaluations
    let lhs_pseudo_inv_eval = builder.consume_intermediate_mle();
    let selection_not_eval = builder.consume_intermediate_mle();
    let selection_eval = builder.mle_evaluations.one_evaluation - selection_not_eval;

    // subpolynomial: selection * lhs
    let eval = builder.mle_evaluations.random_evaluation * (selection_eval * lhs_eval);
    builder.produce_sumcheck_subpolynomial_evaluation(&eval);

    // subpolynomial: selection_not - lhs * lhs_pseudo_inv
    let eval = builder.mle_evaluations.random_evaluation
        * (selection_not_eval - lhs_eval * lhs_pseudo_inv_eval);
    builder.produce_sumcheck_subpolynomial_evaluation(&eval);

    selection_eval
}

pub fn count_equals_zero(builder: &mut CountBuilder) {
    builder.count_subpolynomials(2);
    builder.count_intermediate_mles(2);
    builder.count_degree(3);
}
