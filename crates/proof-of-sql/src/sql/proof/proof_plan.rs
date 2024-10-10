use super::{CountBuilder, ProofBuilder, ResultBuilder, VerificationBuilder};
use crate::base::{
    commitment::Commitment,
    database::{
        Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
        OwnedTable,
    },
    map::IndexSet,
    proof::ProofError,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::fmt::Debug;

/// Provable nodes in the provable AST.
pub trait ProofPlan<C: Commitment>: Debug + Send + Sync + ProverEvaluate<C> {
    /// Count terms used within the Query's proof
    fn count(
        &self,
        builder: &mut CountBuilder,
        accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError>;

    /// The offset of the query, that is, how many rows to skip before starting to read the input table
    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize;

    /// Form components needed to verify and proof store into `VerificationBuilder`
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError>;

    /// Return all the result column fields
    fn get_column_result_fields(&self) -> Vec<ColumnField>;

    /// Return all the columns referenced in the Query
    fn get_column_references(&self) -> IndexSet<ColumnRef>;
}

pub trait ProverEvaluate<C: Commitment> {
    /// Evaluate the query and modify `ResultBuilder` to track the result of the query.
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>>;

    /// Evaluate the query and modify `ProofBuilder` to store an intermediate representation
    /// of the query result and track all the components needed to form the query's proof.
    ///
    /// Intermediate values that are needed to form the proof are allocated into the arena
    /// allocator alloc. These intermediate values will persist through proof creation and
    /// will be bulk deallocated once the proof is formed.
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>>;

    /// The length of the input table
    fn get_input_length<'a>(
        &self,
        builder: &mut ResultBuilder,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> usize;

    /// Check if the input table is empty
    fn is_empty<'a>(
        &self,
        builder: &mut ResultBuilder,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> bool {
        self.get_input_length(builder, alloc, accessor) == 0
    }

    /// The length of the output table
    fn get_output_length<'a>(
        &self,
        builder: &mut ResultBuilder,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> usize {
        self.result_evaluate(builder, alloc, accessor)
            .first()
            .map_or(0, |column| column.len())
    }
}

/// Marker used as a trait bound for generic [`ProofPlan`] types to indicate the honesty of their implementation.
///
/// This allows us to define alternative prover implementations that misbehave, and test that the verifier rejects their results.
pub trait ProverHonestyMarker: Debug + Send + Sync + PartialEq + 'static {}

/// [`ProverHonestyMarker`] for generic [`ProofPlan`] types whose implementation is canonical/honest.
#[derive(Debug, PartialEq)]
pub struct HonestProver;
impl ProverHonestyMarker for HonestProver {}
