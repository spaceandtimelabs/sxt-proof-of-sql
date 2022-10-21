use crate::base::database::{CommitmentAccessor, DataAccessor};
use crate::sql::ast::{BoolExpr, TableExpr};
use crate::sql::proof::{ProofBuilder, ProofCounts, VerificationBuilder};

use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;

/// Provable logical OR expression
#[derive(Debug)]
#[allow(dead_code)]
pub struct OrExpr {
    lhs: Box<dyn BoolExpr>,
    rhs: Box<dyn BoolExpr>,
}

impl OrExpr {
    /// Create logical OR expression
    pub fn new(lhs: Box<dyn BoolExpr>, rhs: Box<dyn BoolExpr>) -> Self {
        Self { lhs, rhs }
    }
}

impl BoolExpr for OrExpr {
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
