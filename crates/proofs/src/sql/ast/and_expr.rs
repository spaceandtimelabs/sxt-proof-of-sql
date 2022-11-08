use crate::base::database::{CommitmentAccessor, DataAccessor};
use crate::sql::ast::{BoolExpr, TableExpr};
use crate::sql::proof::{ProofBuilder, ProofCounts, VerificationBuilder};

use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use dyn_partial_eq::DynPartialEq;

/// Provable logical AND expression
#[derive(Debug, DynPartialEq, PartialEq)]
#[allow(dead_code)]
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
    #[allow(unused_variables)]
    fn count(&self, counts: &mut ProofCounts) {
        todo!();
    }

    #[allow(unused_variables)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        table: &TableExpr,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        todo!();
    }

    #[allow(unused_variables)]
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        table: &TableExpr,
        accessor: &dyn CommitmentAccessor,
    ) -> Scalar {
        todo!();
    }
}
