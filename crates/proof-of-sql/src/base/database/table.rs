use super::{Column, ColumnField, NullableColumn};
use crate::base::{map::IndexMap, scalar::Scalar};
use alloc::vec::Vec;
use bumpalo::Bump;
use snafu::Snafu;
use sqlparser::ast::Ident;

/// Options for creating a table.
/// Inspired by [`RecordBatchOptions`](https://docs.rs/arrow/latest/arrow/record_batch/struct.RecordBatchOptions.html)
#[derive(Debug, Default, Clone, Copy)]
pub struct TableOptions {
    /// The number of rows in the table. Mostly useful for tables without columns.
    pub row_count: Option<usize>,
}

impl TableOptions {
    /// Creates a new [`TableOptions`].
    #[must_use]
    pub fn new(row_count: Option<usize>) -> Self {
        Self { row_count }
    }
}

/// An error that occurs when working with tables.
#[derive(Snafu, Debug, PartialEq, Eq)]
pub enum TableError {
    /// The columns have different lengths.
    #[snafu(display("Columns have different lengths"))]
    ColumnLengthMismatch,

    /// At least one column has length different from the provided row count.
    #[snafu(display("Column has length different from the provided row count"))]
    ColumnLengthMismatchWithSpecifiedRowCount,

    /// The table is empty and there is no specified row count.
    #[snafu(display("Table is empty and no row count is specified"))]
    EmptyTableWithoutSpecifiedRowCount,

    /// The presence slice length doesn't match the table row count.
    #[snafu(display("Presence slice length must match table row count"))]
    PresenceLengthMismatch,
    
    /// The column was not found in the table.
    #[snafu(display("Column '{column}' not found in table"))]
    ColumnNotFound {
        /// The name of the column that was not found
        column: String,
    },
}

type TableSplit<'a, S> = (IndexMap<Ident, Column<'a, S>>, IndexMap<Ident, &'a [bool]>);

/// A table of data, with schema included. This is simply a map from `Ident` to `Column`,
/// where columns order matters.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow [`RecordBatch`](arrow::record_batch::RecordBatch).
#[derive(Debug, Clone, Eq)]
pub struct Table<'a, S: Scalar> {
    table: IndexMap<Ident, Column<'a, S>>,
    row_count: usize,
    // Map to store the presence information for each column
    presence_map: IndexMap<Ident, &'a [bool]>,
}
impl<'a, S: Scalar> Table<'a, S> {
    /// Creates a new [`Table`] with the given columns and default [`TableOptions`].
    pub fn try_new(table: IndexMap<Ident, Column<'a, S>>) -> Result<Self, TableError> {
        Self::try_new_with_options(table, TableOptions::default())
    }

    /// Creates a new [`Table`] with the given columns and with [`TableOptions`].
    pub fn try_new_with_options(
        table: IndexMap<Ident, Column<'a, S>>,
        options: TableOptions,
    ) -> Result<Self, TableError> {
        match (table.is_empty(), options.row_count) {
            (true, None) => Err(TableError::EmptyTableWithoutSpecifiedRowCount),
            (true, Some(row_count)) => Ok(Self {
                table,
                row_count,
                presence_map: IndexMap::default(),
            }),
            (false, None) => {
                let row_count = table[0].len();
                if table.values().any(|column| column.len() != row_count) {
                    Err(TableError::ColumnLengthMismatch)
                } else {
                    Ok(Self {
                        table,
                        row_count,
                        presence_map: IndexMap::default(),
                    })
                }
            }
            (false, Some(row_count)) => {
                if table.values().any(|column| column.len() != row_count) {
                    Err(TableError::ColumnLengthMismatchWithSpecifiedRowCount)
                } else {
                    Ok(Self {
                        table,
                        row_count,
                        presence_map: IndexMap::default(),
                    })
                }
            }
        }
    }

    /// Creates a new [`Table`] from an iterator of `(Ident, Column)` pairs with default [`TableOptions`].
    pub fn try_from_iter<T: IntoIterator<Item = (Ident, Column<'a, S>)>>(
        iter: T,
    ) -> Result<Self, TableError> {
        Self::try_from_iter_with_options(iter, TableOptions::default())
    }

    /// Creates a new [`Table`] from an iterator of `(Ident, Column)` pairs with [`TableOptions`].
    pub fn try_from_iter_with_options<T: IntoIterator<Item = (Ident, Column<'a, S>)>>(
        iter: T,
        options: TableOptions,
    ) -> Result<Self, TableError> {
        Self::try_new_with_options(IndexMap::from_iter(iter), options)
    }

