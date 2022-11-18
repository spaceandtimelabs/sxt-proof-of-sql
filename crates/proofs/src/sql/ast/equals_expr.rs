use super::{BoolExpr, ColumnRef};

use crate::base::database::{Column, CommitmentAccessor, DataAccessor};
use crate::base::scalar::IntoScalar;
use crate::sql::proof::{
    make_sumcheck_term, ProofBuilder, ProofCounts, SumcheckSubpolynomial, VerificationBuilder,
};

use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use dyn_partial_eq::DynPartialEq;
use pedersen::compute::get_one_commit;
use std::cmp::max;

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
}

impl BoolExpr for EqualsExpr {
    fn count(&self, counts: &mut ProofCounts) {
        counts.sumcheck_subpolynomials += 2;
        counts.anchored_mles += 1;
        counts.intermediate_mles += 2;
        counts.sumcheck_max_multiplicands = max(counts.sumcheck_max_multiplicands, 3);
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        let Column::BigInt(col) =
            accessor.get_column(&self.column_ref.table_name, &self.column_ref.column_name);

        // lhs
        let lhs =
            alloc.alloc_slice_fill_with(counts.table_length, |i| col[i].into_scalar() - self.value);
        builder.produce_anchored_mle(lhs);

        // lhs_pseudo_inv
        // Note: We can do this more efficiently with bulk inversion; but we're keeping things
        // simple to start with
        let lhs_pseudo_inv = alloc.alloc_slice_fill_with(counts.table_length, |i| {
            if lhs[i] != Scalar::zero() {
                lhs[i].invert()
            } else {
                Scalar::zero()
            }
        });
        builder.produce_intermediate_mle(lhs_pseudo_inv);

        // selection_not
        let selection_not =
            alloc.alloc_slice_fill_with(counts.table_length, |i| lhs[i] != Scalar::zero());
        builder.produce_intermediate_mle(selection_not);

        // selection
        let selection = alloc.alloc_slice_fill_with(counts.table_length, |i| !selection_not[i]);

        // subpolynomial: selection * lhs
        let terms = vec![(
            Scalar::one(),
            vec![
                make_sumcheck_term(counts.sumcheck_variables, lhs),
                make_sumcheck_term(counts.sumcheck_variables, selection),
            ],
        )];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));

        // subpolynomial: selection_not - lhs * lhs_pseudo_inv
        let terms = vec![
            (
                Scalar::one(),
                vec![make_sumcheck_term(counts.sumcheck_variables, selection_not)],
            ),
            (
                -Scalar::one(),
                vec![
                    make_sumcheck_term(counts.sumcheck_variables, lhs),
                    make_sumcheck_term(counts.sumcheck_variables, lhs_pseudo_inv),
                ],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));

        selection
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) -> Scalar {
        // lhs_commit
        let lhs_commit = accessor
            .get_commitment(&self.column_ref.table_name, &self.column_ref.column_name)
            - self.value * get_one_commit(counts.table_length as u64);

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
}
