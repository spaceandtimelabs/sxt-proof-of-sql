use crate::{
    base::{
        database::{ColumnRef, CommitmentAccessor, DataAccessor},
        proof::ProofError,
        scalar::ArkScalar,
    },
    sql::{
        ast::BoolExpr,
        proof::{
            CountBuilder, MultilinearExtensionImpl, ProofBuilder, SumcheckSubpolynomial,
            SumcheckSubpolynomialType, VerificationBuilder,
        },
    },
};
use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use num_traits::One;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable logical AND expression
#[derive(Debug, DynPartialEq, PartialEq, Serialize, Deserialize)]
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

#[typetag::serde]
impl BoolExpr for AndExpr {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        builder.count_subpolynomials(1);
        builder.count_intermediate_mles(1);
        builder.count_degree(3);
        Ok(())
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
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        let lhs = self.lhs.prover_evaluate(builder, alloc, accessor);
        let rhs = self.rhs.prover_evaluate(builder, alloc, accessor);
        let n = lhs.len();
        assert_eq!(n, rhs.len());

        // lhs_and_rhs
        let lhs_and_rhs = alloc.alloc_slice_fill_with(n, |i| lhs[i] && rhs[i]);
        builder.produce_intermediate_mle(lhs_and_rhs);

        // subpolynomial: lhs_and_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(
            SumcheckSubpolynomialType::Identity,
            vec![
                (
                    ArkScalar::one(),
                    vec![Box::new(MultilinearExtensionImpl::new(lhs_and_rhs))],
                ),
                (
                    -ArkScalar::one(),
                    vec![
                        Box::new(MultilinearExtensionImpl::new(lhs)),
                        Box::new(MultilinearExtensionImpl::new(rhs)),
                    ],
                ),
            ],
        ));

        // selection
        lhs_and_rhs
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> Result<ArkScalar, ProofError> {
        let lhs = self.lhs.verifier_evaluate(builder, accessor)?;
        let rhs = self.rhs.verifier_evaluate(builder, accessor)?;

        // lhs_and_rhs
        let lhs_and_rhs = builder.consume_intermediate_mle();

        // subpolynomial: lhs_and_rhs - lhs * rhs
        let eval = builder.mle_evaluations.random_evaluation * (lhs_and_rhs - lhs * rhs);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);

        // selection
        Ok(lhs_and_rhs)
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