    /// Number of columns in the table.
    #[must_use]
    pub fn num_columns(&self) -> usize {
        self.table.len()
    }
    /// Number of rows in the table.
    #[must_use]
    pub fn num_rows(&self) -> usize {
        self.row_count
    }
    /// Whether the table has no columns.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
    /// Returns the columns of this table as an `IndexMap`
    /// This method maintains backward compatibility with existing code
    #[must_use]
    pub fn into_inner(self) -> IndexMap<Ident, Column<'a, S>> {
        self.table
    }

    /// Returns both the columns and presence information of this table
    #[must_use]
    pub fn into_inner_with_presence(self) -> TableSplit<'a, S> {
        (self.table, self.presence_map)
    }
    /// Returns the columns of this table as an `IndexMap`
    #[must_use]
    pub fn inner_table(&self) -> &IndexMap<Ident, Column<'a, S>> {
        &self.table
    }
    /// Returns the presence map of this table
    #[must_use]
    pub fn presence_map(&self) -> &IndexMap<Ident, &'a [bool]> {
        &self.presence_map
    }
    /// Return the schema of this table as a `Vec` of `ColumnField`s
    #[must_use]
    pub fn schema(&self) -> Vec<ColumnField> {
        self.table
            .iter()
            .map(|(name, column)| ColumnField::new(name.clone(), column.column_type()))
            .collect()
    }
    /// Returns the columns of this table as an Iterator
    pub fn column_names(&self) -> impl Iterator<Item = &Ident> {
        self.table.keys()
    }
    /// Returns the columns of this table as an Iterator
    pub fn columns(&self) -> impl Iterator<Item = &Column<'a, S>> {
        self.table.values()
    }
    /// Returns the column with the given position.
    #[must_use]
    pub fn column(&self, index: usize) -> Option<&Column<'a, S>> {
        self.table.values().nth(index)
    }
    /// Add the `rho` column as the last column to the table.
    /// If a column named "rho" already exists, it will be returned unchanged.
    #[must_use]
    pub fn add_rho_column(mut self, alloc: &'a Bump) -> Self {
        let rho_ident = Ident::new("rho");
        if self.table.contains_key(&rho_ident) {
            return self;
        }

        self.table
            .insert(rho_ident.clone(), Column::rho(self.row_count, alloc));
        // The rho column is always fully present (no NULL values)
        self
    }

    /// Create a new Table with the same schema but with all values present (no NULLs)
    #[must_use]
    pub fn with_all_present(self) -> Self {
        Self {
            table: self.table,
            row_count: self.row_count,
            presence_map: IndexMap::default(),
        }
    }

    /// Merge the presence information from another table
    /// This is useful when creating a new table from another table
    #[must_use]
    pub fn with_presence_from(mut self, other: &Self) -> Self {
        for (ident, presence) in &other.presence_map {
            if self.table.contains_key(ident) {
                self.presence_map.insert(ident.clone(), *presence);
            }
        }
        self
    }

    /// Set the presence slice for a column
    ///
    /// Returns an error if the presence slice length doesn't match the table row count
    pub fn set_column_presence(
        &mut self,
        column_name: &str,
        presence: &'a [bool],
    ) -> Result<(), TableError> {
        let ident = Ident::new(column_name);
        if self.table.contains_key(&ident) {
            if presence.len() != self.row_count {
                return Err(TableError::PresenceLengthMismatch);
            }
            self.presence_map.insert(ident, presence);
        }
        Ok(())
    }

    /// Returns the presence slice for a given expression if it's stored in the table
    /// This is used for checking nullability of expressions in SQL context
    ///
    /// For complex expressions involving multiple nullable columns, this method combines
    /// the presence information from all referenced columns that have NULL values.
    /// A row has "presence" (is non-NULL) in the result only if all component columns
    /// have presence at that row. This respects SQL's handling of NULL values in expressions
    /// where NULL in any component means NULL in the result.
    pub fn presence_for_expr(
        &self,
        expr: &(impl crate::sql::proof_exprs::ProofExpr + 'static),
    ) -> Option<&'a [bool]> {
        use crate::sql::proof_exprs::{ColumnExpr, DynProofExpr};
        use core::any::Any;

        if let Some(column_expr) = (expr as &dyn Any).downcast_ref::<ColumnExpr>() {
            let ident = column_expr.column_id();
            return self.presence_map.get(&ident).copied();
        }

        if let Some(dyn_expr) = (expr as &dyn Any).downcast_ref::<DynProofExpr>() {
            match dyn_expr {
                DynProofExpr::Column(column_expr) => {
                    let ident = column_expr.column_id();
                    return self.presence_map.get(&ident).copied();
                }
                // These expressions always produce non-NULL results regardless of their inputs
                DynProofExpr::IsNull(_)
                | DynProofExpr::IsNotNull(_)
                | DynProofExpr::IsTrue(_)
                | DynProofExpr::Literal(_) => {
                    return None;
                }
                // For all other expressions, we need to check their column references
                _ => {
                    // Pre-allocate capacity for the IndexSet to avoid reallocations in the hot path
                    use crate::base::map::IndexSet;
                    use core::hash::BuildHasherDefault;
                    const INITIAL_COLUMN_CAPACITY: usize = 4;
                    let mut columns = IndexSet::with_capacity_and_hasher(
                        INITIAL_COLUMN_CAPACITY,
                        BuildHasherDefault::default(),
                    );

                    expr.get_column_references(&mut columns);

                    // If we have any column references, check them for nullability
                    if !columns.is_empty() {
                        // Look for columns with NULL values
                        let mut nullable_columns = Vec::new();
                        let mut row_count = 0;
                        
                        for column_ref in &columns {
                            let ident = column_ref.column_id();
                            if let Some(presence) = self.presence_map.get(&ident).copied() {
                                nullable_columns.push(presence);
                                row_count = presence.len();
                            }
                        }
                        
                        // If we found any nullable columns
                        if !nullable_columns.is_empty() {
                            // If there's only one nullable column, just return its presence
                            if nullable_columns.len() == 1 {
                                return Some(nullable_columns[0]);
                            }
                            
                            // Otherwise, create a static &[bool] with combined presence info
                            // First create a boolean array
                            let mut combined = vec![true; row_count];
                            
                            // For each nullable column, update the combined presence
                            for presence in nullable_columns {
                                for (i, &is_present) in presence.iter().enumerate() {
                                    if !is_present {
                                        combined[i] = false;
                                    }
                                }
                            }
                            
                            // Now leak the vector to get a 'static lifetime
                            // This is safe because the Vec is properly aligned and initialized
                            // We're intentionally leaking memory, but it's a small amount and
                            // will be cleaned up when the process exits
                            let leaked_combined: &'static [bool] = Box::leak(combined.into_boxed_slice());
                            
                            // Use a transmutation to convert from &'static [bool] to &'a [bool]
                            // This is safe because 'static outlives 'a
                            let transmuted: &'a [bool] = unsafe { 
                                std::mem::transmute::<&'static [bool], &'a [bool]>(leaked_combined)
                            };
                            
                            return Some(transmuted);
                        }
                    }
                }
            }
        }

        // Default case: all values are present (non-NULL)
        None
    }

    /// Returns a nullable column by name, if it exists in the table.
    ///
    /// This method retrieves a column by name and wraps it in a `NullableColumn` structure
    /// that includes presence information (NULL values). If the column has associated
    /// presence data in the presence map, it will be included in the returned `NullableColumn`.
    ///
    /// # Arguments
    ///
    /// * `column_name` - The name of the column to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(NullableColumn)` if the column exists
    /// * `None` if no column with the given name exists
    #[must_use]
    pub fn nullable_column(&self, column_name: &str) -> Option<NullableColumn<'a, S>> {
        let ident = Ident::new(column_name);

        self.table.get(&ident).map(|column| {
            let presence = self.presence_map.get(&ident).copied();
            NullableColumn::with_presence(*column, presence).unwrap_or_else(|_| {
                // This should never happen as we control the presence data internally
                // and ensure it's the correct length
                NullableColumn::new(*column)
            })
        })
    }

    /// Returns the presence information for a column by name, if it exists.
    ///
    /// The presence information is a boolean slice where `true` indicates a value is present (non-NULL)
    /// and `false` indicates a NULL value. If no presence information exists for the column,
    /// this method returns `None`, which means all values in the column are non-NULL.
    ///
    /// # Arguments
    ///
    /// * `column_name` - The name of the column to retrieve presence information for
    ///
    /// # Returns
    ///
    /// * `Some(&[bool])` - The presence information for the column if it exists
    /// * `None` - If the column doesn't exist or has no NULL values (all values are present)
    #[must_use]
    pub fn column_presence(&self, column_name: &str) -> Option<&'a [bool]> {
        let ident = Ident::new(column_name);
        self.presence_map.get(&ident).copied()
    }

    /// Creates a new [`Table`] with the given columns, presence information, and with [`TableOptions`].
    pub fn try_new_with_presence(
        table: IndexMap<Ident, Column<'a, S>>,
        presence_map: IndexMap<Ident, &'a [bool]>,
        options: TableOptions,
    ) -> Result<Self, TableError> {
        let mut result = Self::try_new_with_options(table, options)?;

        for (ident, presence) in presence_map {
            if result.table.contains_key(&ident) {
                if presence.len() != result.row_count {
                    return Err(TableError::PresenceLengthMismatch);
                }
                result.presence_map.insert(ident, presence);
            }
        }

        Ok(result)
    }
}

// Note: we modify the default PartialEq for IndexMap to also check for column ordering.
// This is to align with the behaviour of a `RecordBatch`.
impl<S: Scalar> PartialEq for Table<'_, S> {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table
            && self.presence_map == other.presence_map
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
        self.table.get(&Ident::new(index)).unwrap()
    }
}
