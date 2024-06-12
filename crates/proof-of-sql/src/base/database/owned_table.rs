use super::OwnedColumn;
use crate::base::scalar::Scalar;
use indexmap::IndexMap;
use proof_of_sql_parser::Identifier;
use thiserror::Error;

/// An error that occurs when working with tables.
#[derive(Error, Debug)]
pub enum OwnedTableError {
    /// The columns have different lengths.
    #[error("Columns have different lengths")]
    ColumnLengthMismatch,
}
/// A table of data, with schema included. This is simply a map from `Identifier` to `OwnedColumn`,
/// where columns order matters.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow RecordBatch.
#[derive(Debug, Clone, Eq)]
pub struct OwnedTable<S: Scalar> {
    table: IndexMap<Identifier, OwnedColumn<S>>,
}
impl<S: Scalar> OwnedTable<S> {
    /// Creates a new OwnedTable.
    pub fn try_new(table: IndexMap<Identifier, OwnedColumn<S>>) -> Result<Self, OwnedTableError> {
        if table.is_empty() {
            return Ok(Self { table });
        }
        let num_rows = table[0].len();
        if table.values().any(|column| column.len() != num_rows) {
            Err(OwnedTableError::ColumnLengthMismatch)
        } else {
            Ok(Self { table })
        }
    }
    /// Creates a new OwnedTable.
    pub fn try_from_iter<T: IntoIterator<Item = (Identifier, OwnedColumn<S>)>>(
        iter: T,
    ) -> Result<Self, OwnedTableError> {
        Self::try_new(IndexMap::from_iter(iter))
    }
    /// Number of columns in the table.
    pub fn num_columns(&self) -> usize {
        self.table.len()
    }
    /// Number of rows in the table.
    pub fn num_rows(&self) -> usize {
        if self.table.is_empty() {
            0
        } else {
            self.table[0].len()
        }
    }
    /// Whether the table has no columns.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
    /// Returns the columns of this table as an IndexMap
    pub fn into_inner(self) -> IndexMap<Identifier, OwnedColumn<S>> {
        self.table
    }
    /// Returns the columns of this table as an IndexMap
    pub fn inner_table(&self) -> &IndexMap<Identifier, OwnedColumn<S>> {
        &self.table
    }
    /// Returns the columns of this table as an Iterator
    pub fn column_names(&self) -> impl Iterator<Item = &Identifier> {
        self.table.keys()
    }

    /// Applies a filter to this table via polars, returning a new table. This is useful for testing that a filter is executed correctly.
    #[cfg(test)]
    pub fn apply_polars_filter(
        &self,
        results: &[&str],
        predicate: polars::prelude::Expr,
    ) -> OwnedTable<S> {
        OwnedTable::try_from(
            super::dataframe_to_record_batch(
                polars::prelude::IntoLazy::lazy(
                    super::record_batch_to_dataframe(
                        arrow::record_batch::RecordBatch::try_from(self.clone()).unwrap(),
                    )
                    .unwrap(),
                )
                .filter(predicate)
                .select(
                    results
                        .iter()
                        .map(|v| polars::prelude::col(v))
                        .collect::<Vec<_>>()
                        .as_slice(),
                )
                .collect()
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }
}

// Note: we modify the default PartialEq for IndexMap to also check for column ordering.
// This is to align with the behaviour of a `RecordBatch`.
impl<S: Scalar> PartialEq for OwnedTable<S> {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
            && self
                .table
                .keys()
                .zip(other.table.keys())
                .all(|(a, b)| a == b)
    }
}
