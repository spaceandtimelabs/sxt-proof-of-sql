use crate::base::database::{Column, ColumnRef, CommitmentAccessor, DataAccessor};
use crate::base::scalar::ToScalar;
use crate::sql::ast::BoolExpr;
use crate::sql::proof::{
    MultilinearExtensionImpl, ProofBuilder, ProofCounts, SumcheckSubpolynomial, VerificationBuilder,
};

use crate::base::scalar::batch_pseudo_invert;
use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use dyn_partial_eq::DynPartialEq;
use proofs_gpu::compute::get_one_commit;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::cmp::max;
use std::collections::HashSet;

/// Provable AST expression for an equals expression
///
/// Note: we are currently limited only to expressions of the form
/// ```ignore
///     <col> = <constant>
/// ```
#[derive(Debug, DynPartialEq, PartialEq, Eq)]
pub struct EqualsExpr {
    value: Scalar,
    column_ref: ColumnRef,
}

impl EqualsExpr {
    /// Create a new equals expression
    pub fn new(column_ref: ColumnRef, value: Scalar) -> Self {
        Self { value, column_ref }
    }

    fn prover_evaluate_impl<'a, T: ToScalar + Sync>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        col: &'a [T],
    ) -> &'a [bool] {
        // lhs
        let lhs = alloc.alloc_slice_fill_default(counts.table_length);
        lhs.par_iter_mut()
            .zip(col)
            .for_each(|(a, b)| *a = b.to_scalar() - self.value);
        builder.produce_anchored_mle(lhs);

        // lhs_pseudo_inv
        let lhs_pseudo_inv = alloc.alloc_slice_fill_default::<Scalar>(counts.table_length);
        batch_pseudo_invert(lhs_pseudo_inv, lhs);

        builder.produce_intermediate_mle(lhs_pseudo_inv);

        // selection_not
        let selection_not =
            alloc.alloc_slice_fill_with(counts.table_length, |i| lhs[i] != Scalar::zero());
        builder.produce_intermediate_mle(selection_not);

        // selection
        let selection = alloc.alloc_slice_fill_with(counts.table_length, |i| !selection_not[i]);

        // subpolynomial: selection * lhs
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![(
            Scalar::one(),
            vec![
                Box::new(MultilinearExtensionImpl::new(lhs)),
                Box::new(MultilinearExtensionImpl::new(selection)),
            ],
        )]));

        // subpolynomial: selection_not - lhs * lhs_pseudo_inv
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(selection_not))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(lhs)),
                    Box::new(MultilinearExtensionImpl::new(lhs_pseudo_inv)),
                ],
            ),
        ]));

        selection
    }
}

impl BoolExpr for EqualsExpr {
    fn count(&self, counts: &mut ProofCounts) {
        counts.sumcheck_subpolynomials += 2;
        counts.anchored_mles += 1;
        counts.intermediate_mles += 2;
        counts.sumcheck_max_multiplicands = max(counts.sumcheck_max_multiplicands, 3);
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
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        match accessor.get_column(self.column_ref) {
            Column::BigInt(col) => self.prover_evaluate_impl(builder, alloc, counts, col),
            Column::HashedBytes((_, scals)) => {
                self.prover_evaluate_impl(builder, alloc, counts, scals)
            }
        }
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) -> Scalar {
        let one_commit = get_one_commit((counts.table_length + counts.offset_generators) as u64)
            - get_one_commit(counts.offset_generators as u64);

        // lhs_commit
        let lhs_commit = accessor.get_commitment(self.column_ref) - self.value * one_commit;

        // consume mle evaluations
        let lhs_eval = builder.consume_anchored_mle(&lhs_commit);
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

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        columns.insert(self.column_ref);
    }
}
