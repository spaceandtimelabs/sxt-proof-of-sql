use super::{Column, ColumnRef, DataAccessor, TableRef};
use crate::base::{
    map::{IndexMap, IndexSet},
    scalar::Scalar,
};
use alloc::vec;
use bumpalo::Bump;
use proof_of_sql_parser::Identifier;
use snafu::Snafu;

/// An error that occurs when working with tables.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum TableError {
    /// The columns have different lengths.
    #[snafu(display("Columns have different lengths"))]
    ColumnLengthMismatch,
}
/// A table of data, with schema included. This is simply a map from `Identifier` to `Column`,
/// where columns order matters.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow [`RecordBatch`](arrow::record_batch::RecordBatch).
#[derive(Debug, Clone, Eq)]
pub struct Table<'a, S: Scalar> {
    table: IndexMap<Identifier, Column<'a, S>>,
    num_rows: usize,
}
impl<'a, S: Scalar> Table<'a, S> {
    /// Creates a new [`Table`].
    pub fn try_new(table: IndexMap<Identifier, Column<'a, S>>) -> Result<Self, TableError> {
        if table.is_empty() {
            // `EmptyExec` should have one row for queries such as `SELECT 1`.
            return Ok(Self { table, num_rows: 1 });
        }
        let num_rows = table[0].len();
        if table.values().any(|column| column.len() != num_rows) {
            Err(TableError::ColumnLengthMismatch)
        } else {
            Ok(Self { table, num_rows })
        }
    }
    /// Creates a new [`Table`].
    pub fn try_from_iter<T: IntoIterator<Item = (Identifier, Column<'a, S>)>>(
        iter: T,
    ) -> Result<Self, TableError> {
        Self::try_new(IndexMap::from_iter(iter))
    }
    /// Creates a new [`Table`] from a [`DataAccessor`], [`TableRef`] and [`ColumnRef`]s.
    ///
    /// Columns are retrieved from the [`DataAccessor`] using the provided [`ColumnRef`]s.
    /// # Panics
    /// Missing columns or column length mismatches can occur if the accessor doesn't
    /// contain the necessary columns. In practice, this should not happen.
    pub(crate) fn from_columns(
        column_refs: &IndexSet<ColumnRef>,
        table_ref: TableRef,
        accessor: &'a dyn DataAccessor<S>,
        alloc: &'a Bump,
    ) -> Self {
        if column_refs.is_empty() {
            // TODO: Currently we have to have non-empty column references to have a non-empty table
            // to evaluate `ProofExpr`s on. Once we restrict [`DataAccessor`] to [`TableExec`]
            // and use input `DynProofPlan`s we should no longer need this.
            let input_length = accessor.get_length(table_ref);
            let bogus_vec = vec![true; input_length];
            let bogus_col = Column::Boolean(alloc.alloc_slice_copy(&bogus_vec));
            Table::<'a, S>::try_from_iter(core::iter::once(("bogus".parse().unwrap(), bogus_col)))
        } else {
            Table::<'a, S>::try_from_iter(column_refs.into_iter().map(|column_ref| {
                let column = accessor.get_column(*column_ref);
                (column_ref.column_id(), column)
            }))
        }
        .expect("Failed to create table from column references")
    }
    /// Number of columns in the table.
    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.table.len()
    }
    /// Number of rows in the table.
    #[must_use]
    pub fn num_rows(&self) -> usize {
        self.num_rows
    }
    /// Whether the table has no columns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
    /// Returns the columns of this table as an `IndexMap`
    #[must_use]
    pub fn into_inner(self) -> IndexMap<Identifier, Column<'a, S>> {
        self.table
    }
    /// Returns the columns of this table as an `IndexMap`
    #[must_use]
    pub fn inner_table(&self) -> &IndexMap<Identifier, Column<'a, S>> {
        &self.table
    }
    /// Returns the columns of this table as an Iterator
    pub fn column_names(&self) -> impl Iterator<Item = &Identifier> {
        self.table.keys()
    }
}

// Note: we modify the default PartialEq for IndexMap to also check for column ordering.
// This is to align with the behaviour of a `RecordBatch`.
impl<S: Scalar> PartialEq for Table<'_, S> {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
            && self
                .table
                .keys()
                .zip(other.table.keys())
                .all(|(a, b)| a == b)
    }
}

#[cfg(test)]
impl<'a, S: Scalar> core::ops::Index<&str> for Table<'a, S> {
    type Output = Column<'a, S>;
    fn index(&self, index: &str) -> &Self::Output {
        self.table
            .get(&index.parse::<Identifier>().unwrap())
            .unwrap()
    }
}
