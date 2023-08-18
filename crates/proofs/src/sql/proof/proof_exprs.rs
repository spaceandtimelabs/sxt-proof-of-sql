use super::{CountBuilder, ProofBuilder, VerificationBuilder};
use crate::base::database::{
    ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
};
use crate::base::proof::ProofError;

use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use dyn_partial_eq::dyn_partial_eq;
use std::collections::HashSet;
use std::fmt::Debug;

#[dyn_partial_eq]
pub trait ProofExpr: Debug + Send + Sync {
    /// Count terms used within the Query's proof
    fn count(
        &self,
        builder: &mut CountBuilder,
        accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError>;

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize;

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize;

    fn is_empty(&self, accessor: &dyn MetadataAccessor) -> bool {
        self.get_length(accessor) == 0
    }

    /// Evaluate the query and modify `ProofBuilder` to store an intermediate representation
    /// of the query result and track all the components needed to form the query's proof.
    ///
    /// Intermediate values that are needed to form the proof are allocated into the arena
    /// allocator alloc. These intermediate values will persist through proof creation and
    /// will be bulk deallocated once the proof is formed.
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    );

    /// Form components needed to verify and proof store into VerificationBuilder
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> Result<(), ProofError>;

    /// Return all the result column fields
    fn get_column_result_fields(&self) -> Vec<ColumnField>;

    /// Return all the columns referenced in the Query
    fn get_column_references(&self) -> HashSet<ColumnRef>;
}

#[dyn_partial_eq]
pub trait TransformExpr: Debug + Send + Sync {
    /// Apply transformations to the resulting record batch
    fn transform_results(&self, result: RecordBatch) -> RecordBatch {
        result
    }
}
