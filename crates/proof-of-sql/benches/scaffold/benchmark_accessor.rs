use proof_of_sql::base::{
    commitment::Commitment,
    database::{
        Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
        SchemaAccessor, TableRef,
    },
};
use proof_of_sql_parser::Identifier;
use std::collections::HashMap;

#[derive(Default)]
pub struct BenchmarkAccessor<'a, C: Commitment> {
    columns: HashMap<ColumnRef, Column<'a, C::Scalar>>,
    lengths: HashMap<TableRef, usize>,
    commitments: HashMap<ColumnRef, C>,
    column_types: HashMap<(TableRef, Identifier), ColumnType>,
    table_schemas: HashMap<TableRef, Vec<(Identifier, ColumnType)>>,
}

impl<'a, C: Commitment> BenchmarkAccessor<'a, C> {
    pub fn insert_table(
        &mut self,
        table_ref: TableRef,
        columns: &[(Identifier, Column<'a, C::Scalar>)],
        setup: &C::PublicSetup<'_>,
    ) {
        self.table_schemas.insert(
            table_ref,
            columns
                .iter()
                .map(|(id, col)| (*id, col.column_type()))
                .collect(),
        );

        let mut commitments = vec![C::default(); columns.len()];
        let committable_columns = columns
            .iter()
            .map(|(_, col)| col.into())
            .collect::<Vec<_>>();
        C::compute_commitments(&mut commitments, &committable_columns, 0, setup);

        let mut length = None;
        for (column, commitment) in columns.iter().zip(commitments) {
            self.columns.insert(
                ColumnRef::new(table_ref, column.0, column.1.column_type()),
                column.1.clone(),
            );
            self.commitments.insert(
                ColumnRef::new(table_ref, column.0, column.1.column_type()),
                commitment,
            );
            self.column_types
                .insert((table_ref, column.0), column.1.column_type());

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
    fn get_column(&self, column: ColumnRef) -> Column<C::Scalar> {
        self.columns.get(&column).unwrap().clone()
    }
}
impl<C: Commitment> MetadataAccessor for BenchmarkAccessor<'_, C> {
    fn get_length(&self, table_ref: TableRef) -> usize {
        *self.lengths.get(&table_ref).unwrap()
    }
    fn get_offset(&self, _table_ref: TableRef) -> usize {
        0
    }
}
impl<C: Commitment> CommitmentAccessor<C> for BenchmarkAccessor<'_, C> {
    fn get_commitment(&self, column: ColumnRef) -> C {
        *self.commitments.get(&column).unwrap()
    }
}
impl<C: Commitment> SchemaAccessor for BenchmarkAccessor<'_, C> {
    fn lookup_column(&self, table_ref: TableRef, column_id: Identifier) -> Option<ColumnType> {
        self.column_types.get(&(table_ref, column_id)).copied()
    }
    fn lookup_schema(&self, table_ref: TableRef) -> Vec<(Identifier, ColumnType)> {
        self.table_schemas.get(&table_ref).unwrap().clone()
    }
}
