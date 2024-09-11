use super::{
    Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor, OwnedColumn,
    OwnedTable, SchemaAccessor, TableRef, TestAccessor,
};
use crate::base::commitment::{CommitmentEvaluationProof, VecCommitmentExt};
use bumpalo::Bump;
use indexmap::IndexMap;
use proof_of_sql_parser::Identifier;

/// A test accessor that uses OwnedTable as the underlying table type.
/// Note: this is not optimized for performance, so should not be used for benchmarks.
pub struct OwnedTableTestAccessor<'a, CP: CommitmentEvaluationProof> {
    tables: IndexMap<TableRef, (OwnedTable<CP::Scalar>, usize)>,
    alloc: Bump,
    setup: Option<CP::ProverPublicSetup<'a>>,
}

impl<CP: CommitmentEvaluationProof> Default for OwnedTableTestAccessor<'_, CP> {
    fn default() -> Self {
        Self {
            tables: Default::default(),
            alloc: Bump::new(),
            setup: None,
        }
    }
}

impl<CP: CommitmentEvaluationProof> Clone for OwnedTableTestAccessor<'_, CP> {
    fn clone(&self) -> Self {
        Self {
            tables: self.tables.clone(),
            setup: self.setup,
            ..Default::default()
        }
    }
}

impl<CP: CommitmentEvaluationProof> TestAccessor<CP::Commitment>
    for OwnedTableTestAccessor<'_, CP>
{
    type Table = OwnedTable<CP::Scalar>;

    fn new_empty() -> Self {
        Default::default()
    }

    fn add_table(&mut self, table_ref: TableRef, data: Self::Table, table_offset: usize) {
        self.tables.insert(table_ref, (data, table_offset));
    }

    fn get_column_names(&self, table_ref: TableRef) -> Vec<&str> {
        self.tables
            .get(&table_ref)
            .unwrap()
            .0
            .column_names()
            .map(|id| id.as_str())
            .collect()
    }

    fn update_offset(&mut self, table_ref: TableRef, new_offset: usize) {
        self.tables.get_mut(&table_ref).unwrap().1 = new_offset;
    }
}
impl<CP: CommitmentEvaluationProof> DataAccessor<CP::Scalar> for OwnedTableTestAccessor<'_, CP> {
    fn get_column(&self, column: ColumnRef) -> Column<CP::Scalar> {
        match self
            .tables
            .get(&column.table_ref())
            .unwrap()
            .0
            .inner_table()
            .get(&column.column_id())
            .unwrap()
        {
            OwnedColumn::Boolean(col) => Column::Boolean(col),
            OwnedColumn::SmallInt(col) => Column::SmallInt(col),
            OwnedColumn::Int(col) => Column::Int(col),
            OwnedColumn::BigInt(col) => Column::BigInt(col),
            OwnedColumn::Int128(col) => Column::Int128(col),
            OwnedColumn::Decimal75(precision, scale, col) => {
                Column::Decimal75(*precision, *scale, col)
            }
            OwnedColumn::Scalar(col) => Column::Scalar(col),
            OwnedColumn::VarChar(col) => {
                let col: &mut [&str] = self
                    .alloc
                    .alloc_slice_fill_iter(col.iter().map(|s| s.as_str()));
                let scals: &mut [_] = self
                    .alloc
                    .alloc_slice_fill_iter(col.iter().map(|s| (*s).into()));
                Column::VarChar((col, scals))
            }
            OwnedColumn::TimestampTZ(tu, tz, col) => Column::TimestampTZ(*tu, *tz, col),
        }
    }
}
impl<CP: CommitmentEvaluationProof> CommitmentAccessor<CP::Commitment>
    for OwnedTableTestAccessor<'_, CP>
{
    fn get_commitment(&self, column: ColumnRef) -> CP::Commitment {
        let (table, offset) = self.tables.get(&column.table_ref()).unwrap();
        let owned_column = table.inner_table().get(&column.column_id()).unwrap();
        Vec::<CP::Commitment>::from_columns_with_offset([owned_column], *offset, self.setup.as_ref().unwrap())[0].clone()
    }
}
impl<CP: CommitmentEvaluationProof> MetadataAccessor for OwnedTableTestAccessor<'_, CP> {
    fn get_length(&self, table_ref: TableRef) -> usize {
        self.tables.get(&table_ref).unwrap().0.num_rows()
    }

    fn get_offset(&self, table_ref: TableRef) -> usize {
        self.tables.get(&table_ref).unwrap().1
    }
}
impl<CP: CommitmentEvaluationProof> SchemaAccessor for OwnedTableTestAccessor<'_, CP> {
    fn lookup_column(&self, table_ref: TableRef, column_id: Identifier) -> Option<ColumnType> {
        Some(
            self.tables
                .get(&table_ref)?
                .0
                .inner_table()
                .get(&column_id)?
                .column_type(),
        )
    }

    fn lookup_schema(&self, table_ref: TableRef) -> Vec<(Identifier, ColumnType)> {
        self.tables
            .get(&table_ref)
            .unwrap()
            .0
            .inner_table()
            .iter()
            .map(|(&id, col)| (id, col.column_type()))
            .collect()
    }
}

impl<'a, CP: CommitmentEvaluationProof> OwnedTableTestAccessor<'a, CP> {
    /// Create a new empty test accessor with the given setup.
    pub fn new_empty_with_setup(setup: CP::ProverPublicSetup<'a>) -> Self {
        let mut res = Self::new_empty();
        res.setup = Some(setup);
        res
    }

    /// Create a new test accessor containing the provided table.
    pub fn new_from_table(
        table_ref: TableRef,
        owned_table: OwnedTable<CP::Scalar>,
        offset: usize,
        setup: CP::ProverPublicSetup<'a>,
    ) -> Self {
        let mut res = Self::new_empty_with_setup(setup);
        res.add_table(table_ref, owned_table, offset);
        res
    }
}
