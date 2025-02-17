use super::{
    Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
    SchemaAccessor, TableTestAccessor, TestAccessor,
};
use crate::base::{
    commitment::{
        naive_commitment::NaiveCommitment, naive_evaluation_proof::NaiveEvaluationProof,
        Commitment, CommittableColumn,
    },
    database::{table_utility::*, TableRef},
    scalar::test_scalar::TestScalar,
};
use bumpalo::Bump;
use proof_of_sql_parser::posql_time::PoSQLTimeUnit;
use sqlparser::ast::TimezoneInfo;

#[test]
fn we_can_query_the_length_of_a_table() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    let table_ref_1 = TableRef::new("sxt", "test");
    let table_ref_2 = TableRef::new("sxt", "test2");

    let data1 = table([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_bigint("b", [4, 5, 6], &alloc),
    ]);
    accessor.add_table(table_ref_1.clone(), data1, 0_usize);

    assert_eq!(accessor.get_length(&table_ref_1), 3);

    let data2 = table([
        borrowed_bigint("a", [1, 2, 3, 4], &alloc),
        borrowed_bigint("b", [4, 5, 6, 5], &alloc),
    ]);
    accessor.add_table(table_ref_2.clone(), data2, 0_usize);

    assert_eq!(accessor.get_length(&table_ref_1), 3);
    assert_eq!(accessor.get_length(&table_ref_2), 4);
}

#[test]
fn we_can_access_the_columns_of_a_table() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    let table_ref_1 = TableRef::new("sxt", "test");
    let table_ref_2 = TableRef::new("sxt", "test2");

    let data1 = table([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_bigint("b", [4, 5, 6], &alloc),
    ]);
    accessor.add_table(table_ref_1.clone(), data1, 0_usize);

    let column = ColumnRef::new(table_ref_1.clone(), "b".into(), ColumnType::BigInt);
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![4, 5, 6]),
        _ => panic!("Invalid column type"),
    };

    let data2 = table([
        borrowed_bigint("a", [1, 2, 3, 4], &alloc),
        borrowed_bigint("b", [4, 5, 6, 5], &alloc),
        borrowed_int128("c128", [1, 2, 3, 4], &alloc),
        borrowed_varchar("varchar", ["a", "bc", "d", "e"], &alloc),
        borrowed_scalar("scalar", [1, 2, 3, 4], &alloc),
        borrowed_boolean("boolean", [true, false, true, false], &alloc),
        borrowed_timestamptz(
            "time",
            PoSQLTimeUnit::Second,
            TimezoneInfo::None,
            [4, 5, 6, 5],
            &alloc,
        ),
    ]);
    accessor.add_table(table_ref_2.clone(), data2, 0_usize);

    let column = ColumnRef::new(table_ref_1.clone(), "a".into(), ColumnType::BigInt);
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![1, 2, 3]),
        _ => panic!("Invalid column type"),
    };

    let column = ColumnRef::new(table_ref_2.clone(), "b".into(), ColumnType::BigInt);
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![4, 5, 6, 5]),
        _ => panic!("Invalid column type"),
    };

    let column = ColumnRef::new(table_ref_2.clone(), "c128".into(), ColumnType::Int128);
    match accessor.get_column(column) {
        Column::Int128(col) => assert_eq!(col.to_vec(), vec![1, 2, 3, 4]),
        _ => panic!("Invalid column type"),
    };

    let col_slice: Vec<_> = vec!["a", "bc", "d", "e"];
    let col_scalars: Vec<_> = ["a", "bc", "d", "e"]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column = ColumnRef::new(table_ref_2.clone(), "varchar".into(), ColumnType::VarChar);
    match accessor.get_column(column) {
        Column::VarChar((col, scals)) => {
            assert_eq!(col.to_vec(), col_slice);
            assert_eq!(scals.to_vec(), col_scalars);
        }
        _ => panic!("Invalid column type"),
    };

    let column = ColumnRef::new(table_ref_2.clone(), "scalar".into(), ColumnType::Scalar);
    match accessor.get_column(column) {
        Column::Scalar(col) => assert_eq!(
            col.to_vec(),
            vec![
                TestScalar::from(1),
                TestScalar::from(2),
                TestScalar::from(3),
                TestScalar::from(4)
            ]
        ),
        _ => panic!("Invalid column type"),
    };

    let column = ColumnRef::new(table_ref_2.clone(), "boolean".into(), ColumnType::Boolean);
    match accessor.get_column(column) {
        Column::Boolean(col) => assert_eq!(col.to_vec(), vec![true, false, true, false]),
        _ => panic!("Invalid column type"),
    };

    let column = ColumnRef::new(
        table_ref_2.clone(),
        "time".into(),
        ColumnType::TimestampTZ(PoSQLTimeUnit::Second, TimezoneInfo::None),
    );
    match accessor.get_column(column) {
        Column::TimestampTZ(_, _, col) => assert_eq!(col.to_vec(), vec![4, 5, 6, 5]),
        _ => panic!("Invalid column type"),
    };
}

