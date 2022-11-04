use crate::base::database::{CommitmentAccessor, DataAccessor};
use crate::sql::ast::TableExpr;
use crate::sql::proof::{ProofBuilder, ProofCounts, VerificationBuilder};

use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;

/// Provable expression for a result column within a filter SQL expression
///
/// Note: this is currently limited to named column expressions.
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct FilterResultExpr {
    column: String,
}

impl FilterResultExpr {
    /// Creates a new filter result expression
    pub fn new(column: String) -> Self {
        Self { column }
    }

    /// Count the number of proof terms needed by this expression
    #[allow(unused_variables)]
    pub fn count(&self, counts: &mut ProofCounts) {
        todo!();
    }

    /// Given the selected rows (as a slice of booleans), evaluate the filter result expression and
    /// add the components needed to prove the result
    #[allow(unused_variables)]
    pub fn prove<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        table: &TableExpr,
        accessor: &'a dyn DataAccessor,
        selection: &'a [bool],
    ) {
        todo!();
    }

    /// Give the evaluation of the selected row's multilinear extension at sumcheck's random point,
    /// add components needed to verify this filter result expression
    #[allow(unused_variables)]
    pub fn verify(
        &self,
        builder: &mut VerificationBuilder,
        table: &TableExpr,
        accessor: &dyn CommitmentAccessor,
        selection_eval: Scalar,
    ) {
        todo!();
    }
}
