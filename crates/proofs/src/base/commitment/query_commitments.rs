use super::TableCommitment;
use crate::base::database::{
    ColumnRef, ColumnType, CommitmentAccessor, MetadataAccessor, SchemaAccessor, TableRef,
};
use curve25519_dalek::ristretto::RistrettoPoint;
use proofs_sql::Identifier;
use std::collections::HashMap;

/// The commitments for all of the tables in a query.
///
/// Simply maps table refs to table commitments, and implements the following traits...
/// - [`MetadataAccessor`]
/// - [`CommitmentAccessor`]
/// - [`SchemaAccessor`]
pub type QueryCommitments = HashMap<TableRef, TableCommitment>;

impl MetadataAccessor for QueryCommitments {
    fn get_length(&self, table_ref: crate::base::database::TableRef) -> usize {
        let table_commitment = self.get(&table_ref).unwrap();

        table_commitment.num_rows()
    }

    fn get_offset(&self, table_ref: crate::base::database::TableRef) -> usize {
        let table_commitment = self.get(&table_ref).unwrap();

        table_commitment.range().start
    }
}

impl CommitmentAccessor<RistrettoPoint> for QueryCommitments {
    fn get_commitment(&self, column: ColumnRef) -> curve25519_dalek::ristretto::RistrettoPoint {
        let table_commitment = self.get(&column.table_ref()).unwrap();

        table_commitment
            .column_commitments()
            .get_commitment(&column.column_id())
            .unwrap()
            .decompress()
            .unwrap()
    }
}

impl SchemaAccessor for QueryCommitments {
    fn lookup_column(
        &self,
        table_ref: crate::base::database::TableRef,
        column_id: Identifier,
    ) -> Option<ColumnType> {
        let table_commitment = self.get(&table_ref)?;

        table_commitment
            .column_commitments()
            .get_metadata(&column_id)
            .map(|column_metadata| *column_metadata.column_type())
    }

