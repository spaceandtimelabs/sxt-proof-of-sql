use super::{data_frame_to_accessors, data_frame_to_record_batch, TestAccessorColumns};
use super::{
    Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
    SchemaAccessor, TableRef,
};

use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use indexmap::IndexMap;
use polars::prelude::DataFrame;
use proofs_sql::Identifier;
use std::collections::HashMap;

/// TestTable is used to simulate an in-memory table and commitment tracking table.
#[derive(Clone)]
pub struct TestAccessorTable {
    table_length: usize,
    table_offset: usize,
    data: DataFrame,
    columns: TestAccessorColumns,
    schema: IndexMap<Identifier, ColumnType>,
    commitments: IndexMap<Identifier, RistrettoPoint>,
}

/// TestAccessor is used to simulate an in-memory databasefor proof testing.
pub struct TestAccessor {
    alloc: Bump,
    tables: HashMap<TableRef, TestAccessorTable>,
}

impl Clone for TestAccessor {
    fn clone(&self) -> Self {
        Self {
            alloc: Bump::new(),
            tables: self.tables.clone(),
        }
    }
}

impl Default for TestAccessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TestAccessor {
    /// Create an empty test accessor
    pub fn new() -> Self {
        Self {
            alloc: Bump::new(),
            tables: HashMap::new(),
        }
    }

    /// Add a new table to the current test accessor
    pub fn add_table(&mut self, table_ref: TableRef, data: DataFrame, table_offset: usize) {
        assert!(self.tables.get(&table_ref).is_none());

        let (table_length, columns) = data_frame_to_accessors(&data);

        let schema = columns.iter().map(|(k, v)| (*k, v.column_type())).collect();

        let commitments = columns
            .iter()
            .map(|(k, v)| (*k, v.compute_commitment(table_offset)))
            .collect();

        self.tables.insert(
            table_ref,
            TestAccessorTable {
                table_length,
                table_offset,
                schema,
                commitments,
                columns,
                data,
            },
        );
    }

    /// Update the table offset alongside its column commitments
    pub fn update_offset(&mut self, table_ref: TableRef, new_offset: usize) {
        let table = self.tables.get_mut(&table_ref).unwrap();

        table.table_offset = new_offset;
        table.commitments = table
            .columns
            .iter()
            .map(|(k, col)| (*k, col.compute_commitment(new_offset)))
            .collect();
    }

    /// Apply a query function to table and then convert the result to a RecordBatch
    pub fn query_table(
        &self,
        table_ref: TableRef,
        f: impl Fn(&DataFrame) -> DataFrame,
    ) -> RecordBatch {
        let table = self.tables.get(&table_ref).unwrap();
        data_frame_to_record_batch(&f(&table.data))
    }
}

/// MetadataAccessor implementation for TestAccessor
impl MetadataAccessor for TestAccessor {
    /// Return the table length associated with table_ref
    ///
    /// Note: this function expects table_ref to exist
    fn get_length(&self, table_ref: TableRef) -> usize {
        let table = self.tables.get(&table_ref).unwrap();
        table.table_length
    }

    /// Return the offset associated with table_ref
    ///
    /// Note: this function expects table_ref to exist
    fn get_offset(&self, table_ref: TableRef) -> usize {
        let table = self.tables.get(&table_ref).unwrap();
        table.table_offset
    }
}

/// SchemaAccessor implementation for TestAccessor
impl SchemaAccessor for TestAccessor {
    /// Return the column type associated with column_id, if exists.
    ///
    /// Note: this function expects `table_ref` and `column_id` to exist
    fn lookup_column(&self, table_ref: TableRef, column_id: Identifier) -> Option<ColumnType> {
        let table = self.tables.get(&table_ref)?;
        table.schema.get(&column_id).copied()
    }

    /// Return the column schema + column type associated with table_ref
    ///
    /// Note: this function expects table_ref to exist
    fn lookup_schema(&self, table_ref: TableRef) -> Vec<(Identifier, ColumnType)> {
        let table = self.tables.get(&table_ref).unwrap();
        table.schema.iter().map(|(v, k)| (*v, *k)).collect()
    }
}

/// CommitmentAccessor implementation for TestAccessor
impl CommitmentAccessor for TestAccessor {
    /// Return the commitment associated with column_ref
    ///
    /// Note: this function expects the column_ref to exist
    fn get_commitment(&self, column_ref: ColumnRef) -> RistrettoPoint {
        let table = self.tables.get(&column_ref.table_ref()).unwrap();
        *table.commitments.get(&column_ref.column_id()).unwrap()
    }
}

/// DataAccessor implementation for TestAccessor
impl DataAccessor for TestAccessor {
    /// Return the data slice wrapped within the Column::<some_type>
    ///
    /// Note: this function expects the column_ref to exist
    /// and also have the same type specified by column_ref.column_type()
    fn get_column(&self, column_ref: ColumnRef) -> Column {
        let table = self.tables.get(&column_ref.table_ref()).unwrap();
        table
            .columns
            .get(&column_ref.column_id())
            .unwrap()
            .to_column(*column_ref.column_type(), &self.alloc)
    }
}
