use super::{
    Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
    OwnedTableTestAccessor, SchemaAccessor, TestAccessor,
};
use crate::{
    base::scalar::{compute_commitment_for_testing, Curve25519Scalar},
    owned_table,
};
use blitzar::proof::InnerProductProof;

#[test]
fn we_can_query_the_length_of_a_table() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = owned_table!(
        "a" => [1_i64, 2, 3],
        "b" => [4_i64, 5, 6]
    );
    accessor.add_table(table_ref_1, data1, 0_usize);

    assert_eq!(accessor.get_length(table_ref_1), 3);

    let data2 = owned_table!(
        "a" => [1_i64, 2, 3, 4],
        "b" => [4_i64, 5, 6, 5],
    );
    accessor.add_table(table_ref_2, data2, 0_usize);

    assert_eq!(accessor.get_length(table_ref_1), 3);
    assert_eq!(accessor.get_length(table_ref_2), 4);
}

#[test]
fn we_can_access_the_columns_of_a_table() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = owned_table!(
        "a" => [1_i64, 2, 3],
        "b" => [4_i64, 5, 6],
    );
    accessor.add_table(table_ref_1, data1, 0_usize);

    let column = ColumnRef::new(table_ref_1, "b".parse().unwrap(), ColumnType::BigInt);
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![4, 5, 6]),
        _ => panic!("Invalid column type"),
    };

    let data2 = owned_table!(
        "a" => [1_i64, 2, 3, 4],
        "b" => [4_i64, 5, 6, 5],
        "c128" => [1_i128, 2, 3, 4],
        "d" => ["a", "bc", "d", "e"],
        "e" => [Curve25519Scalar::from(1), Curve25519Scalar::from(2), Curve25519Scalar::from(3), Curve25519Scalar::from(4)],
    );
    accessor.add_table(table_ref_2, data2, 0_usize);

    let column = ColumnRef::new(table_ref_1, "a".parse().unwrap(), ColumnType::BigInt);
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![1, 2, 3]),
        _ => panic!("Invalid column type"),
    };

    let column = ColumnRef::new(table_ref_2, "b".parse().unwrap(), ColumnType::BigInt);
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![4, 5, 6, 5]),
        _ => panic!("Invalid column type"),
    };

    let column = ColumnRef::new(table_ref_2, "c128".parse().unwrap(), ColumnType::Int128);
    match accessor.get_column(column) {
        Column::Int128(col) => assert_eq!(col.to_vec(), vec![1, 2, 3, 4]),
        _ => panic!("Invalid column type"),
    };

    let col_slice: Vec<_> = vec!["a", "bc", "d", "e"];
    let col_scalars: Vec<_> = ["a", "bc", "d", "e"].iter().map(|v| v.into()).collect();
    let column = ColumnRef::new(table_ref_2, "d".parse().unwrap(), ColumnType::VarChar);
    match accessor.get_column(column) {
        Column::VarChar((col, scals)) => {
            assert_eq!(col.to_vec(), col_slice);
            assert_eq!(scals.to_vec(), col_scalars);
        }
        _ => panic!("Invalid column type"),
    };

    let column = ColumnRef::new(table_ref_2, "e".parse().unwrap(), ColumnType::Scalar);
    match accessor.get_column(column) {
        Column::Scalar(col) => assert_eq!(
            col.to_vec(),
            vec![
                Curve25519Scalar::from(1),
                Curve25519Scalar::from(2),
                Curve25519Scalar::from(3),
                Curve25519Scalar::from(4)
            ]
        ),
        _ => panic!("Invalid column type"),
    };
}

