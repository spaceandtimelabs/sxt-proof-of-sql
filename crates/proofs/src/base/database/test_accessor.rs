use super::{Column, CommitmentAccessor, DataAccessor, MetadataAccessor};

use crate::base::scalar::compute_commitment_for_testing;
use curve25519_dalek::ristretto::RistrettoPoint;
use std::collections::HashMap;

struct TestTable {
    /// The total number of rows in the table. Every element in `columns` field must have a Vec<i64> with that same length.
    len: usize,
    /// The pairs of column_name and their respective rows data and commitment value (comprising all rows).
    columns: HashMap<String, (RistrettoPoint, Vec<i64>)>,
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

        let mut table_data = HashMap::new();

        // gets the first element, then its Vec<i64> length (number of rows)
        let num_rows_table = columns.values().next().unwrap().len();

        // computes the commitment of each column and adds it with its rows to `table_data`
        for (col_name, col_rows) in columns {
            // all columns must have the same length
            assert_eq!(col_rows.len(), num_rows_table);

            let commitment = compute_commitment_for_testing(col_rows);

            table_data.insert(col_name.to_string(), (commitment, col_rows.clone()));
        }

        self.data.insert(
            table_name.to_string(),
            TestTable {
                len: num_rows_table,
                columns: table_data,
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
        let columns = &self.data.get(table_name).unwrap().columns;
        let column = &columns.get(column_name).unwrap();
        let column_rows = &column.1;

        Column::BigInt(column_rows)
    }
}

/// This accessor fetches the commitment value associated with the given `table_name` and `column_name`.
///
/// Note: `table_name` and `column_name` must already exist.
impl CommitmentAccessor for TestAccessor {
    fn get_commitment(&self, table_name: &str, column_name: &str) -> RistrettoPoint {
        let columns = &self.data.get(table_name).unwrap().columns;
        let column = &columns.get(column_name).unwrap();

        column.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_accessor() {
        let mut accessor = TestAccessor::new();

        accessor.add_table(
            "test",
            &HashMap::from([
                ("a".to_string(), vec![1, 2, 3]),
                ("b".to_string(), vec![4, 5, 6]),
            ]),
        );

        assert_eq!(accessor.get_length("test"), 3);

        accessor.add_table(
            "test2",
            &HashMap::from([
                ("a".to_string(), vec![1, 2, 3, 4]),
                ("b".to_string(), vec![4, 5, 6, 5]),
            ]),
        );

        assert_eq!(accessor.get_length("test"), 3);
        assert_eq!(accessor.get_length("test2"), 4);
    }

    #[test]
    fn test_data_accessor() {
        let mut accessor = TestAccessor::new();

        accessor.add_table(
            "test",
            &HashMap::from([
                ("a".to_string(), vec![1, 2, 3]),
                ("b".to_string(), vec![4, 5, 6]),
            ]),
        );

        match accessor.get_column("test", "b") {
            Column::BigInt(col) => assert_eq!(col.to_vec(), vec![4, 5, 6]),
        };

        accessor.add_table(
            "test2",
            &HashMap::from([
                ("a".to_string(), vec![1, 2, 3, 4]),
                ("b".to_string(), vec![4, 5, 6, 5]),
            ]),
        );

        match accessor.get_column("test", "a") {
            Column::BigInt(col) => assert_eq!(col.to_vec(), vec![1, 2, 3]),
        };

        match accessor.get_column("test2", "b") {
            Column::BigInt(col) => assert_eq!(col.to_vec(), vec![4, 5, 6, 5]),
        };
    }

    #[test]
    fn test_commitment_accessor() {
        let mut accessor = TestAccessor::new();

        accessor.add_table(
            "test",
            &HashMap::from([
                ("a".to_string(), vec![1, 2, 3]),
                ("b".to_string(), vec![4, 5, 6]),
            ]),
        );

        assert_eq!(
            accessor.get_commitment("test", "b"),
            compute_commitment_for_testing(&[4, 5, 6])
        );

        accessor.add_table(
            "test2",
            &HashMap::from([
                ("a".to_string(), vec![1, 2, 3, 4]),
                ("b".to_string(), vec![4, 5, 6, 5]),
            ]),
        );

        assert_eq!(
            accessor.get_commitment("test", "a"),
            compute_commitment_for_testing(&[1, 2, 3])
        );
        assert_eq!(
            accessor.get_commitment("test2", "b"),
            compute_commitment_for_testing(&[4, 5, 6, 5])
        );
    }
}
