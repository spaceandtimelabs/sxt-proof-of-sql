use crate::base::{
    database::{table_utility::*, Column, Table, TableError},
    map::IndexMap,
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;
use proof_of_sql_parser::{
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    Identifier,
};

#[test]
fn we_can_create_a_table_with_no_columns() {
    let table = Table::<TestScalar>::try_new(IndexMap::default()).unwrap();
    assert_eq!(table.num_columns(), 0);
    assert_eq!(table.num_rows(), 1);
}
#[test]
fn we_can_create_an_empty_table() {
    let alloc = Bump::new();
    let borrowed_table = table::<TestScalar>([
        borrowed_bigint("bigint", [0; 0], &alloc),
        borrowed_int128("decimal", [0; 0], &alloc),
        borrowed_varchar("varchar", ["0"; 0], &alloc),
        borrowed_scalar("scalar", [0; 0], &alloc),
        borrowed_boolean("boolean", [true; 0], &alloc),
    ]);
    let mut table = IndexMap::default();
    table.insert(Identifier::try_new("bigint").unwrap(), Column::BigInt(&[]));
    table.insert(Identifier::try_new("decimal").unwrap(), Column::Int128(&[]));
    table.insert(
        Identifier::try_new("varchar").unwrap(),
        Column::VarChar((&[], &[])),
    );
    table.insert(Identifier::try_new("scalar").unwrap(), Column::Scalar(&[]));
    table.insert(
        Identifier::try_new("boolean").unwrap(),
        Column::Boolean(&[]),
    );
    assert_eq!(borrowed_table.into_inner(), table);
}

#[test]
fn we_can_create_a_table_with_data() {
    let alloc = Bump::new();

    let borrowed_table = table::<TestScalar>([
        borrowed_bigint(
            "bigint",
            [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
            &alloc,
        ),
        borrowed_int128(
            "decimal",
            [0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX],
            &alloc,
        ),
        borrowed_varchar(
            "varchar",
            ["0", "1", "2", "3", "4", "5", "6", "7", "8"],
            &alloc,
        ),
        borrowed_scalar("scalar", [0, 1, 2, 3, 4, 5, 6, 7, 8], &alloc),
        borrowed_boolean(
            "boolean",
            [true, false, true, false, true, false, true, false, true],
            &alloc,
        ),
        borrowed_timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
            &alloc,
        ),
    ]);

    let mut expected_table = IndexMap::default();

    let time_stamp_data = alloc.alloc_slice_copy(&[0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]);
    expected_table.insert(
        Identifier::try_new("time_stamp").unwrap(),
        Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, time_stamp_data),
    );

    let bigint_data = alloc.alloc_slice_copy(&[0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]);
    expected_table.insert(
        Identifier::try_new("bigint").unwrap(),
        Column::BigInt(bigint_data),
    );

    let decimal_data = alloc.alloc_slice_copy(&[0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]);
    expected_table.insert(
        Identifier::try_new("decimal").unwrap(),
        Column::Int128(decimal_data),
    );

    let varchar_data: Vec<&str> = ["0", "1", "2", "3", "4", "5", "6", "7", "8"]
        .iter()
        .map(|&s| alloc.alloc_str(s) as &str)
        .collect();
    let varchar_str_slice = alloc.alloc_slice_clone(&varchar_data);
    let varchar_scalars: Vec<TestScalar> = varchar_data.iter().map(Into::into).collect();
    let varchar_scalars_slice = alloc.alloc_slice_clone(&varchar_scalars);
    expected_table.insert(
        Identifier::try_new("varchar").unwrap(),
        Column::VarChar((varchar_str_slice, varchar_scalars_slice)),
    );

    let scalar_data: Vec<TestScalar> = (0..=8).map(TestScalar::from).collect();
    let scalar_slice = alloc.alloc_slice_copy(&scalar_data);
    expected_table.insert(
        Identifier::try_new("scalar").unwrap(),
        Column::Scalar(scalar_slice),
    );

    let boolean_data =
        alloc.alloc_slice_copy(&[true, false, true, false, true, false, true, false, true]);
    expected_table.insert(
        Identifier::try_new("boolean").unwrap(),
        Column::Boolean(boolean_data),
    );

    assert_eq!(borrowed_table.into_inner(), expected_table);
}

#[test]
fn we_get_inequality_between_tables_with_differing_column_order() {
    let alloc = Bump::new();

    let table_a: Table<'_, TestScalar> = table([
        borrowed_bigint("a", [0; 0], &alloc),
        borrowed_int128("b", [0; 0], &alloc),
        borrowed_varchar("c", ["0"; 0], &alloc),
        borrowed_boolean("d", [false; 0], &alloc),
        borrowed_timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [0_i64; 0],
            &alloc,
        ),
    ]);

    let table_b: Table<'_, TestScalar> = table([
        borrowed_boolean("d", [false; 0], &alloc),
        borrowed_int128("b", [0; 0], &alloc),
        borrowed_bigint("a", [0; 0], &alloc),
        borrowed_varchar("c", ["0"; 0], &alloc),
        borrowed_timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [0_i64; 0],
            &alloc,
        ),
    ]);

    assert_ne!(table_a, table_b);
}

#[test]
fn we_get_inequality_between_tables_with_differing_data() {
    let alloc = Bump::new();

    let table_a: Table<'_, TestScalar> = table([
        borrowed_bigint("a", [0], &alloc),
        borrowed_int128("b", [0], &alloc),
        borrowed_varchar("c", ["0"], &alloc),
        borrowed_boolean("d", [true], &alloc),
        borrowed_timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [1_625_072_400],
            &alloc,
        ),
    ]);

    let table_b: Table<'_, TestScalar> = table([
        borrowed_bigint("a", [1], &alloc),
        borrowed_int128("b", [0], &alloc),
        borrowed_varchar("c", ["0"], &alloc),
        borrowed_boolean("d", [true], &alloc),
        borrowed_timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::Utc,
            [1_625_076_000],
            &alloc,
        ),
    ]);

    assert_ne!(table_a, table_b);
}

#[test]
fn we_cannot_create_a_table_with_differing_column_lengths() {
    assert!(matches!(
        Table::<TestScalar>::try_from_iter([
            ("a".parse().unwrap(), Column::BigInt(&[0])),
            ("b".parse().unwrap(), Column::BigInt(&[])),
        ]),
        Err(TableError::ColumnLengthMismatch)
    ));
}
