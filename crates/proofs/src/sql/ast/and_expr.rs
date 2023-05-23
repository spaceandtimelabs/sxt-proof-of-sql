use crate::base::database::{ColumnRef, CommitmentAccessor, DataAccessor};
use crate::base::polynomial::ArkScalar;
use crate::sql::ast::BoolExpr;
use crate::sql::proof::{
    MultilinearExtensionImpl, ProofBuilder, ProofCounts, SumcheckSubpolynomial, VerificationBuilder,
};

use crate::base::polynomial::Scalar;
use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use std::cmp::max;
use std::collections::HashSet;

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

    #[tracing::instrument(
        name = "proofs.sql.ast.and_expr.prover_evaluate",
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
        let lhs = self.lhs.prover_evaluate(builder, alloc, counts, accessor);
        let rhs = self.rhs.prover_evaluate(builder, alloc, counts, accessor);
        let n = lhs.len();
        assert_eq!(n, rhs.len());

        // lhs_and_rhs
        let lhs_and_rhs = alloc.alloc_slice_fill_with(n, |i| lhs[i] && rhs[i]);
        builder.produce_intermediate_mle(lhs_and_rhs);

        // subpolynomial: lhs_and_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(lhs_and_rhs))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(lhs)),
                    Box::new(MultilinearExtensionImpl::new(rhs)),
                ],
            ),
        ]));

        // selection
        lhs_and_rhs
    }

    fn verifier_evaluate_ark(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) -> ArkScalar {
        let lhs = self.lhs.verifier_evaluate_ark(builder, counts, accessor);
        let rhs = self.rhs.verifier_evaluate_ark(builder, counts, accessor);

        // lhs_and_rhs
        let lhs_and_rhs = builder.consume_intermediate_mle_ark();

        // subpolynomial: lhs_and_rhs - lhs * rhs
        let eval = builder.mle_evaluations.get_random_evaluation_ark() * (lhs_and_rhs - lhs * rhs);
        builder.produce_sumcheck_subpolynomial_evaluation_ark(&eval);

        // selection
        lhs_and_rhs
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