#[test]
fn we_can_access_the_commitments_of_table_columns() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    let table_ref_1 = TableRef::new("sxt", "test");
    let table_ref_2 = TableRef::new("sxt", "test2");

    let data1 = table([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_bigint("b", [4, 5, 6], &alloc),
    ]);
    accessor.add_table(table_ref_1.clone(), data1, 0_usize);

    let column = ColumnRef::new(table_ref_1.clone(), "b".into(), ColumnType::BigInt);
    assert_eq!(
        accessor.get_commitment(column),
        NaiveCommitment::compute_commitments(
            &[CommittableColumn::from(&[4i64, 5, 6][..])],
            0_usize,
            &()
        )[0]
    );

    let data2 = table([
        borrowed_bigint("a", [1, 2, 3, 4], &alloc),
        borrowed_bigint("b", [4, 5, 6, 5], &alloc),
    ]);
    accessor.add_table(table_ref_2.clone(), data2, 0_usize);

    let column = ColumnRef::new(table_ref_1.clone(), "a".into(), ColumnType::BigInt);
    assert_eq!(
        accessor.get_commitment(column),
        NaiveCommitment::compute_commitments(
            &[CommittableColumn::from(&[1i64, 2, 3][..])],
            0_usize,
            &()
        )[0]
    );

    let column = ColumnRef::new(table_ref_2.clone(), "b".into(), ColumnType::BigInt);
    assert_eq!(
        accessor.get_commitment(column),
        NaiveCommitment::compute_commitments(
            &[CommittableColumn::from(&[4i64, 5, 6, 5][..])],
            0_usize,
            &()
        )[0]
    );
}

#[test]
fn we_can_access_the_type_of_table_columns() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    let table_ref_1 = TableRef::new("sxt", "test");
    let table_ref_2 = TableRef::new("sxt", "test2");

    let data1 = table([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_bigint("b", [4, 5, 6], &alloc),
    ]);
    accessor.add_table(table_ref_1.clone(), data1, 0_usize);

    let column = ColumnRef::new(table_ref_1.clone(), "b".into(), ColumnType::BigInt);
    assert_eq!(
        accessor.lookup_column(column.table_ref(), column.column_id()),
        Some(ColumnType::BigInt)
    );

    let column = ColumnRef::new(table_ref_1.clone(), "c".into(), ColumnType::BigInt);
    assert!(accessor
        .lookup_column(column.table_ref(), column.column_id())
        .is_none());

    let data2 = table([
        borrowed_bigint("a", [1, 2, 3, 4], &alloc),
        borrowed_bigint("b", [4, 5, 6, 5], &alloc),
    ]);
    accessor.add_table(table_ref_2.clone(), data2, 0_usize);

    let column = ColumnRef::new(table_ref_1.clone(), "a".into(), ColumnType::BigInt);
    assert_eq!(
        accessor.lookup_column(column.table_ref(), column.column_id()),
        Some(ColumnType::BigInt)
    );

    let column = ColumnRef::new(table_ref_2.clone(), "b".into(), ColumnType::BigInt);
    assert_eq!(
        accessor.lookup_column(column.table_ref(), column.column_id()),
        Some(ColumnType::BigInt)
    );

    let column = ColumnRef::new(table_ref_2.clone(), "c".into(), ColumnType::BigInt);
    assert!(accessor
        .lookup_column(column.table_ref(), column.column_id())
        .is_none());
}

#[test]
fn we_can_access_schema_and_column_names() {
    let alloc = Bump::new();
    let mut accessor = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    let table_ref_1 = TableRef::new("sxt", "test");

    let data1 = table([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_varchar("b", ["x", "y", "z"], &alloc),
    ]);
    accessor.add_table(table_ref_1.clone(), data1, 0_usize);

    assert_eq!(
        accessor.lookup_schema(table_ref_1.clone()),
        vec![
            ("a".into(), ColumnType::BigInt),
            ("b".into(), ColumnType::VarChar)
        ]
    );
    assert_eq!(accessor.get_column_names(&table_ref_1), vec!["a", "b"]);
}

#[test]
fn we_can_correctly_update_offsets() {
    let alloc = Bump::new();
    let mut accessor1 = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    let table_ref = TableRef::new("sxt", "test");

    let data = table([
        borrowed_bigint("a", [1, 2, 3], &alloc),
        borrowed_bigint("b", [123, 5, 123], &alloc),
    ]);
    accessor1.add_table(table_ref.clone(), data.clone(), 0_usize);

    let offset = 123;
    let mut accessor2 = TableTestAccessor::<NaiveEvaluationProof>::new_empty_with_setup(());
    accessor2.add_table(table_ref.clone(), data, offset);

    let column = ColumnRef::new(table_ref.clone(), "a".into(), ColumnType::BigInt);
    assert_ne!(
        accessor1.get_commitment(column.clone()),
        accessor2.get_commitment(column)
    );
    let column = ColumnRef::new(table_ref.clone(), "b".into(), ColumnType::BigInt);
    assert_ne!(
        accessor1.get_commitment(column.clone()),
        accessor2.get_commitment(column)
    );

    assert_eq!(accessor1.get_offset(&table_ref), 0);
    assert_eq!(accessor2.get_offset(&table_ref), offset);

    accessor1.update_offset(&table_ref, offset);

    let column = ColumnRef::new(table_ref.clone(), "a".into(), ColumnType::BigInt);
    assert_eq!(
        accessor1.get_commitment(column.clone()),
        accessor2.get_commitment(column)
    );
    let column = ColumnRef::new(table_ref.clone(), "b".into(), ColumnType::BigInt);
    assert_eq!(
        accessor1.get_commitment(column.clone()),
        accessor2.get_commitment(column)
    );

    assert_eq!(accessor1.get_offset(&table_ref), offset);
    assert_eq!(accessor2.get_offset(&table_ref), offset);
}
