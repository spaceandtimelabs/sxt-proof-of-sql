use crate::base::database::{ColumnRef, CommitmentAccessor, DataAccessor};
use crate::sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder};

use crate::base::proof::ProofError;
use crate::base::scalar::ArkScalar;
use bumpalo::Bump;
use dyn_partial_eq::dyn_partial_eq;
use std::collections::HashSet;
use std::fmt::Debug;

/// Provable AST column expression that evaluates to a boolean
#[typetag::serde(tag = "type")]
#[dyn_partial_eq]
pub trait BoolExpr: Debug + Send + Sync {
    /// Count the number of proof terms needed for this expression
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError>;

    /// Evaluate the expression, add components needed to prove it, and return thet resulting column
    /// of boolean values
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool];

    /// Compute the evaluation of a multilinear extension from this boolean expression
    /// at the random sumcheck point and adds components needed to verify the expression to
    /// VerificationBuilder
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> Result<ArkScalar, ProofError>;

    // Insert in the HashSet `columns` all the column
    // references in the BoolExpr or forwards the call to some
    // subsequent bool_expr
    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>);
}
