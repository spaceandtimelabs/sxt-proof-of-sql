use super::{
    Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
    RecordBatchTestAccessor, SchemaAccessor, TestAccessor,
};
use crate::{base::scalar::compute_commitment_for_testing, record_batch};
use arrow::{
    array::Int64Array,
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use polars::prelude::*;
use std::sync::Arc;

#[test]
fn we_can_query_the_length_of_a_table() {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = record_batch!(
        "a" => [1_i64, 2, 3],
        "b" => [4_i64, 5, 6]
    );
    accessor.add_table(table_ref_1, data1, 0_usize);

    assert_eq!(accessor.get_length(table_ref_1), 3);

    let data2 = record_batch!(
        "a" => [1_i64, 2, 3, 4],
        "b" => [4_i64, 5, 6, 5],
    );
    accessor.add_table(table_ref_2, data2, 0_usize);

    assert_eq!(accessor.get_length(table_ref_1), 3);
    assert_eq!(accessor.get_length(table_ref_2), 4);
}

#[test]
fn we_can_access_the_columns_of_a_table() {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = record_batch!(
        "a" => [1_i64, 2, 3],
        "b" => [4_i64, 5, 6],
    );
    accessor.add_table(table_ref_1, data1, 0_usize);

    let column = ColumnRef::new(table_ref_1, "b".parse().unwrap(), ColumnType::BigInt);
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![4, 5, 6]),
        _ => panic!("Invalid column type"),
    };

    let data2 = record_batch!(
        "a" => [1_i64, 2, 3, 4],
        "d" => ["a", "bc", "d", "e"],
        "b" => [4_i64, 5, 6, 5],
        "c128" => [1_i128, 2, 3, 4],
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
}

#[test]
fn we_can_access_the_commitments_of_table_columns() {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = record_batch!(
        "a" => [1_i64, 2, 3],
        "b" => [4_i64, 5, 6],
    );
    accessor.add_table(table_ref_1, data1, 0_usize);

    let column = ColumnRef::new(table_ref_1, "b".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(
        accessor.get_commitment(column),
        compute_commitment_for_testing(&[4, 5, 6], 0_usize)
    );

    let data2 = record_batch!(
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
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = record_batch!(
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

    let data2 = record_batch!(
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
fn we_can_run_arbitrary_queries_on_a_table() {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let table_ref_1 = "sxt.test".parse().unwrap();

    let data = record_batch!(
        "a" => [1_i64, 2, 3],
        "b" => [123_i64, 5, 123],
    );
    accessor.add_table(table_ref_1, data, 0_usize);
    let res = accessor.query_table(table_ref_1, |df| {
        df.clone()
            .lazy()
            .filter(col("b").eq(123))
            .select([col("a")])
            .collect()
            .unwrap()
    });
    let schema = Arc::new(Schema::new(vec![Field::new("a", DataType::Int64, false)]));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![1, 3]))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_access_schema_and_column_names() {
    let mut accessor = RecordBatchTestAccessor::new_empty();
    let table_ref_1 = "sxt.test".parse().unwrap();

    let data1: RecordBatch = record_batch!(
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
    let mut accessor1 = RecordBatchTestAccessor::new_empty();
    let table_ref = "sxt.test".parse().unwrap();

    let data = record_batch!(
        "a" => [1_i64, 2, 3],
        "b" => [123_i64, 5, 123],
    );
    accessor1.add_table(table_ref, data.clone(), 0_usize);

    let offset = 123;
    let mut accessor2 = RecordBatchTestAccessor::new_empty();
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
