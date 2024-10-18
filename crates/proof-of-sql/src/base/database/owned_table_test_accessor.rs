use super::{
    Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor, OwnedColumn,
    OwnedTable, SchemaAccessor, TableRef, TestAccessor,
};
use crate::base::{
    commitment::{CommitmentEvaluationProof, VecCommitmentExt},
    map::IndexMap,
};
use bumpalo::Bump;
use proof_of_sql_parser::Identifier;

/// A test accessor that uses [`OwnedTable`] as the underlying table type.
/// Note: this is not optimized for performance, so should not be used for benchmarks.
pub struct OwnedTableTestAccessor<'a, CP: CommitmentEvaluationProof> {
    tables: IndexMap<TableRef, (OwnedTable<CP::Scalar>, usize)>,
    alloc: Bump,
    setup: Option<CP::ProverPublicSetup<'a>>,
}

impl<CP: CommitmentEvaluationProof> Default for OwnedTableTestAccessor<'_, CP> {
    fn default() -> Self {
        Self {
            tables: IndexMap::default(),
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
        OwnedTableTestAccessor::default()
    }

    fn add_table(&mut self, table_ref: TableRef, data: Self::Table, table_offset: usize) {
        self.tables.insert(table_ref, (data, table_offset));
    }
    ///
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`, indicating
    /// that an invalid reference was provided.
    fn get_column_names(&self, table_ref: TableRef) -> Vec<&str> {
        self.tables
            .get(&table_ref)
            .unwrap()
            .0
            .column_names()
            .map(proof_of_sql_parser::Identifier::as_str)
            .collect()
    }

    ///
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`, indicating that an invalid reference was provided.
    fn update_offset(&mut self, table_ref: TableRef, new_offset: usize) {
        self.tables.get_mut(&table_ref).unwrap().1 = new_offset;
    }
}

///
/// # Panics
///
/// Will panic if the `column.table_ref()` is not found in `self.tables`, or if
/// the `column.column_id()` is not found in the inner table for that reference,
/// indicating that an invalid column reference was provided.
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
            OwnedColumn::Boolean(meta,  col) => Column::Boolean(*meta, col),
            OwnedColumn::TinyInt(meta,  col) => Column::TinyInt(*meta, col),
            OwnedColumn::SmallInt(meta,  col) => Column::SmallInt(*meta, col),
            OwnedColumn::Int(meta,  col) => Column::Int(*meta, col),
            OwnedColumn::BigInt(meta,  col) => Column::BigInt(*meta, col),
            OwnedColumn::Int128(meta,  col) => Column::Int128(*meta, col),
            OwnedColumn::Decimal75(meta, precision, scale, col) => {
                Column::Decimal75(*meta, *precision, *scale, col)
            }
            OwnedColumn::Scalar(meta,  col) => Column::Scalar(*meta, col),
            OwnedColumn::VarChar(meta,  col) => {
                let col: &mut [&str] = self
                    .alloc
                    .alloc_slice_fill_iter(col.iter().map(String::as_str));
                let scals: &mut [_] = self
                    .alloc
                    .alloc_slice_fill_iter(col.iter().map(|s| (*s).into()));
                Column::VarChar(*meta, (col, scals))
            }
            OwnedColumn::TimestampTZ(meta, tu, tz, col) => Column::TimestampTZ(*meta, *tu, *tz, col),
        }
    }
}

///
/// # Panics
///
/// Will panic if the `column.table_ref()` is not found in `self.tables`, or if the `column.column_id()` is not found in the inner table for that reference,indicating that an invalid column reference was provided.
impl<CP: CommitmentEvaluationProof> CommitmentAccessor<CP::Commitment>
    for OwnedTableTestAccessor<'_, CP>
{
    fn get_commitment(&self, column: ColumnRef) -> CP::Commitment {
        let (table, offset) = self.tables.get(&column.table_ref()).unwrap();
        let owned_column = table.inner_table().get(&column.column_id()).unwrap();
        Vec::<CP::Commitment>::from_columns_with_offset(
            [owned_column],
            *offset,
            self.setup.as_ref().unwrap(),
        )[0]
        .clone()
    }
}
impl<CP: CommitmentEvaluationProof> MetadataAccessor for OwnedTableTestAccessor<'_, CP> {
    ///
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`, indicating that an invalid reference was provided.
    fn get_length(&self, table_ref: TableRef) -> usize {
        self.tables.get(&table_ref).unwrap().0.num_rows()
    }
    ///
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`, indicating that an invalid reference was provided.
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
    ///
    /// # Panics
    ///
    /// Will panic if the `table_ref` is not found in `self.tables`, indicating that an invalid reference was provided.
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
