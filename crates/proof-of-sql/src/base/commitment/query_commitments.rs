use super::{Commitment, TableCommitment};
use crate::base::{
    database::{
        ColumnField, ColumnRef, ColumnType, CommitmentAccessor, MetadataAccessor, SchemaAccessor,
        TableRef,
    },
    map::IndexMap,
};
use alloc::vec::Vec;
use sqlparser::ast::Ident;

/// The commitments for all of the tables in a query.
///
/// Simply maps table refs to table commitments, and implements the following traits...
/// - [`MetadataAccessor`]
/// - [`CommitmentAccessor`]
/// - [`SchemaAccessor`]
pub type QueryCommitments<C> = IndexMap<TableRef, TableCommitment<C>>;

/// A trait for extending the functionality of the [`QueryCommitments`] alias.
pub trait QueryCommitmentsExt<C>
where
    C: Commitment,
{
    /// Create a new `QueryCommitments` from a collection of columns and an accessor.
    fn from_accessor_with_max_bounds(
        columns: impl IntoIterator<Item = ColumnRef>,
        accessor: &(impl CommitmentAccessor<C> + SchemaAccessor),
    ) -> Self;
}

impl<C: Commitment> QueryCommitmentsExt<C> for QueryCommitments<C> {
    fn from_accessor_with_max_bounds(
        columns: impl IntoIterator<Item = ColumnRef>,
        accessor: &(impl CommitmentAccessor<C> + SchemaAccessor),
    ) -> Self {
        columns
            .into_iter()
            .fold(
                IndexMap::<_, Vec<_>>::default(),
                |mut table_columns, column| {
                    table_columns
                        .entry(column.table_ref())
                        .or_default()
                        .push(ColumnField::new(column.column_id(), *column.column_type()));
                    table_columns
                },
            )
            .into_iter()
            .map(|(table_ref, column_fields)| {
                let selected_column_fields = accessor
                    .lookup_schema(table_ref.clone())
                    .into_iter()
                    .filter_map(|(ident, _)| {
                        column_fields
                            .iter()
                            .find(|column_field| column_field.name() == ident)
                            .cloned()
                    })
                    .collect::<Vec<_>>();
                let table_commitment = TableCommitment::from_accessor_with_max_bounds(
                    &table_ref,
                    &selected_column_fields,
                    accessor,
                );
                (table_ref, table_commitment)
            })
            .collect()
    }
}

impl<C: Commitment> MetadataAccessor for QueryCommitments<C> {
    fn get_length(&self, table_ref: &TableRef) -> usize {
        let table_commitment = self.get(&table_ref).unwrap();
        table_commitment.num_rows()
    }

    fn get_offset(&self, table_ref: &TableRef) -> usize {
        let table_commitment = self.get(&table_ref).unwrap();
        table_commitment.range().start
    }
}

/// # Panics
///
/// Panics if the commitment for the table or column cannot be found.
impl<C: Commitment> CommitmentAccessor<C> for QueryCommitments<C> {
    fn get_commitment(&self, column: ColumnRef) -> C {
        let table_commitment = self.get(&column.table_ref()).unwrap();

        table_commitment
            .column_commitments()
            .get_commitment(&column.column_id())
            .unwrap()
    }
}

impl<C: Commitment> SchemaAccessor for QueryCommitments<C> {
    fn lookup_column(
        &self,
        table_ref: crate::base::database::TableRef,
        column_id: Ident,
    ) -> Option<ColumnType> {
        let table_commitment = self.get(&table_ref)?;

        table_commitment
            .column_commitments()
            .get_metadata(&column_id)
            .map(|column_metadata| *column_metadata.column_type())
    }

