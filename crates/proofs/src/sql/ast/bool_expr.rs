use crate::base::database::{CommitmentAccessor, DataAccessor};
use crate::sql::proof::{ProofBuilder, ProofCounts, VerificationBuilder};

use bumpalo::Bump;
use curve25519_dalek::scalar::Scalar;
use dyn_partial_eq::dyn_partial_eq;
use std::fmt::Debug;

/// Provable AST column expression that evaluates to a boolean
#[dyn_partial_eq]
pub trait BoolExpr: Debug + Send + Sync {
    /// Count the number of proof terms needed for this expression
    fn count(&self, counts: &mut ProofCounts);

    /// Evaluate the expression, add components needed to prove it, and return thet resulting column
    /// of boolean values
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool];

    /// Compute the evaluation of a multilinear extension from this boolean expression
    /// at the random sumcheck point and adds components needed to verify the expression to
    /// VerificationBuilder
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        accessor: &dyn CommitmentAccessor,
    ) -> Scalar;
}
