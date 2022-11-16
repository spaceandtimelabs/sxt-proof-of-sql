use crate::base::database::{CommitmentAccessor, DataAccessor};
use crate::sql::ast::{BoolExpr, TableExpr};
use crate::sql::proof::{
    make_sumcheck_term, ProofBuilder, ProofCounts, SumcheckSubpolynomial, VerificationBuilder,
};

use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use dyn_partial_eq::DynPartialEq;
use std::cmp::max;

/// Provable logical AND expression
#[derive(Debug, DynPartialEq, PartialEq)]
pub struct AndExpr {
    lhs: Box<dyn BoolExpr>,
    rhs: Box<dyn BoolExpr>,
}

impl AndExpr {
    /// Create logical AND expression
    pub fn new(lhs: Box<dyn BoolExpr>, rhs: Box<dyn BoolExpr>) -> Self {
        Self { lhs, rhs }
    }
}

impl BoolExpr for AndExpr {
    fn count(&self, counts: &mut ProofCounts) {
        self.lhs.count(counts);
        self.rhs.count(counts);

        counts.sumcheck_subpolynomials += 1;
        counts.intermediate_mles += 1;
        counts.sumcheck_max_multiplicands = max(counts.sumcheck_max_multiplicands, 3);
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        table: &TableExpr,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        let lhs = self
            .lhs
            .prover_evaluate(builder, alloc, table, counts, accessor);
        let rhs = self
            .rhs
            .prover_evaluate(builder, alloc, table, counts, accessor);
        let n = lhs.len();
        assert_eq!(n, rhs.len());

        // lhs_and_rhs
        let lhs_and_rhs = alloc.alloc_slice_fill_with(n, |i| lhs[i] && rhs[i]);
        builder.produce_intermediate_mle(lhs_and_rhs);

        // subpolynomial: lhs_and_rhs - lhs * rhs
        let terms = vec![
            (
                Scalar::one(),
                vec![make_sumcheck_term(counts.sumcheck_variables, lhs_and_rhs)],
            ),
            (
                -Scalar::one(),
                vec![
                    make_sumcheck_term(counts.sumcheck_variables, lhs),
                    make_sumcheck_term(counts.sumcheck_variables, rhs),
                ],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));

        // selection
        lhs_and_rhs
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        table: &TableExpr,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) -> Scalar {
        let lhs = self.lhs.verifier_evaluate(builder, table, counts, accessor);
        let rhs = self.rhs.verifier_evaluate(builder, table, counts, accessor);

        // lhs_and_rhs
        let lhs_and_rhs = builder.consume_intermediate_mle();

        // subpolynomial: lhs_and_rhs - lhs * rhs
        let eval = builder.mle_evaluations.random_evaluation * (lhs_and_rhs - lhs * rhs);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);

        // selection
        lhs_and_rhs
    }
}
