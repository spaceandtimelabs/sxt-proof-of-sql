use super::{CountBuilder, FinalRoundBuilder, FirstRoundBuilder, VerificationBuilder};
use crate::base::{
    commitment::Commitment,
    database::{
        Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
        OwnedTable, TableRef,
    },
    map::IndexSet,
    proof::ProofError,
    scalar::Scalar,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use core::fmt::Debug;

/// Provable nodes in the provable AST.
pub trait ProofPlan<C: Commitment>: Debug + Send + Sync + ProverEvaluate<C::Scalar> {
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

    /// Return all the columns referenced in the query
    fn get_column_references(&self) -> IndexSet<ColumnRef>;

    /// Return all the tables referenced in the query
    fn get_table_references(&self) -> IndexSet<TableRef> {
        self.get_column_references()
            .iter()
            .map(ColumnRef::table_ref)
            .collect()
    }

    /// Check if the input tables are all empty
    fn is_empty(&self, accessor: &dyn MetadataAccessor) -> bool {
        self.get_table_references()
            .iter()
            .all(|table_ref| accessor.get_length(*table_ref) == 0)
    }
}

pub trait ProverEvaluate<S: Scalar> {
    /// Get the length of the input tables
    fn get_input_lengths<'a>(
        &self,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<usize>;

    /// Get the length of the output table
    fn get_output_length<'a>(&self, alloc: &'a Bump, accessor: &'a dyn DataAccessor<S>) -> usize;

    /// Evaluate the query and modify `FirstRoundBuilder` to track the result of the query.
    fn result_evaluate<'a>(
        &self,
        input_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>>;

    /// Evaluate the query and modify `FirstRoundBuilder` to form the query's proof.
    fn first_round_evaluate(&self, builder: &mut FirstRoundBuilder);

    /// Evaluate the query and modify `FinalRoundBuilder` to store an intermediate representation
    /// of the query result and track all the components needed to form the query's proof.
    ///
    /// Intermediate values that are needed to form the proof are allocated into the arena
    /// allocator alloc. These intermediate values will persist through proof creation and
    /// will be bulk deallocated once the proof is formed.
    fn final_round_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>>;
}

/// Marker used as a trait bound for generic [`ProofPlan`] types to indicate the honesty of their implementation.
///
/// This allows us to define alternative prover implementations that misbehave, and test that the verifier rejects their results.
pub trait ProverHonestyMarker: Debug + Send + Sync + PartialEq + 'static {}

/// [`ProverHonestyMarker`] for generic [`ProofPlan`] types whose implementation is canonical/honest.
#[derive(Debug, PartialEq)]
pub struct HonestProver;
impl ProverHonestyMarker for HonestProver {}
