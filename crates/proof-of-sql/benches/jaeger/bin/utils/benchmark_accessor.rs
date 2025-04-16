use indexmap::IndexMap;
use proof_of_sql::base::{
    commitment::Commitment,
    database::{
        Column, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor,
        TableRef,
    },
};
use sqlparser::ast::Ident;
#[derive(Default)]
pub struct BenchmarkAccessor<'a, C: Commitment> {
    columns: IndexMap<(TableRef, Ident), Column<'a, C::Scalar>>,
    lengths: IndexMap<TableRef, usize>,
    commitments: IndexMap<(TableRef, Ident), C>,
    column_types: IndexMap<(TableRef, Ident), ColumnType>,
    table_schemas: IndexMap<TableRef, Vec<(Ident, ColumnType)>>,
}

impl<'a, C: Commitment> BenchmarkAccessor<'a, C> {
    /// # Panics
    ///
    /// Will panic if the length of the columns does not match after insertion or if the commitment computation fails.
    pub fn insert_table(
        &mut self,
        table_ref: TableRef,
        columns: &[(Ident, Column<'a, C::Scalar>)],
        setup: &C::PublicSetup<'_>,
    ) {
        self.table_schemas.insert(
            table_ref.clone(),
            columns
                .iter()
                .map(|(id, col)| (id.clone(), col.column_type()))
                .collect(),
        );

        let committable_columns = columns
            .iter()
            .map(|(_, col)| col.into())
            .collect::<Vec<_>>();

        let commitments = C::compute_commitments(&committable_columns, 0, setup);

        let mut length = None;
        for (column, commitment) in columns.iter().zip(commitments) {
            self.columns
                .insert((table_ref.clone(), column.0.clone()), column.1);
            self.commitments
                .insert((table_ref.clone(), column.0.clone()), commitment);
            self.column_types.insert(
                (table_ref.clone(), column.0.clone()),
                column.1.column_type(),
            );

            if let Some(len) = length {
                assert!(len == column.1.len());
            } else {
                length = Some(column.1.len());
            }
        }
        self.lengths.insert(table_ref, length.unwrap());
    }
}

impl<C: Commitment> DataAccessor<C::Scalar> for BenchmarkAccessor<'_, C> {
    /// # Panics
    ///
    /// Will panic if the [`TableRef`]-[`Ident`] pair does not exist in the accessor.
    fn get_column(&self, table_ref: &TableRef, column_id: &Ident) -> Column<C::Scalar> {
        *self
            .columns
            .get(&(table_ref.clone(), column_id.clone()))
            .unwrap()
    }
}
impl<C: Commitment> MetadataAccessor for BenchmarkAccessor<'_, C> {
    /// # Panics
    ///
    /// Will panic if the table reference does not exist in the lengths map.
    fn get_length(&self, table_ref: &TableRef) -> usize {
        *self.lengths.get(&table_ref).unwrap()
    }
    fn get_offset(&self, _table_ref: &TableRef) -> usize {
        0
    }
}
impl<C: Commitment> CommitmentAccessor<C> for BenchmarkAccessor<'_, C> {
    /// # Panics
    ///
    /// Will panic if the [`TableRef`]-[`Ident`] pair does not exist in the commitments map.
    fn get_commitment(&self, table_ref: &TableRef, column_id: &Ident) -> C {
        self.commitments
            .get(&(table_ref.clone(), column_id.clone()))
            .unwrap()
            .clone()
    }
}
impl<C: Commitment> SchemaAccessor for BenchmarkAccessor<'_, C> {
    fn lookup_column(&self, table_ref: &TableRef, column_id: &Ident) -> Option<ColumnType> {
        self.column_types
            .get(&(table_ref.clone(), column_id.clone()))
            .copied()
    }
    /// # Panics
    ///
    /// Will panic if the table reference does not exist in the table schemas map.
    fn lookup_schema(&self, table_ref: &TableRef) -> Vec<(Ident, ColumnType)> {
        self.table_schemas.get(&table_ref).unwrap().clone()
    }
}