#[test]
fn we_can_access_the_commitments_of_table_columns() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = owned_table!(
        "a" => [1_i64, 2, 3],
        "b" => [4_i64, 5, 6],
    );
    accessor.add_table(table_ref_1, data1, 0_usize);

    let column = ColumnRef::new(table_ref_1, "b".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(
        accessor.get_commitment(column),
        compute_commitment_for_testing(&[4, 5, 6], 0_usize)
    );

    let data2 = owned_table!(
        "a" => [1_i64, 2, 3, 4],
        "b" => [4_i64, 5, 6, 5],
    );
    accessor.add_table(table_ref_2, data2, 0_usize);

    let column = ColumnRef::new(table_ref_1, "a".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(
        accessor.get_commitment(column),
        compute_commitment_for_testing(&[1, 2, 3], 0_usize)
    );

    let column = ColumnRef::new(table_ref_2, "b".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(
        accessor.get_commitment(column),
        compute_commitment_for_testing(&[4, 5, 6, 5], 0_usize)
    );
}

#[test]
fn we_can_access_the_type_of_table_columns() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = owned_table!(
        "a" => [1_i64, 2, 3],
        "b" => [4_i64, 5, 6],
    );
    accessor.add_table(table_ref_1, data1, 0_usize);

    let column = ColumnRef::new(table_ref_1, "b".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(
        accessor.lookup_column(column.table_ref(), column.column_id()),
        Some(ColumnType::BigInt)
    );

    let column = ColumnRef::new(table_ref_1, "c".parse().unwrap(), ColumnType::BigInt);
    assert!(accessor
        .lookup_column(column.table_ref(), column.column_id())
        .is_none());

    let data2 = owned_table!(
        "a" => [1_i64, 2, 3, 4],
        "b" => [4_i64, 5, 6, 5],
    );
    accessor.add_table(table_ref_2, data2, 0_usize);

    let column = ColumnRef::new(table_ref_1, "a".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(
        accessor.lookup_column(column.table_ref(), column.column_id()),
        Some(ColumnType::BigInt)
    );

    let column = ColumnRef::new(table_ref_2, "b".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(
        accessor.lookup_column(column.table_ref(), column.column_id()),
        Some(ColumnType::BigInt)
    );

    let column = ColumnRef::new(table_ref_2, "c".parse().unwrap(), ColumnType::BigInt);
    assert!(accessor
        .lookup_column(column.table_ref(), column.column_id())
        .is_none());
}

#[test]
fn we_can_access_schema_and_column_names() {
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let table_ref_1 = "sxt.test".parse().unwrap();

    let data1 = owned_table!(
        "a" => [1_i64, 2, 3],
        "b" => ["x", "y", "z"],
    );
    accessor.add_table(table_ref_1, data1, 0_usize);

    assert_eq!(
        accessor.lookup_schema(table_ref_1),
        vec![
            ("a".parse().unwrap(), ColumnType::BigInt),
            ("b".parse().unwrap(), ColumnType::VarChar)
        ]
    );
    assert_eq!(accessor.get_column_names(table_ref_1), vec!["a", "b"]);
}

#[test]
fn we_can_correctly_update_offsets() {
    let mut accessor1 = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    let table_ref = "sxt.test".parse().unwrap();

    let data = owned_table!(
        "a" => [1_i64, 2, 3],
        "b" => [123_i64, 5, 123],
    );
    accessor1.add_table(table_ref, data.clone(), 0_usize);

    let offset = 123;
    let mut accessor2 = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor2.add_table(table_ref, data, offset);

    let column = ColumnRef::new(table_ref, "a".parse().unwrap(), ColumnType::BigInt);
    assert_ne!(
        accessor1.get_commitment(column),
        accessor2.get_commitment(column)
    );
    let column = ColumnRef::new(table_ref, "b".parse().unwrap(), ColumnType::BigInt);
    assert_ne!(
        accessor1.get_commitment(column),
        accessor2.get_commitment(column)
    );

    assert_eq!(accessor1.get_offset(table_ref), 0);
    assert_eq!(accessor2.get_offset(table_ref), offset);

    accessor1.update_offset(table_ref, offset);

    let column = ColumnRef::new(table_ref, "a".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(
        accessor1.get_commitment(column),
        accessor2.get_commitment(column)
    );
    let column = ColumnRef::new(table_ref, "b".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(
        accessor1.get_commitment(column),
        accessor2.get_commitment(column)
    );

    assert_eq!(accessor1.get_offset(table_ref), offset);
    assert_eq!(accessor2.get_offset(table_ref), offset);
}
