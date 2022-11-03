use super::{ProofBuilder, ProofCounts, VerificationBuilder};

use crate::base::database::{CommitmentAccessor, DataAccessor, MetadataAccessor};

use bumpalo::Bump;
use std::fmt::Debug;

/// A query expression that we can evaluate, prove, and verify
pub trait QueryExpr: Debug {
    /// Count terms used within the Query's proof
    fn count(&self, counts: &mut ProofCounts, accessor: &dyn MetadataAccessor);

    /// Evaluate the query and modify `ProofBuilder` to store an intermediate representation
    /// of the query result and track all the components needed to form the query's proof.
    ///
    /// Intermediate values that are needed to form the proof are allocated into the arena
    /// allocator alloc. These intermediate values will persist through proof creation and
    /// will be bulk deallocated once the proof is formed.
    fn prove<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    );

    /// Form components needed to verify and proof store into VerificationBuilder
    fn verify(&self, builder: &mut VerificationBuilder, accessor: &dyn CommitmentAccessor);
}
