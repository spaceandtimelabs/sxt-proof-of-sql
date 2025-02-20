use crate::base::{
    database::{Column, Table, TableError, TableOptions, table_utility::*},
    map::{IndexMap, indexmap},
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
use sqlparser::ast::Ident;
#[test]
fn we_can_create_a_table_with_no_columns_specifying_row_count() {
    let table =
        Table::<TestScalar>::try_new_with_options(IndexMap::default(), TableOptions::new(Some(1)))
            .unwrap();
    assert_eq!(table.num_columns(), 0);
    assert_eq!(table.num_rows(), 1);

    let table =
        Table::<TestScalar>::try_new_with_options(IndexMap::default(), TableOptions::new(Some(0)))
            .unwrap();
    assert_eq!(table.num_columns(), 0);
    assert_eq!(table.num_rows(), 0);
}

#[test]
fn we_can_create_a_table_with_default_options() {
    let table = Table::<TestScalar>::try_new(indexmap! {
        "a".into() => Column::BigInt(&[0, 1]),
        "b".into() => Column::Int128(&[0, 1]),
    })
    .unwrap();
    assert_eq!(table.num_columns(), 2);
    assert_eq!(table.num_rows(), 2);

    let table = Table::<TestScalar>::try_new(indexmap! {
        "a".into() => Column::BigInt(&[]),
        "b".into() => Column::Int128(&[]),
    })
    .unwrap();
    assert_eq!(table.num_columns(), 2);
    assert_eq!(table.num_rows(), 0);
}

#[test]
fn we_can_create_a_table_with_specified_row_count() {
    let table = Table::<TestScalar>::try_new_with_options(
        indexmap! {
            "a".into() => Column::BigInt(&[0, 1]),
            "b".into() => Column::Int128(&[0, 1]),
        },
        TableOptions::new(Some(2)),
    )
    .unwrap();
    assert_eq!(table.num_columns(), 2);
    assert_eq!(table.num_rows(), 2);

    let table = Table::<TestScalar>::try_new_with_options(
        indexmap! {
            "a".into() => Column::BigInt(&[]),
            "b".into() => Column::Int128(&[]),
        },
        TableOptions::new(Some(0)),
    )
    .unwrap();
    assert_eq!(table.num_columns(), 2);
    assert_eq!(table.num_rows(), 0);
}

#[test]
fn we_cannot_create_a_table_with_differing_column_lengths() {
    assert!(matches!(
        Table::<TestScalar>::try_from_iter([
            ("a".into(), Column::BigInt(&[0])),
            ("b".into(), Column::BigInt(&[])),
        ]),
        Err(TableError::ColumnLengthMismatch)
    ));
}

#[test]
fn we_cannot_create_a_table_with_column_length_different_from_specified_row_count() {
    assert!(matches!(
        Table::<TestScalar>::try_from_iter_with_options(
            [
                ("a".into(), Column::BigInt(&[0])),
                ("b".into(), Column::BigInt(&[1])),
            ],
            TableOptions::new(Some(0))
        ),
        Err(TableError::ColumnLengthMismatchWithSpecifiedRowCount)
    ));
}

#[test]
fn we_cannot_create_a_table_with_no_columns_without_specified_row_count() {
    assert!(matches!(
        Table::<TestScalar>::try_from_iter_with_options([], TableOptions::new(None)),
        Err(TableError::EmptyTableWithoutSpecifiedRowCount)
    ));

    assert!(matches!(
        Table::<TestScalar>::try_new(IndexMap::default()),
        Err(TableError::EmptyTableWithoutSpecifiedRowCount)
    ));
}

#[test]
fn we_can_create_an_empty_table_with_some_columns() {
    let alloc = Bump::new();
    let borrowed_table = table::<TestScalar>([
        borrowed_bigint("bigint", [0; 0], &alloc),
        borrowed_int128("decimal", [0; 0], &alloc),
        borrowed_varchar("varchar", ["0"; 0], &alloc),
        borrowed_scalar("scalar", [0; 0], &alloc),
        borrowed_boolean("boolean", [true; 0], &alloc),
    ]);
    let mut table = IndexMap::default();
    table.insert(Ident::new("bigint"), Column::BigInt(&[]));
    table.insert(Ident::new("decimal"), Column::Int128(&[]));
    table.insert(Ident::new("varchar"), Column::VarChar((&[], &[])));
    table.insert(Ident::new("scalar"), Column::Scalar(&[]));
    table.insert(Ident::new("boolean"), Column::Boolean(&[]));
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
            PoSQLTimeZone::utc(),
            [0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX],
            &alloc,
        ),
    ]);

    let mut expected_table = IndexMap::default();

    let time_stamp_data = alloc.alloc_slice_copy(&[0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]);
    expected_table.insert(
        Ident::new("time_stamp"),
        Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), time_stamp_data),
    );

    let bigint_data = alloc.alloc_slice_copy(&[0_i64, 1, 2, 3, 4, 5, 6, i64::MIN, i64::MAX]);
    expected_table.insert(Ident::new("bigint"), Column::BigInt(bigint_data));

    let decimal_data = alloc.alloc_slice_copy(&[0_i128, 1, 2, 3, 4, 5, 6, i128::MIN, i128::MAX]);
    expected_table.insert(Ident::new("decimal"), Column::Int128(decimal_data));

    let varchar_data: Vec<&str> = ["0", "1", "2", "3", "4", "5", "6", "7", "8"]
        .iter()
        .map(|&s| alloc.alloc_str(s) as &str)
        .collect();
    let varchar_str_slice = alloc.alloc_slice_clone(&varchar_data);
    let varchar_scalars: Vec<TestScalar> = varchar_data.iter().map(Into::into).collect();
    let varchar_scalars_slice = alloc.alloc_slice_clone(&varchar_scalars);
    expected_table.insert(
        Ident::new("varchar"),
        Column::VarChar((varchar_str_slice, varchar_scalars_slice)),
    );

    let scalar_data: Vec<TestScalar> = (0..=8).map(TestScalar::from).collect();
    let scalar_slice = alloc.alloc_slice_copy(&scalar_data);
    expected_table.insert(Ident::new("scalar"), Column::Scalar(scalar_slice));

    let boolean_data =
        alloc.alloc_slice_copy(&[true, false, true, false, true, false, true, false, true]);
    expected_table.insert(Ident::new("boolean"), Column::Boolean(boolean_data));

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
            PoSQLTimeZone::utc(),
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
            PoSQLTimeZone::utc(),
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
            PoSQLTimeZone::utc(),
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
            PoSQLTimeZone::utc(),
            [1_625_076_000],
            &alloc,
        ),
    ]);

    assert_ne!(table_a, table_b);
}

