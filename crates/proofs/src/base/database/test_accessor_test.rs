use super::{
    Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor, MetadataAccessor,
    SchemaAccessor, TestAccessor,
};
use crate::base::scalar::compute_commitment_for_testing;
use arrow::array::Int64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use indexmap::IndexMap;
use polars::prelude::*;
use proofs_sql::Identifier;
use std::sync::Arc;

#[test]
fn we_can_query_the_length_of_a_table() {
    let mut accessor = TestAccessor::new();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    accessor.add_table(
        table_ref_1,
        &IndexMap::from([
            ("a".to_string(), vec![1, 2, 3]),
            ("b".to_string(), vec![4, 5, 6]),
        ]),
        0_usize,
    );

    assert_eq!(accessor.get_length(table_ref_1), 3);

    accessor.add_table(
        table_ref_2,
        &IndexMap::from([
            ("a".to_string(), vec![1, 2, 3, 4]),
            ("b".to_string(), vec![4, 5, 6, 5]),
        ]),
        0_usize,
    );

    assert_eq!(accessor.get_length(table_ref_1), 3);
    assert_eq!(accessor.get_length(table_ref_2), 4);
}

#[test]
fn we_can_access_the_columns_of_a_table() {
    let mut accessor = TestAccessor::new();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    accessor.add_table(
        table_ref_1,
        &IndexMap::from([
            ("a".to_string(), vec![1, 2, 3]),
            ("b".to_string(), vec![4, 5, 6]),
        ]),
        0_usize,
    );

    let column = ColumnRef::new(
        table_ref_1,
        Identifier::try_new("b").unwrap(),
        ColumnType::BigInt,
    );
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![4, 5, 6]),
    };

    accessor.add_table(
        table_ref_2,
        &IndexMap::from([
            ("a".to_string(), vec![1, 2, 3, 4]),
            ("b".to_string(), vec![4, 5, 6, 5]),
        ]),
        0_usize,
    );

    let column = ColumnRef::new(
        table_ref_1,
        Identifier::try_new("a").unwrap(),
        ColumnType::BigInt,
    );
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![1, 2, 3]),
    };

    let column = ColumnRef::new(
        table_ref_2,
        Identifier::try_new("b").unwrap(),
        ColumnType::BigInt,
    );
    match accessor.get_column(column) {
        Column::BigInt(col) => assert_eq!(col.to_vec(), vec![4, 5, 6, 5]),
    };
}

#[test]
fn we_can_access_the_commitments_of_table_columns() {
    let mut accessor = TestAccessor::new();
    let table_ref_1 = "sxt.test".parse().unwrap();
    let table_ref_2 = "sxt.test2".parse().unwrap();

    accessor.add_table(
        table_ref_1,
        &IndexMap::from([
            ("a".to_string(), vec![1, 2, 3]),
            ("b".to_string(), vec![4, 5, 6]),
        ]),
        0_usize,
    );

    let column = ColumnRef::new(
        table_ref_1,
        Identifier::try_new("b").unwrap(),
        ColumnType::BigInt,
    );
    assert_eq!(
        accessor.get_commitment(column),
        compute_commitment_for_testing(&[4, 5, 6], 0_usize)
    );

    accessor.add_table(
        table_ref_2,
        &IndexMap::from([
            ("a".to_string(), vec![1, 2, 3, 4]),
            ("b".to_string(), vec![4, 5, 6, 5]),
        ]),
        0_usize,
    );

    let column = ColumnRef::new(
        table_ref_1,
        Identifier::try_new("a").unwrap(),
        ColumnType::BigInt,
    );
    assert_eq!(
        accessor.get_commitment(column),
        compute_commitment_for_testing(&[1, 2, 3], 0_usize)
    );

    let column = ColumnRef::new(
        table_ref_2,
        Identifier::try_new("b").unwrap(),
        ColumnType::BigInt,
    );
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

    accessor.add_table(
        table_ref_1,
        &IndexMap::from([
            ("a".to_string(), vec![1, 2, 3]),
            ("b".to_string(), vec![4, 5, 6]),
        ]),
        0_usize,
    );

    let column = ColumnRef::new(
        table_ref_1,
        Identifier::try_new("b").unwrap(),
        ColumnType::BigInt,
    );
    assert_eq!(accessor.lookup_column(column), Some(ColumnType::BigInt));

    let column = ColumnRef::new(
        table_ref_1,
        Identifier::try_new("c").unwrap(),
        ColumnType::BigInt,
    );
    assert!(accessor.lookup_column(column).is_none());

    accessor.add_table(
        table_ref_2,
        &IndexMap::from([
            ("a".to_string(), vec![1, 2, 3, 4]),
            ("b".to_string(), vec![4, 5, 6, 5]),
        ]),
        0_usize,
    );

    let column = ColumnRef::new(
        table_ref_1,
        Identifier::try_new("a").unwrap(),
        ColumnType::BigInt,
    );
    assert_eq!(accessor.lookup_column(column), Some(ColumnType::BigInt));

    let column = ColumnRef::new(
        table_ref_2,
        Identifier::try_new("b").unwrap(),
        ColumnType::BigInt,
    );
    assert_eq!(accessor.lookup_column(column), Some(ColumnType::BigInt));

    let column = ColumnRef::new(
        table_ref_2,
        Identifier::try_new("c").unwrap(),
        ColumnType::BigInt,
    );
    assert!(accessor.lookup_column(column).is_none());
}

#[test]
fn we_can_run_arbitrary_queries_on_a_table() {
    let mut accessor = TestAccessor::new();
    let table_ref_1 = "sxt.test".parse().unwrap();

    accessor.add_table(
        table_ref_1,
        &IndexMap::from([
            ("a".to_string(), vec![1, 2, 3]),
            ("b".to_string(), vec![123, 5, 123]),
        ]),
        0_usize,
    );
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
