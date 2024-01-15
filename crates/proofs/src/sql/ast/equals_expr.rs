use crate::{
    base::{
        database::{Column, ColumnRef, CommitmentAccessor, DataAccessor},
        proof::ProofError,
        scalar::ArkScalar,
        slice_ops,
    },
    sql::{
        ast::BoolExpr,
        proof::{CountBuilder, ProofBuilder, SumcheckSubpolynomialType, VerificationBuilder},
    },
};
use blitzar::compute::get_one_commit;
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use dyn_partial_eq::DynPartialEq;
use num_traits::{One, Zero};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable AST expression for an equals expression
///
/// Note: we are currently limited only to expressions of the form
/// ```ignore
///     <col> = <constant>
/// ```
#[derive(Debug, DynPartialEq, PartialEq, Eq, Serialize, Deserialize)]
pub struct EqualsExpr {
    value: ArkScalar,
    column_ref: ColumnRef,
}

impl EqualsExpr {
    /// Create a new equals expression
    pub fn new(column_ref: ColumnRef, value: ArkScalar) -> Self {
        Self { value, column_ref }
    }

    fn result_evaluate_impl<'a, T: Sync>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        col: &'a [T],
    ) -> &'a [bool]
    where
        &'a T: Into<ArkScalar>,
        &'a ArkScalar: Into<ArkScalar>,
    {
        let lhs = alloc.alloc_slice_fill_default(table_length);
        lhs.par_iter_mut()
            .zip(col)
            .for_each(|(a, b)| *a = Into::<ArkScalar>::into(b) - self.value);
        result_evaluate_equals_zero(table_length, alloc, lhs)
    }

    fn prover_evaluate_impl<'a, T: Sync>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        col: &'a [T],
    ) -> &'a [bool]
    where
        &'a T: Into<ArkScalar>,
        &'a ArkScalar: Into<ArkScalar>,
    {
        let table_length = builder.table_length();

        // lhs
        let lhs = alloc.alloc_slice_fill_default(table_length);
        lhs.par_iter_mut()
            .zip(col)
            .for_each(|(a, b)| *a = Into::<ArkScalar>::into(b) - self.value);
        prover_evaluate_equals_zero(builder, alloc, lhs)
    }
}

#[typetag::serde]
impl BoolExpr for EqualsExpr {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        count_equals_zero(builder);
        Ok(())
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        match accessor.get_column(self.column_ref) {
            Column::BigInt(col) => self.result_evaluate_impl(table_length, alloc, col),
            Column::Int128(col) => self.result_evaluate_impl(table_length, alloc, col),
            Column::VarChar((_, scals)) => self.result_evaluate_impl(table_length, alloc, scals),
            #[cfg(test)]
            // While implementing this for a Scalar columns is very simple
            // major refactoring is required to create tests for this
            // (in particular the tests need to used the OwnedTableTestAccessor)
            Column::Scalar(_) => todo!("Scalar column type not supported in equals_expr"),
        }
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.equals_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        match accessor.get_column(self.column_ref) {
            Column::BigInt(col) => self.prover_evaluate_impl(builder, alloc, col),
            Column::Int128(col) => self.prover_evaluate_impl(builder, alloc, col),
            Column::VarChar((_, scals)) => self.prover_evaluate_impl(builder, alloc, scals),
            #[cfg(test)]
            // While implementing this for a Scalar columns is very simple
            // major refactoring is required to create tests for this
            // (in particular the tests need to used the OwnedTableTestAccessor)
            Column::Scalar(_) => todo!("Scalar column type not supported in equals_expr"),
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> Result<ArkScalar, ProofError> {
        let table_length = builder.table_length();
        let generator_offset = builder.generator_offset();
        let one_commit = get_one_commit((table_length + generator_offset) as u64)
            - get_one_commit(generator_offset as u64);

        // lhs_commit
        let lhs_commit = accessor.get_commitment(self.column_ref) - self.value * one_commit;

        Ok(verifier_evaluate_equals_zero(builder, &lhs_commit))
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        columns.insert(self.column_ref);
    }
}

pub fn result_evaluate_equals_zero<'a>(
    table_length: usize,
    alloc: &'a Bump,
    lhs: &'a [ArkScalar],
) -> &'a [bool] {
    assert_eq!(table_length, lhs.len());
    alloc.alloc_slice_fill_with(table_length, |i| lhs[i] == ArkScalar::zero())
}

pub fn prover_evaluate_equals_zero<'a>(
    builder: &mut ProofBuilder<'a>,
    alloc: &'a Bump,
    lhs: &'a [ArkScalar],
) -> &'a [bool] {
    let table_length = builder.table_length();

    // lhs
    builder.produce_anchored_mle(lhs);

    // lhs_pseudo_inv
    let lhs_pseudo_inv = alloc.alloc_slice_copy(lhs);
    slice_ops::batch_inversion(lhs_pseudo_inv);

    builder.produce_intermediate_mle(lhs_pseudo_inv as &[_]);

    // selection_not
    let selection_not: &[_] =
        alloc.alloc_slice_fill_with(table_length, |i| lhs[i] != ArkScalar::zero());
    builder.produce_intermediate_mle(selection_not);

    // selection
    let selection: &[_] = alloc.alloc_slice_fill_with(table_length, |i| !selection_not[i]);

    // subpolynomial: selection * lhs
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![(ArkScalar::one(), vec![Box::new(lhs), Box::new(selection)])],
    );

    // subpolynomial: selection_not - lhs * lhs_pseudo_inv
    builder.produce_sumcheck_subpolynomial(
        SumcheckSubpolynomialType::Identity,
        vec![
            (ArkScalar::one(), vec![Box::new(selection_not)]),
            (
                -ArkScalar::one(),
                vec![Box::new(lhs), Box::new(lhs_pseudo_inv as &[_])],
            ),
        ],
    );

    selection
}

pub fn verifier_evaluate_equals_zero(
    builder: &mut VerificationBuilder,
    lhs_commit: &RistrettoPoint,
) -> ArkScalar {
    // consume mle evaluations
    let lhs_eval = builder.consume_anchored_mle(lhs_commit);
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
    builder.count_anchored_mles(1);
    builder.count_intermediate_mles(2);
    builder.count_degree(3);
}
