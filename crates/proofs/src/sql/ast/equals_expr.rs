use crate::base::database::{CommitmentAccessor, DataAccessor};
use crate::sql::ast::{BoolExpr, TableExpr};
use crate::sql::proof::{ProofBuilder, ProofCounts, VerificationBuilder};

use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use dyn_partial_eq::DynPartialEq;

/// Provable AST expression for an equals expression
///
/// Note: we are currently limited only to expressions of the form
/// ```ignore
///     <col> = <constant>
/// ```
#[derive(Debug, DynPartialEq, PartialEq, Eq)]
#[allow(dead_code)]
pub struct EqualsExpr {
    column: String,
    value: Scalar,
}

impl EqualsExpr {
    /// Create a new equals expression
    pub fn new(column: String, value: Scalar) -> Self {
        Self { column, value }
    }
}

impl BoolExpr for EqualsExpr {
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
