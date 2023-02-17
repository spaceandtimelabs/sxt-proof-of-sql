use super::{
    Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
    SchemaAccessor, TestAccessor,
};
use crate::base::scalar::compute_commitment_for_testing;
use crate::base::scalar::ToScalar;

use arrow::array::Int64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use polars::prelude::*;
use std::sync::Arc;

#[test]
fn we_can_query_the_length_of_a_table() {
    let mut accessor = TestAccessor::new();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = df!(
        "a" => [1, 2, 3],
        "b" => [4, 5, 6]
    )
    .unwrap();
    accessor.add_table(table_ref_1, data1, 0_usize);

    assert_eq!(accessor.get_length(table_ref_1), 3);

    let data2 = df!(
        "a" => [1, 2, 3, 4],
        "b" => [4, 5, 6, 5],
    )
    .unwrap();
    accessor.add_table(table_ref_2, data2, 0_usize);

    assert_eq!(accessor.get_length(table_ref_1), 3);
    assert_eq!(accessor.get_length(table_ref_2), 4);
}

#[test]
fn we_can_access_the_columns_of_a_table() {
    let mut accessor = TestAccessor::new();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = df!(
        "a" => [1, 2, 3],
        "b" => [4, 5, 6],
    )
    .unwrap();
    accessor.add_table(table_ref_1, data1, 0_usize);

    let column = ColumnRef::new(table_ref_1, "b".parse().unwrap(), ColumnType::BigInt);
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![4, 5, 6]),
        _ => panic!("Invalid column type"),
    };

    let data2 = df!(
        "a" => [1, 2, 3, 4],
        "d" => ["a", "bc", "d", "e"],
        "b" => [4, 5, 6, 5],
    )
    .unwrap();
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

    let col_slice: Vec<_> = ["a", "bc", "d", "e"].iter().map(|v| v.as_bytes()).collect();
    let col_scalars: Vec<_> = ["a", "bc", "d", "e"]
        .iter()
        .map(|v| v.to_scalar())
        .collect();
    let column = ColumnRef::new(table_ref_2, "d".parse().unwrap(), ColumnType::VarChar);
    match accessor.get_column(column) {
        Column::HashedBytes((col, scals)) => {
            assert_eq!(col.to_vec(), col_slice);
            assert_eq!(scals.to_vec(), col_scalars);
        }
        _ => panic!("Invalid column type"),
    };
}

#[test]
fn we_can_access_the_commitments_of_table_columns() {
    let mut accessor = TestAccessor::new();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = df!(
        "a" => [1, 2, 3],
        "b" => [4, 5, 6],
    )
    .unwrap();
    accessor.add_table(table_ref_1, data1, 0_usize);

    let column = ColumnRef::new(table_ref_1, "b".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(
        accessor.get_commitment(column),
        compute_commitment_for_testing(&[4, 5, 6], 0_usize)
    );

    let data2 = df!(
        "a" => [1, 2, 3, 4],
        "b" => [4, 5, 6, 5],
    )
    .unwrap();
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
    let mut accessor = TestAccessor::new();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    let data1 = df!(
        "a" => [1, 2, 3],
        "b" => [4, 5, 6],
    )
    .unwrap();
    accessor.add_table(table_ref_1, data1, 0_usize);

    let column = ColumnRef::new(table_ref_1, "b".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(accessor.lookup_column(column), Some(ColumnType::BigInt));

    let column = ColumnRef::new(table_ref_1, "c".parse().unwrap(), ColumnType::BigInt);
    assert!(accessor.lookup_column(column).is_none());

    let data2 = df!(
        "a" => [1, 2, 3, 4],
        "b" => [4, 5, 6, 5],
    )
    .unwrap();
    accessor.add_table(table_ref_2, data2, 0_usize);

    let column = ColumnRef::new(table_ref_1, "a".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(accessor.lookup_column(column), Some(ColumnType::BigInt));

    let column = ColumnRef::new(table_ref_2, "b".parse().unwrap(), ColumnType::BigInt);
    assert_eq!(accessor.lookup_column(column), Some(ColumnType::BigInt));

    let column = ColumnRef::new(table_ref_2, "c".parse().unwrap(), ColumnType::BigInt);
    assert!(accessor.lookup_column(column).is_none());
}

#[test]
fn we_can_run_arbitrary_queries_on_a_table() {
    let mut accessor = TestAccessor::new();
    let table_ref_1 = "sxt.test".parse().unwrap();

    let data = df!(
        "a" => [1, 2, 3],
        "b" => [123, 5, 123],
    )
    .unwrap();
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
fn we_can_correctly_update_offsets() {
    let mut accessor1 = TestAccessor::new();
    let table_ref = "sxt.test".parse().unwrap();

    let data = df!(
        "a" => [1, 2, 3],
        "b" => [123, 5, 123],
    )
    .unwrap();
    accessor1.add_table(table_ref, data.clone(), 0_usize);

    let offset = 123;
    let mut accessor2 = TestAccessor::new();
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