    fn lookup_schema(
        &self,
        table_ref: crate::base::database::TableRef,
    ) -> Vec<(Identifier, ColumnType)> {
        let table_commitment = self.get(&table_ref).unwrap();

        table_commitment
            .column_commitments()
            .column_metadata()
            .iter()
            .map(|(identifier, column_metadata)| (*identifier, *column_metadata.column_type()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{OwnedColumn, OwnedTable},
            scalar::ArkScalar,
        },
        owned_table,
    };

    #[test]
    fn we_can_get_length_and_offset_of_tables() {
        let table_a: OwnedTable<ArkScalar> = owned_table!(
            "column_a" => [1i64, 2, 3, 4],
            "column_b" => ["Lorem", "ipsum", "dolor", "sit"]
        );

        let table_b: OwnedTable<ArkScalar> = owned_table!(
            "column_c" => [1, 2].map(ArkScalar::from)
        );

        let offset_commitment = TableCommitment::from_owned_table_with_offset(&table_a, 2);
        let offset_table_id = "off.table".parse().unwrap();

        let no_offset_commitment = TableCommitment::from_owned_table_with_offset(&table_b, 0);
        let no_offset_id = "no.off".parse().unwrap();

        let no_columns_commitment = TableCommitment::try_from_columns_with_offset(
            Vec::<(&Identifier, &OwnedColumn<ArkScalar>)>::new(),
            0,
        )
        .unwrap();
        let no_columns_id = "no.columns".parse().unwrap();

        let no_rows_commitment = TableCommitment::try_from_columns_with_offset(
            [(
                &"column_c".parse().unwrap(),
                &OwnedColumn::<ArkScalar>::BigInt(vec![]),
            )],
            3,
        )
        .unwrap();
        let no_rows_id = "no.rows".parse().unwrap();

        let query_commitments = QueryCommitments::from_iter([
            (offset_table_id, offset_commitment),
            (no_offset_id, no_offset_commitment),
            (no_columns_id, no_columns_commitment),
            (no_rows_id, no_rows_commitment),
        ]);

        assert_eq!(query_commitments.get_offset(offset_table_id), 2);
        assert_eq!(query_commitments.get_length(offset_table_id), 4);

        assert_eq!(query_commitments.get_offset(no_offset_id), 0);
        assert_eq!(query_commitments.get_length(no_offset_id), 2);

        assert_eq!(query_commitments.get_offset(no_columns_id), 0);
        assert_eq!(query_commitments.get_length(no_columns_id), 0);

        assert_eq!(query_commitments.get_offset(no_rows_id), 3);
        assert_eq!(query_commitments.get_length(no_rows_id), 0);
    }

    #[test]
    fn we_can_get_commitment_of_a_column() {
        let column_a_id: Identifier = "column_a".parse().unwrap();
        let column_b_id: Identifier = "column_b".parse().unwrap();

        let table_a: OwnedTable<ArkScalar> = owned_table!(
            column_a_id => [1i64, 2, 3, 4],
            column_b_id => ["Lorem", "ipsum", "dolor", "sit"]
        );
        let table_b: OwnedTable<ArkScalar> = owned_table!(
            column_a_id => [1, 2].map(ArkScalar::from)
        );

        let table_a_commitment = TableCommitment::from_owned_table_with_offset(&table_a, 2);
        let table_a_id = "table.a".parse().unwrap();

        let table_b_commitment = TableCommitment::from_owned_table_with_offset(&table_b, 0);
        let table_b_id = "table.b".parse().unwrap();

        let query_commitments = QueryCommitments::from_iter([
            (table_a_id, table_a_commitment.clone()),
            (table_b_id, table_b_commitment.clone()),
        ]);

        assert_eq!(
            query_commitments.get_commitment(ColumnRef::new(
                table_a_id,
                column_a_id,
                ColumnType::BigInt
            )),
            table_a_commitment.column_commitments().commitments()[0]
                .decompress()
                .unwrap()
        );
        assert_eq!(
            query_commitments.get_commitment(ColumnRef::new(
                table_a_id,
                column_b_id,
                ColumnType::VarChar
            )),
            table_a_commitment.column_commitments().commitments()[1]
                .decompress()
                .unwrap()
        );
        assert_eq!(
            query_commitments.get_commitment(ColumnRef::new(
                table_b_id,
                column_a_id,
                ColumnType::Scalar
            )),
            table_b_commitment.column_commitments().commitments()[0]
                .decompress()
                .unwrap()
        );
    }

    #[test]
    fn we_can_get_schema_of_tables() {
        let column_a_id: Identifier = "column_a".parse().unwrap();
        let column_b_id: Identifier = "column_b".parse().unwrap();

        let table_a: OwnedTable<ArkScalar> = owned_table!(
            column_a_id => [1i64, 2, 3, 4],
            column_b_id => ["Lorem", "ipsum", "dolor", "sit"]
        );
        let table_b: OwnedTable<ArkScalar> = owned_table!(
            column_a_id => [1, 2].map(ArkScalar::from)
        );

        let table_a_commitment = TableCommitment::from_owned_table_with_offset(&table_a, 2);
        let table_a_id = "table.a".parse().unwrap();

        let table_b_commitment = TableCommitment::from_owned_table_with_offset(&table_b, 0);
        let table_b_id = "table.b".parse().unwrap();

        let no_columns_commitment = TableCommitment::try_from_columns_with_offset(
            Vec::<(&Identifier, &OwnedColumn<ArkScalar>)>::new(),
            0,
        )
        .unwrap();
        let no_columns_id = "no.columns".parse().unwrap();

        let query_commitments = QueryCommitments::from_iter([
            (table_a_id, table_a_commitment),
            (table_b_id, table_b_commitment),
            (no_columns_id, no_columns_commitment),
        ]);

        assert_eq!(
            query_commitments
                .lookup_column(table_a_id, column_a_id)
                .unwrap(),
            ColumnType::BigInt
        );
        assert_eq!(
            query_commitments
                .lookup_column(table_a_id, column_b_id)
                .unwrap(),
            ColumnType::VarChar
        );
        assert_eq!(
            query_commitments.lookup_schema(table_a_id),
            vec![
                (column_a_id, ColumnType::BigInt),
                (column_b_id, ColumnType::VarChar)
            ]
        );

        assert_eq!(
            query_commitments
                .lookup_column(table_b_id, column_a_id)
                .unwrap(),
            ColumnType::Scalar
        );
        assert_eq!(
            query_commitments.lookup_column(table_b_id, column_b_id),
            None
        );
        assert_eq!(
            query_commitments.lookup_schema(table_b_id),
            vec![(column_a_id, ColumnType::Scalar),]
        );

        assert_eq!(
            query_commitments.lookup_column(no_columns_id, column_a_id),
            None
        );
        assert_eq!(query_commitments.lookup_schema(no_columns_id), vec![]);
    }
}
