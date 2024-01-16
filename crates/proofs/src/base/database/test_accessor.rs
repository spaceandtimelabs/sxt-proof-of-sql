use super::{
    Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
    SchemaAccessor, TableRef,
};
use crate::base::scalar::ArkScalar;
use curve25519_dalek::ristretto::RistrettoPoint;
use proofs_sql::Identifier;

/// A trait that defines the interface for a combined metadata, schema, commitment, and data accessor for unit testing purposes.
pub trait TestAccessor:
    Clone + Default + MetadataAccessor + SchemaAccessor + CommitmentAccessor + DataAccessor
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

#[derive(Clone, Default)]
/// A test accessor that leaves all of the required methods except `new` `unimplemented!()`.
pub struct UnimplementedTestAccessor;
impl TestAccessor for UnimplementedTestAccessor {
    type Table = ();

    fn new_empty() -> Self {
        Default::default()
    }

    fn add_table(&mut self, _table_ref: TableRef, _data: (), _table_offset: usize) {
        unimplemented!()
    }

    fn get_column_names(&self, _table_ref: TableRef) -> Vec<&str> {
        unimplemented!()
    }

    fn update_offset(&mut self, _table_ref: TableRef, _new_offset: usize) {
        unimplemented!()
    }
}
impl DataAccessor for UnimplementedTestAccessor {
    fn get_column(&self, _column: ColumnRef) -> Column<ArkScalar> {
        unimplemented!()
    }
}
impl CommitmentAccessor for UnimplementedTestAccessor {
    fn get_commitment(&self, _column: ColumnRef) -> RistrettoPoint {
        unimplemented!()
    }
}
impl MetadataAccessor for UnimplementedTestAccessor {
    fn get_length(&self, _table_ref: TableRef) -> usize {
        unimplemented!()
    }

    fn get_offset(&self, _table_ref: TableRef) -> usize {
        unimplemented!()
    }
}
impl SchemaAccessor for UnimplementedTestAccessor {
    fn lookup_column(&self, _table_ref: TableRef, _column_id: Identifier) -> Option<ColumnType> {
        unimplemented!()
    }

    fn lookup_schema(&self, _table_ref: TableRef) -> Vec<(Identifier, ColumnType)> {
        unimplemented!()
    }
}