// add_rho_column
#[test]
fn we_can_add_rho_column_to_table_with_neither_columns_nor_rows() {
    let alloc = Bump::new();
    let original_table = table_with_row_count::<TestScalar>([], 0);
    let enhanced_table = original_table.add_rho_column(&alloc);
    let expected_table = table([borrowed_int128("rho", [0_i128; 0], &alloc)]);
    assert_eq!(enhanced_table, expected_table);
}

#[test]
fn we_can_add_rho_column_to_table_with_no_columns() {
    let alloc = Bump::new();
    let original_table = table_with_row_count::<TestScalar>([], 2);
    let enhanced_table = original_table.add_rho_column(&alloc);
    let expected_table = table([borrowed_int128("rho", [0_i128, 1], &alloc)]);
    assert_eq!(enhanced_table, expected_table);
}

#[test]
fn we_can_add_rho_column_to_table_with_no_rows() {
    let alloc = Bump::new();
    let original_table = table::<TestScalar>([
        borrowed_bigint("a", [0_i64; 0], &alloc),
        borrowed_int128("b", [0_i128; 0], &alloc),
        borrowed_varchar("c", ["0"; 0], &alloc),
        borrowed_boolean("d", [true; 0], &alloc),
        borrowed_timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0_i64; 0],
            &alloc,
        ),
    ]);
    let enhanced_table = original_table.add_rho_column(&alloc);
    let expected_table = table([
        borrowed_bigint("a", [0_i64; 0], &alloc),
        borrowed_int128("b", [0_i128; 0], &alloc),
        borrowed_varchar("c", ["0"; 0], &alloc),
        borrowed_boolean("d", [true; 0], &alloc),
        borrowed_timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0_i64; 0],
            &alloc,
        ),
        borrowed_int128("rho", [0_i128; 0], &alloc),
    ]);
    assert_eq!(enhanced_table, expected_table);
}

#[test]
fn we_can_add_rho_column() {
    let alloc = Bump::new();
    let original_table = table_with_row_count::<TestScalar>(
        [
            borrowed_bigint("a", [0_i64, 1, 2], &alloc),
            borrowed_int128("b", [0_i128, 1, 2], &alloc),
            borrowed_varchar("c", ["0", "1", "2"], &alloc),
            borrowed_boolean("d", [true, false, true], &alloc),
            borrowed_timestamptz(
                "time_stamp",
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::utc(),
                [0_i64, 1, 2],
                &alloc,
            ),
        ],
        3,
    );
    let enhanced_table = original_table.add_rho_column(&alloc);
    let expected_table = table([
        borrowed_bigint("a", [0_i64, 1, 2], &alloc),
        borrowed_int128("b", [0_i128, 1, 2], &alloc),
        borrowed_varchar("c", ["0", "1", "2"], &alloc),
        borrowed_boolean("d", [true, false, true], &alloc),
        borrowed_timestamptz(
            "time_stamp",
            PoSQLTimeUnit::Second,
            PoSQLTimeZone::utc(),
            [0_i64, 1, 2],
            &alloc,
        ),
        borrowed_int128("rho", [0_i128, 1, 2], &alloc),
    ]);
    assert_eq!(enhanced_table, expected_table);
}
