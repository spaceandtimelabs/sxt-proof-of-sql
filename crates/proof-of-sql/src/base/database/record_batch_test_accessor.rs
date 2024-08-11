use super::{
    dataframe_to_record_batch, record_batch_to_dataframe, ArrayRefExt, Column, ColumnRef,
    ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor, TableRef,
    TestAccessor,
};
use crate::base::scalar::{compute_commitment_for_testing, Curve25519Scalar};
use arrow::{array::ArrayRef, datatypes::DataType, record_batch::RecordBatch};
use bumpalo::Bump;
use curve25519_dalek::ristretto::RistrettoPoint;
use indexmap::IndexMap;
use polars::prelude::DataFrame;
use proof_of_sql_parser::Identifier;

/// TestTable is used to simulate an in-memory table and commitment tracking table.
#[derive(Clone)]
struct TestAccessorTable {
    data: RecordBatch,
    table_offset: usize,
    columns: IndexMap<Identifier, ArrayRef>,
    commitments: IndexMap<Identifier, RistrettoPoint>,
}

/// TestAccessor is used to simulate an in-memory databasefor proof testing.
pub struct RecordBatchTestAccessor {
    alloc: Bump,
    tables: IndexMap<TableRef, TestAccessorTable>,
}

impl Clone for RecordBatchTestAccessor {
    fn clone(&self) -> Self {
        Self {
            alloc: Bump::new(),
            tables: self.tables.clone(),
        }
    }
}

impl Default for RecordBatchTestAccessor {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl TestAccessor<RistrettoPoint> for RecordBatchTestAccessor {
    type Table = RecordBatch;

    fn new_empty() -> Self {
        Self {
            alloc: Bump::new(),
            tables: IndexMap::new(),
        }
    }
    fn add_table(&mut self, table_ref: TableRef, data: RecordBatch, table_offset: usize) {
        assert!(!self.tables.contains_key(&table_ref));

        let columns: IndexMap<_, _> = data
            .schema()
            .fields()
            .iter()
            .zip(data.columns())
            .map(|(f, v)| (f.name().parse().unwrap(), v.clone()))
            .collect();

        let commitments = columns
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    compute_commitment_for_testing(
                        &v.to_curve25519_scalars().unwrap()[..],
                        table_offset,
                    ),
                )
            })
            .collect();

        self.tables.insert(
            table_ref,
            TestAccessorTable {
                table_offset,
                commitments,
                data,
                columns,
            },
        );
    }

    fn get_column_names(&self, table_ref: TableRef) -> Vec<&str> {
        assert_eq!(self.tables.len(), 1);
        let table = self.tables.get(&table_ref).unwrap();
        table.columns.keys().map(|c| c.as_str()).collect()
    }

    fn update_offset(&mut self, table_ref: TableRef, new_offset: usize) {
        let table = self.tables.get_mut(&table_ref).unwrap();

        table.table_offset = new_offset;
        table.commitments = table
            .columns
            .iter()
            .map(|(k, col)| {
                (
                    *k,
                    compute_commitment_for_testing(
                        &col.to_curve25519_scalars().unwrap()[..],
                        new_offset,
                    ),
                )
            })
            .collect();
    }
}

impl RecordBatchTestAccessor {
    /// Apply a query function to table and then convert the result to a RecordBatch
    pub fn query_table(
        &self,
        table_ref: TableRef,
        f: impl Fn(&DataFrame) -> DataFrame,
    ) -> RecordBatch {
        let table = self.tables.get(&table_ref).unwrap();

        dataframe_to_record_batch(f(&record_batch_to_dataframe(table.data.clone()).unwrap()))
            .unwrap()
    }
}

/// MetadataAccessor implementation for TestAccessor
impl MetadataAccessor for RecordBatchTestAccessor {
    /// Return the table length associated with table_ref
    ///
    /// Note: this function expects table_ref to exist
    fn get_length(&self, table_ref: TableRef) -> usize {
        let table = self.tables.get(&table_ref).unwrap();
        table.data.num_rows()
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
impl SchemaAccessor for RecordBatchTestAccessor {
    /// Return the column type associated with column_id, if exists.
    ///
    /// Note: this function expects `table_ref` and `column_id` to exist
    fn lookup_column(&self, table_ref: TableRef, column_id: Identifier) -> Option<ColumnType> {
        let table = self.tables.get(&table_ref)?;
        table
            .columns
            .get(&column_id)
            .map(|dt| DataType::try_into(dt.data_type().clone()).unwrap())
    }

    /// Return the column schema + column type associated with table_ref
    ///
    /// Note: this function expects table_ref to exist
    fn lookup_schema(&self, table_ref: TableRef) -> Vec<(Identifier, ColumnType)> {
        let table = self.tables.get(&table_ref).unwrap();
        table
            .columns
            .iter()
            .map(|(k, dt)| (*k, DataType::try_into(dt.data_type().clone()).unwrap()))
            .collect()
    }
}

/// CommitmentAccessor implementation for TestAccessor
impl CommitmentAccessor<RistrettoPoint> for RecordBatchTestAccessor {
    /// Return the commitment associated with column_ref
    ///
    /// Note: this function expects the column_ref to exist
    fn get_commitment(&self, column_ref: ColumnRef) -> RistrettoPoint {
        let table = self.tables.get(&column_ref.table_ref()).unwrap();
        *table.commitments.get(&column_ref.column_id()).unwrap()
    }
}

/// DataAccessor implementation for TestAccessor
impl DataAccessor<Curve25519Scalar> for RecordBatchTestAccessor {
    /// Return the data slice wrapped within the Column::<some_type>
    ///
    /// Note: this function expects the column_ref to exist
    /// and also have the same type specified by column_ref.column_type()
    fn get_column(&self, column_ref: ColumnRef) -> Column<Curve25519Scalar> {
        let table = self.tables.get(&column_ref.table_ref()).unwrap();
        table
            .columns
            .get(&column_ref.column_id())
            .unwrap()
            .to_column(&self.alloc, &(0..table.data.num_rows()), None)
            .unwrap()
    }
}
