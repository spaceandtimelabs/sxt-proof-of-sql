use super::{CountBuilder, ProofBuilder, ResultBuilder, VerificationBuilder};
use crate::base::{
    database::{ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor},
    proof::ProofError,
    scalar::ArkScalar,
};
use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use dyn_partial_eq::dyn_partial_eq;
use std::{collections::HashSet, fmt::Debug};

#[typetag::serde(tag = "type")]
#[dyn_partial_eq]
/// A trait that represents a query expression that can be proven.
/// This is simply a "wrapper" around [`ProofExpr`]
/// that allows us to more easily implement `DynPartialEq`, `Serialize`, and `Deserialize`.
pub trait SerializableProofExpr: ProofExpr {}
pub trait ProofExpr: Debug + Send + Sync + ProverEvaluate {
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

    /// Form components needed to verify and proof store into VerificationBuilder
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor<RistrettoPoint>,
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

pub trait ProverEvaluate {
    /// Evaluate the query and modify `ResultBuilder` to track the result of the query.
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<ArkScalar>,
    );

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
        accessor: &'a dyn DataAccessor<ArkScalar>,
    );
}

/// Marker used as a trait bound for generic [`ProofExpr`] types to indicate the honesty of their implementation.
///
/// This allows us to define alternative prover implementations that misbehave, and test that the verifier rejects their results.
pub trait ProverHonestyMarker: Debug + Send + Sync + PartialEq + 'static {}

/// [`ProverHonestyMarker`] for generic [`ProofExpr`] types whose implementation is canonical/honest.
#[derive(Debug, PartialEq)]
pub struct HonestProver;
impl ProverHonestyMarker for HonestProver {}
