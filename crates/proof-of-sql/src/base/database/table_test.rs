use crate::base::{
    database::{table_utility::*, Column, Table, TableError, TableOptions},
    map::{indexmap, IndexMap},
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;
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

#[test]
fn we_can_get_table_with_presence_information() {
    let alloc = Bump::new();
    let mut table = table::<TestScalar>([
        borrowed_bigint("a", [0, 1, 2], &alloc),
        borrowed_int128("b", [0, 1, 2], &alloc),
    ]);

    let presence_a = &[true, false, true];
    let presence_slice = alloc.alloc_slice_copy(presence_a);
    table.set_column_presence("a", presence_slice).unwrap();

    let (columns, presence_map) = table.into_inner_with_presence();
    assert_eq!(columns.len(), 2);
    assert_eq!(presence_map.len(), 1);
    assert!(presence_map.contains_key(&Ident::new("a")));
}

#[test]
fn we_can_add_rho_column_when_it_already_exists() {
    let alloc = Bump::new();
    let original_table = table::<TestScalar>([
        borrowed_bigint("a", [0, 1], &alloc),
        borrowed_int128("rho", [10, 20], &alloc),
    ]);

    let enhanced_table = original_table.add_rho_column(&alloc);
    let expected_table = table::<TestScalar>([
        borrowed_bigint("a", [0, 1], &alloc),
        borrowed_int128("rho", [10, 20], &alloc),
    ]);

    assert_eq!(enhanced_table, expected_table);
}

#[test]
fn we_can_create_table_with_all_present() {
    let alloc = Bump::new();
    let mut table = table::<TestScalar>([
        borrowed_bigint("a", [0, 1, 2], &alloc),
        borrowed_int128("b", [0, 1, 2], &alloc),
    ]);

    let presence_a = &[true, false, true];
    let presence_slice = alloc.alloc_slice_copy(presence_a);
    table.set_column_presence("a", presence_slice).unwrap();
    let all_present_table = table.with_all_present();
    assert_eq!(all_present_table.presence_map().len(), 0);
    assert_eq!(all_present_table.inner_table().len(), 2);
}

#[test]
fn we_can_create_table_with_presence_from_another_table() {
    let alloc = Bump::new();
    let mut source_table = table::<TestScalar>([
        borrowed_bigint("a", [0, 1, 2], &alloc),
        borrowed_int128("b", [0, 1, 2], &alloc),
        borrowed_varchar("c", ["0", "1", "2"], &alloc),
    ]);

    let presence_a = &[true, false, true];
    let presence_b = &[false, true, false];
    let presence_slice_a = alloc.alloc_slice_copy(presence_a);
    let presence_slice_b = alloc.alloc_slice_copy(presence_b);
    source_table
        .set_column_presence("a", presence_slice_a)
        .unwrap();
    source_table
        .set_column_presence("b", presence_slice_b)
        .unwrap();

    let target_table = table::<TestScalar>([
        borrowed_bigint("a", [5, 6, 7], &alloc),
        borrowed_varchar("c", ["5", "6", "7"], &alloc),
        borrowed_boolean("d", [true, false, true], &alloc),
    ]);

    let result_table = target_table.with_presence_from(&source_table);

    assert_eq!(result_table.presence_map().len(), 1);
    assert!(result_table.presence_map().contains_key(&Ident::new("a")));
    assert!(!result_table.presence_map().contains_key(&Ident::new("c")));
}

#[test]
fn we_can_set_column_presence() {
    let alloc = Bump::new();
    let mut table = table::<TestScalar>([
        borrowed_bigint("a", [0, 1, 2], &alloc),
        borrowed_int128("b", [0, 1, 2], &alloc),
    ]);

    let presence_a = &[true, false, true];
    let presence_slice = alloc.alloc_slice_copy(presence_a);
    let result = table.set_column_presence("a", presence_slice);

    assert!(result.is_ok());
    assert!(table.presence_map().contains_key(&Ident::new("a")));

    let wrong_length_presence = &[true, false];
    let wrong_presence_slice = alloc.alloc_slice_copy(wrong_length_presence);
    let result = table.set_column_presence("b", wrong_presence_slice);
    assert!(matches!(result, Err(TableError::PresenceLengthMismatch)));

    let presence_x = &[true, false, true];
    let presence_x_slice = alloc.alloc_slice_copy(presence_x);
    let result = table.set_column_presence("x", presence_x_slice);

    assert!(result.is_ok());
    assert_eq!(table.presence_map().len(), 1);
}

#[test]
fn we_can_get_nullable_column() {
    let alloc = Bump::new();
    let mut table = table::<TestScalar>([
        borrowed_bigint("a", [0, 1, 2], &alloc),
        borrowed_int128("b", [0, 1, 2], &alloc),
    ]);

    let presence_a = &[true, false, true];
    let presence_slice = alloc.alloc_slice_copy(presence_a);
    table.set_column_presence("a", presence_slice).unwrap();

    let nullable_a = table.nullable_column("a");
    assert!(nullable_a.is_some());
    let nullable_a = nullable_a.unwrap();
    assert!(nullable_a.presence.is_some());

    let nullable_b = table.nullable_column("b");
    assert!(nullable_b.is_some());
    let nullable_b = nullable_b.unwrap();
    assert_eq!(nullable_b.presence, None);

    let nullable_c = table.nullable_column("c");
    assert!(nullable_c.is_none());
}

#[test]
fn we_can_get_column_presence() {
    let alloc = Bump::new();
    let mut table = table::<TestScalar>([
        borrowed_bigint("a", [0, 1, 2], &alloc),
        borrowed_int128("b", [0, 1, 2], &alloc),
    ]);

    let presence_a = &[true, false, true];
    let presence_slice = alloc.alloc_slice_copy(presence_a);
    table.set_column_presence("a", presence_slice).unwrap();

    let result = table.column_presence("a");
    assert!(result.is_some());

    let result = table.column_presence("b");
    assert_eq!(result, None);

    let result = table.column_presence("c");
    assert_eq!(result, None);
}

#[test]
fn we_can_create_table_with_presence() {
    let alloc = Bump::new();
    let mut table = table::<TestScalar>([
        borrowed_bigint("a", [0, 1, 2], &alloc),
        borrowed_int128("b", [0, 1, 2], &alloc),
    ]);

    let presence_data = &[true, false, true];
    let presence_slice = alloc.alloc_slice_copy(presence_data);
    table.set_column_presence("a", presence_slice).unwrap();

    let (columns, presence_map) = table.into_inner_with_presence();
    let result =
        Table::<TestScalar>::try_new_with_presence(columns, presence_map, TableOptions::default());

    assert!(result.is_ok());
    let new_table = result.unwrap();

    assert!(new_table.presence_map().contains_key(&Ident::new("a")));
    assert_eq!(new_table.presence_map().len(), 1);

    let alloc2 = Bump::new();
    let column_a = Column::BigInt(alloc2.alloc_slice_copy(&[0, 1, 2]));
    let column_b = Column::Int128(alloc2.alloc_slice_copy(&[0, 1, 2]));

    let mut column_map = IndexMap::default();
    column_map.insert(Ident::new("a"), column_a);
    column_map.insert(Ident::new("b"), column_b);

    let _orig_table = Table::<TestScalar>::try_new(column_map.clone()).unwrap();
    let mut wrong_table =
        crate::base::database::table_utility::table::<TestScalar>([borrowed_bigint(
            "a",
            [0, 1],
            &alloc2,
        )]);

    let wrong_presence = &[true, false];
    let wrong_presence_slice = alloc2.alloc_slice_copy(wrong_presence);
    wrong_table
        .set_column_presence("a", wrong_presence_slice)
        .unwrap();

    let (_, wrong_presence_map) = wrong_table.into_inner_with_presence();
    let result = Table::<TestScalar>::try_new_with_presence(
        column_map,
        wrong_presence_map,
        TableOptions::default(),
    );

    assert!(matches!(result, Err(TableError::PresenceLengthMismatch)));
}
