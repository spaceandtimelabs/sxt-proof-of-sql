use super::{
    Column, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor,
};

use crate::base::scalar::compute_commitment_for_testing;

use curve25519_dalek::ristretto::RistrettoPoint;
use polars::prelude::{DataFrame, NamedFrom, Series};
use std::collections::HashMap;

struct TestTable {
    /// The total number of rows in the table. Every element in `columns` field must have a Vec<i64> with that same length.
    len: usize,

    /// commitments of each column
    commitments: HashMap<String, RistrettoPoint>,

    /// the column values
    data: DataFrame,
}

/// TestAccessor is used to simulate an in-memory database and commitment tracking database for proof testing.
#[derive(Default)]
pub struct TestAccessor {
    /// This `data` field defines a HashMap with pairs of table_name and their respective table values
    /// (columns with their associated rows and commitment values).
    data: HashMap<String, TestTable>,
}

impl TestAccessor {
    /// Creates an empty Test Accessor
    pub fn new() -> Self {
        TestAccessor {
            data: HashMap::new(),
        }
    }

    /// Adds a new table (with associated rows and commitment values) to the current test accessor.
    ///
    /// Note 1: we assume that the `columns` argument is nonempty
    /// and all elements in it have the same Vec<i64> length.
    ///
    /// Note 2: for simplicity, we assume that `table_name` was not
    /// previously added to the accessor.
    pub fn add_table(&mut self, table_name: &str, columns: &HashMap<String, Vec<i64>>) {
        assert!(!columns.is_empty());
        assert!(!self.data.contains_key(table_name));

        // gets the first element, then its Vec<i64> length (number of rows)
        let num_rows_table = columns.values().next().unwrap().len();

        // computes the commitment of each column and adds it with its rows to `table_data`
        let mut cols: Vec<Series> = Vec::with_capacity(columns.len());
        let mut commitments = HashMap::new();
        for (col_name, col_rows) in columns {
            // all columns must have the same length
            assert_eq!(col_rows.len(), num_rows_table);

            cols.push(Series::new(col_name, &col_rows));
            let commitment = compute_commitment_for_testing(col_rows);

            commitments.insert(col_name.to_string(), commitment);
        }

        self.data.insert(
            table_name.to_string(),
            TestTable {
                len: num_rows_table,
                commitments,
                data: DataFrame::new(cols).unwrap(),
            },
        );
    }
}

/// This accessor fetches the total number of rows associated with the given `table_name`.
///
/// Note: `table_name` must already exist.
impl MetadataAccessor for TestAccessor {
    fn get_length(&self, table_name: &str) -> usize {
        self.data.get(table_name).unwrap().len
    }
}

/// This accessor fetches the rows data associated with the given `table_name` and `column_name`.
///
/// Note: `table_name` and `column_name` must already exist.
impl DataAccessor for TestAccessor {
    fn get_column(&self, table_name: &str, column_name: &str) -> Column {
        let column = &self
            .data
            .get(table_name)
            .unwrap()
            .data
            .column(column_name)
            .unwrap();
        let data = column.i64().unwrap().cont_slice().unwrap();
        Column::BigInt(data)
    }
}

/// This accessor fetches the commitment value associated with the given `table_name` and `column_name`.
///
/// Note: `table_name` and `column_name` must already exist.
impl CommitmentAccessor for TestAccessor {
    fn get_commitment(&self, table_name: &str, column_name: &str) -> RistrettoPoint {
        let commitments = &self.data.get(table_name).unwrap().commitments;
        *commitments.get(column_name).unwrap()
    }
}

impl SchemaAccessor for TestAccessor {
    fn lookup_column(&self, table_name: &str, column_name: &str) -> Option<ColumnType> {
        let df = &self.data.get(table_name).unwrap().data;
        let column = df.column(column_name);

        if column.is_ok() {
            return Some(ColumnType::BigInt);
        }

        None
    }
}
