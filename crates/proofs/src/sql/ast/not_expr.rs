use crate::base::database::{CommitmentAccessor, DataAccessor};
use crate::sql::ast::{BoolExpr, TableExpr};
use crate::sql::proof::{ProofBuilder, ProofCounts, VerificationBuilder};

use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use dyn_partial_eq::DynPartialEq;

/// Provable logical NOT expression
#[derive(Debug, DynPartialEq, PartialEq)]
#[allow(dead_code)]
pub struct NotExpr {
    expr: Box<dyn BoolExpr>,
}

impl NotExpr {
    /// Create logical NOT expression
    pub fn new(expr: Box<dyn BoolExpr>) -> Self {
        Self { expr }
    }
}

impl BoolExpr for NotExpr {
    #[allow(unused_variables)]
    fn count(&self, counts: &mut ProofCounts) {
        todo!();
    }

    #[allow(unused_variables)]
    fn prove<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        table: &TableExpr,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        todo!();
    }

    #[allow(unused_variables)]
    fn verify(
        &self,
        builder: &mut VerificationBuilder,
        table: &TableExpr,
        accessor: &dyn CommitmentAccessor,
    ) -> Scalar {
        todo!();
    }
}
