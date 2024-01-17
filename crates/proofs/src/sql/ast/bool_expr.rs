use crate::{
    base::{
        database::{ColumnRef, CommitmentAccessor, DataAccessor},
        proof::ProofError,
        scalar::ArkScalar,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use dyn_partial_eq::dyn_partial_eq;
use std::{collections::HashSet, fmt::Debug};

/// Provable AST column expression that evaluates to a boolean
#[typetag::serde(tag = "type")]
#[dyn_partial_eq]
pub trait BoolExpr: Debug + Send + Sync {
    /// Count the number of proof terms needed for this expression
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError>;

    /// This returns the result of evaluating the expression on the given table, and returns
    /// a column of boolean values. This result slice is guarenteed to have length `table_length`.
    /// Implementations must ensure that the returned slice has length `table_length`.
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) -> &'a [bool];

    /// Evaluate the expression, add components needed to prove it, and return thet resulting column
    /// of boolean values
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    ) -> &'a [bool];

    /// Compute the evaluation of a multilinear extension from this boolean expression
    /// at the random sumcheck point and adds components needed to verify the expression to
    /// VerificationBuilder
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor<RistrettoPoint>,
    ) -> Result<ArkScalar, ProofError>;

    // Insert in the HashSet `columns` all the column
    // references in the BoolExpr or forwards the call to some
    // subsequent bool_expr
    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>);
}