    /// # Panics
    ///
    /// Panics if the column metadata cannot be found.
    fn lookup_schema(
        &self,
        table_ref: crate::base::database::TableRef,
    ) -> Vec<(Ident, ColumnType)> {
        let table_commitment = self.get(&table_ref).unwrap();

        table_commitment
            .column_commitments()
            .column_metadata()
            .iter()
            .map(|(identifier, column_metadata)| {
                (identifier.clone(), *column_metadata.column_type())
            })
            .collect()
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::{
        base::{
            commitment::{naive_commitment::NaiveCommitment, Bounds, ColumnBounds},
            database::{
                owned_table_utility::*, OwnedColumn, OwnedTable, OwnedTableTestAccessor,
                TestAccessor,
            },
            scalar::test_scalar::TestScalar,
        },
        proof_primitive::dory::{
            test_rng, DoryCommitment, DoryEvaluationProof, DoryProverPublicSetup, ProverSetup,
            PublicParameters,
        },
    };

    #[test]
    fn we_can_get_length_and_offset_of_tables() {
        let table_a: OwnedTable<TestScalar> = owned_table([
            bigint("column_a", [1, 2, 3, 4]),
            varchar("column_b", ["Lorem", "ipsum", "dolor", "sit"]),
        ]);

        let table_b: OwnedTable<TestScalar> = owned_table([scalar("column_c", [1, 2])]);

        let offset_commitment =
            TableCommitment::<NaiveCommitment>::from_owned_table_with_offset(&table_a, 2, &());
        let offset_table_id = TableRef::new("off", "table");

        let no_offset_commitment = TableCommitment::from_owned_table_with_offset(&table_b, 0, &());
        let no_offset_id = TableRef::new("no", "off");

        let no_columns_commitment = TableCommitment::try_from_columns_with_offset(
            Vec::<(&Ident, &OwnedColumn<TestScalar>)>::new(),
            0,
            &(),
        )
        .unwrap();
        let no_columns_id = TableRef::new("no", "columns");

        let no_rows_commitment = TableCommitment::try_from_columns_with_offset(
            [(
                &"column_c".into(),
                &OwnedColumn::<TestScalar>::BigInt(vec![]),
            )],
            3,
            &(),
        )
        .unwrap();
        let no_rows_id = TableRef::new("no", "rows");

        let query_commitments = QueryCommitments::from_iter([
            (offset_table_id.clone(), offset_commitment),
            (no_offset_id.clone(), no_offset_commitment),
            (no_columns_id.clone(), no_columns_commitment),
            (no_rows_id.clone(), no_rows_commitment),
        ]);

        assert_eq!(query_commitments.get_offset(&offset_table_id), 2);
        assert_eq!(query_commitments.get_length(&offset_table_id), 4);

        assert_eq!(query_commitments.get_offset(&no_offset_id), 0);
        assert_eq!(query_commitments.get_length(&no_offset_id), 2);

        assert_eq!(query_commitments.get_offset(&no_columns_id), 0);
        assert_eq!(query_commitments.get_length(&no_columns_id), 0);

        assert_eq!(query_commitments.get_offset(&no_rows_id), 3);
        assert_eq!(query_commitments.get_length(&no_rows_id), 0);
    }

    #[expect(clippy::similar_names)]
    #[test]
    fn we_can_get_commitment_of_a_column() {
        let column_a_id: Ident = "column_a".into();
        let column_b_id: Ident = "column_b".into();

        let table_a: OwnedTable<TestScalar> = owned_table([
            bigint(column_a_id.value.as_str(), [1, 2, 3, 4]),
            varchar(
                column_b_id.value.as_str(),
                ["Lorem", "ipsum", "dolor", "sit"],
            ),
        ]);
        let table_b: OwnedTable<TestScalar> =
            owned_table([scalar(column_a_id.value.as_str(), [1, 2])]);

        let table_a_commitment =
            TableCommitment::<NaiveCommitment>::from_owned_table_with_offset(&table_a, 2, &());
        let table_a_id = TableRef::new("table", "a");

        let table_b_commitment = TableCommitment::from_owned_table_with_offset(&table_b, 0, &());
        let table_b_id = TableRef::new("table", "b");

        let query_commitments = QueryCommitments::from_iter([
            (table_a_id.clone(), table_a_commitment.clone()),
            (table_b_id.clone(), table_b_commitment.clone()),
        ]);

        assert_eq!(
            query_commitments.get_commitment(ColumnRef::new(
                table_a_id.clone(),
                column_a_id.clone(),
                ColumnType::BigInt
            )),
            table_a_commitment.column_commitments().commitments()[0]
        );
        assert_eq!(
            query_commitments.get_commitment(ColumnRef::new(
                table_a_id,
                column_b_id,
                ColumnType::VarChar
            )),
            table_a_commitment.column_commitments().commitments()[1]
        );
        assert_eq!(
            query_commitments.get_commitment(ColumnRef::new(
                table_b_id,
                column_a_id,
                ColumnType::Scalar
            )),
            table_b_commitment.column_commitments().commitments()[0]
        );
    }

    #[expect(clippy::similar_names)]
    #[test]
    fn we_can_get_schema_of_tables() {
        let column_a_id: Ident = "column_a".into();
        let column_b_id: Ident = "column_b".into();

        let table_a: OwnedTable<TestScalar> = owned_table([
            bigint(column_a_id.value.as_str(), [1, 2, 3, 4]),
            varchar(
                column_b_id.value.as_str(),
                ["Lorem", "ipsum", "dolor", "sit"],
            ),
        ]);
        let table_b: OwnedTable<TestScalar> =
            owned_table([scalar(column_a_id.value.as_str(), [1, 2])]);

        let table_a_commitment =
            TableCommitment::<NaiveCommitment>::from_owned_table_with_offset(&table_a, 2, &());
        let table_a_id = TableRef::new("table", "a");

        let table_b_commitment = TableCommitment::from_owned_table_with_offset(&table_b, 0, &());
        let table_b_id = TableRef::new("table", "b");

        let no_columns_commitment = TableCommitment::try_from_columns_with_offset(
            Vec::<(&Ident, &OwnedColumn<TestScalar>)>::new(),
            0,
            &(),
        )
        .unwrap();
        let no_columns_id = TableRef::new("no", "columns");

        let query_commitments = QueryCommitments::from_iter([
            (table_a_id.clone(), table_a_commitment),
            (table_b_id.clone(), table_b_commitment),
            (no_columns_id.clone(), no_columns_commitment),
        ]);

        assert_eq!(
            query_commitments
                .lookup_column(table_a_id.clone(), column_a_id.clone())
                .unwrap(),
            ColumnType::BigInt
        );
        assert_eq!(
            query_commitments
                .lookup_column(table_a_id.clone(), column_b_id.clone())
                .unwrap(),
            ColumnType::VarChar
        );
        assert_eq!(
            query_commitments.lookup_schema(table_a_id),
            vec![
                (column_a_id.clone(), ColumnType::BigInt),
                (column_b_id.clone(), ColumnType::VarChar)
            ]
        );

        assert_eq!(
            query_commitments
                .lookup_column(table_b_id.clone(), column_a_id.clone())
                .unwrap(),
            ColumnType::Scalar
        );
        assert_eq!(
            query_commitments.lookup_column(table_b_id.clone(), column_b_id),
            None
        );
        assert_eq!(
            query_commitments.lookup_schema(table_b_id),
            vec![(column_a_id.clone(), ColumnType::Scalar),]
        );

        assert_eq!(
            query_commitments.lookup_column(no_columns_id.clone(), column_a_id),
            None
        );
        assert_eq!(query_commitments.lookup_schema(no_columns_id), vec![]);
    }

    #[expect(clippy::similar_names)]
    #[test]
    fn we_can_get_query_commitments_from_accessor() {
        let public_parameters = PublicParameters::test_rand(4, &mut test_rng());
        let prover_setup = ProverSetup::from(&public_parameters);
        let setup = DoryProverPublicSetup::new(&prover_setup, 3);

        let column_a_id: Ident = "column_a".into();
        let column_b_id: Ident = "column_b".into();

        let table_a = owned_table([
            bigint(column_a_id.value.as_str(), [1, 2, 3, 4]),
            varchar(
                column_b_id.value.as_str(),
                ["Lorem", "ipsum", "dolor", "sit"],
            ),
        ]);
        let table_b = owned_table([
            scalar(column_a_id.value.as_str(), [1, 2]),
            int128(column_b_id.value.as_str(), [1, 2]),
        ]);

        let mut table_a_commitment =
            TableCommitment::from_owned_table_with_offset(&table_a, 0, &setup);
        let table_a_id = TableRef::new("table", "a");
        *table_a_commitment
            .column_commitments_mut()
            .column_metadata_mut()
            .get_mut(&column_a_id)
            .unwrap()
            .bounds_mut() = ColumnBounds::BigInt(Bounds::bounded(i64::MIN, i64::MAX).unwrap());

        let mut table_b_commitment =
            TableCommitment::from_owned_table_with_offset(&table_b, 0, &setup);
        let table_b_id = TableRef::new("table", "b");
        *table_b_commitment
            .column_commitments_mut()
            .column_metadata_mut()
            .get_mut(&column_b_id)
            .unwrap()
            .bounds_mut() = ColumnBounds::Int128(Bounds::bounded(i128::MIN, i128::MAX).unwrap());

        let expected_query_commitments = QueryCommitments::from_iter([
            (table_a_id.clone(), table_a_commitment.clone()),
            (table_b_id.clone(), table_b_commitment.clone()),
        ]);

        let mut accessor =
            OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(setup);
        accessor.add_table(table_a_id.clone(), table_a, 0);
        accessor.add_table(table_b_id.clone(), table_b, 0);

        let query_commitments = QueryCommitments::<DoryCommitment>::from_accessor_with_max_bounds(
            [
                ColumnRef::new(table_a_id.clone(), column_a_id.clone(), ColumnType::BigInt),
                ColumnRef::new(table_b_id.clone(), column_a_id, ColumnType::Scalar),
                ColumnRef::new(table_a_id, column_b_id.clone(), ColumnType::VarChar),
                ColumnRef::new(table_b_id, column_b_id, ColumnType::Int128),
            ],
            &accessor,
        );
        assert_eq!(query_commitments, expected_query_commitments);
    }
}
