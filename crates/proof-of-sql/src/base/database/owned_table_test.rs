use crate::{
    base::{
        database::{owned_table_utility::*, OwnedColumn, OwnedTable, OwnedTableError},
        map::IndexMap,
        scalar::test_scalar::TestScalar,
    },
    proof_primitive::dory::DoryScalar,
};
use proof_of_sql_parser::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    Identifier,
};

#[test]
fn we_can_create_an_owned_table_with_no_columns() {
    let table = OwnedTable::<TestScalar>::try_new(IndexMap::default()).unwrap();
    assert_eq!(table.num_columns(), 0);
}
#[test]
fn we_can_create_an_empty_owned_table() {
    let owned_table = owned_table::<DoryScalar>([
        bigint("bigint", [0; 0]),
        int128("decimal", [0; 0]),
        varchar("varchar", ["0"; 0]),
        scalar("scalar", [0; 0]),
        boolean("boolean", [true; 0]),
    ]);
    let mut table = IndexMap::default();
    table.insert(
        Identifier::try_new("bigint").unwrap(),
        OwnedColumn::BigInt(vec![]),
    );
    table.insert(
        Identifier::try_new("decimal").unwrap(),
        OwnedColumn::Int128(vec![]),
    );
    table.insert(
        Identifier::try_new("varchar").unwrap(),
        OwnedColumn::VarChar(vec![]),
    );
    table.insert(
        Identifier::try_new("scalar").unwrap(),
        OwnedColumn::Scalar(vec![]),
    );
    table.insert(
        Identifier::try_new("boolean").unwrap(),
        OwnedColumn::Boolean(vec![]),
    );
    assert_eq!(owned_table.into_inner(), table);
}
#[test]
fn we_can_create_an_owned_table_with_data() {
    let owned_table = owned_table([
        bigint("bigint", [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
        int128("decimal", [0, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
        varchar("varchar", ["0", "1", "2", "3", "4", "5", "6", "7", "8"]),
        scalar("scalar", [0, 1, 2, 3, 4, 5, 6, 7, 8]),
        boolean(
            "boolean",
            [true, false, true, false, true, false, true, false, true],
        ),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
        ),
    ]);
    let mut table = IndexMap::default();
    table.insert(
        Identifier::try_new("time_stamp").unwrap(),
        OwnedColumn::TimestampTZ(
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [0, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX].into(),
        ),
    );
    table.insert(
        Identifier::try_new("bigint").unwrap(),
        OwnedColumn::BigInt(vec![0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]),
    );
    table.insert(
        Identifier::try_new("decimal").unwrap(),
        OwnedColumn::Int128(vec![0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]),
    );
    table.insert(
        Identifier::try_new("varchar").unwrap(),
        OwnedColumn::VarChar(vec![
            "0".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "4".to_string(),
            "5".to_string(),
            "6".to_string(),
            "7".to_string(),
            "8".to_string(),
        ]),
    );
    table.insert(
        Identifier::try_new("scalar").unwrap(),
        OwnedColumn::Scalar(vec![
            DoryScalar::from(0),
            1.into(),
            2.into(),
            3.into(),
            4.into(),
            5.into(),
            6.into(),
            7.into(),
            8.into(),
        ]),
    );
    table.insert(
        Identifier::try_new("boolean").unwrap(),
        OwnedColumn::Boolean(vec![
            true, false, true, false, true, false, true, false, true,
        ]),
    );
    assert_eq!(owned_table.into_inner(), table);
}
#[test]
fn we_get_inequality_between_tables_with_differing_column_order() {
    let owned_table_a: OwnedTable<TestScalar> = owned_table([
        bigint("a", [0; 0]),
        int128("b", [0; 0]),
        varchar("c", ["0"; 0]),
        boolean("d", [false; 0]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [0; 0],
        ),
    ]);
    let owned_table_b: OwnedTable<TestScalar> = owned_table([
        boolean("d", [false; 0]),
        int128("b", [0; 0]),
        bigint("a", [0; 0]),
        varchar("c", ["0"; 0]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [0; 0],
        ),
    ]);
    assert_ne!(owned_table_a, owned_table_b);
}
#[test]
fn we_get_inequality_between_tables_with_differing_data() {
    let owned_table_a: OwnedTable<DoryScalar> = owned_table([
        bigint("a", [0]),
        int128("b", [0]),
        varchar("c", ["0"]),
        boolean("d", [true]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [1625072400],
        ),
    ]);
    let owned_table_b: OwnedTable<DoryScalar> = owned_table([
        bigint("a", [1]),
        int128("b", [0]),
        varchar("c", ["0"]),
        boolean("d", [true]),
        timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [1625076000],
        ),
    ]);
    assert_ne!(owned_table_a, owned_table_b);
}
#[test]
fn we_cannot_create_an_owned_table_with_differing_column_lengths() {
    assert!(matches!(
        OwnedTable::<TestScalar>::try_from_iter([
            ("a".parse().unwrap(), OwnedColumn::BigInt(vec![0])),
            ("b".parse().unwrap(), OwnedColumn::BigInt(vec![])),
        ]),
        Err(OwnedTableError::ColumnLengthMismatch)
    ));
}
