use super::{CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor, TableRef};
use crate::base::commitment::Commitment;
use alloc::vec::Vec;

/// A trait that defines the interface for a combined metadata, schema, commitment, and data accessor for unit testing or example purposes.
pub trait TestAccessor<C: Commitment>:
    Clone
    + Default
    + MetadataAccessor
    + SchemaAccessor
    + CommitmentAccessor<C>
    + DataAccessor<C::Scalar>
{
    /// The table type that the accessor will accept in the `add_table` method, and likely the inner table type.
    type Table;

    /// Create an empty test accessor
    fn new_empty() -> Self;

    /// Add a new table to the current test accessor
    fn add_table(&mut self, table_ref: TableRef, data: Self::Table, table_offset: usize);

    /// Get the column names for a given table
    fn get_column_names(&self, table_ref: TableRef) -> Vec<&str>;

    /// Update the table offset alongside its column commitments
    fn update_offset(&mut self, table_ref: TableRef, new_offset: usize);
}
