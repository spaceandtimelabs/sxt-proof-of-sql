use super::{EqualsExpr, FilterExpr, FilterResultExpr, TableExpr};
use crate::base::database::{
    make_random_test_accessor_data, ColumnRef, ColumnType, RandomTestAccessorDescriptor, TableRef,
    TestAccessor,
};
use crate::base::scalar::ToScalar;
use crate::sql::proof::QueryExpr;
use crate::sql::proof::{exercise_verification, VerifiableQueryResult};
use arrow::array::Int64Array;
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use curve25519_dalek::scalar::Scalar;
use polars::prelude::*;
use proofs_sql::Identifier;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;
use std::sync::Arc;

#[test]
fn we_can_prove_an_equality_query_with_no_rows() {
    let table_ref: TableRef = "sxt.t".parse().unwrap();
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr { table_ref },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::zero(),
        )),
    );
    let data = df!(
        "a" => Vec::<i64>::new(),
        "b" => Vec::<i64>::new()
    )
    .unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(table_ref, data, 0_usize);
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor, table_ref);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let res_col: Vec<i64> = vec![];
    let column_fields = expr
        .get_column_result_fields()
        .iter()
        .map(|v| v.into())
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(res_col))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_selected_row() {
    let table_ref: TableRef = "sxt.t".parse().unwrap();
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr { table_ref },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::zero(),
        )),
    );
    let data = df!(
        "a" => [123],
        "b" => [0]
    )
    .unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(table_ref, data, 0_usize);
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor, table_ref);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let res_col: Vec<i64> = vec![123];
    let column_fields = expr
        .get_column_result_fields()
        .iter()
        .map(|v| v.into())
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(res_col))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_non_selected_row() {
    let table_ref: TableRef = "sxt.t".parse().unwrap();
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr { table_ref },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::zero(),
        )),
    );
    let data = df!(
        "a" => [123],
        "b" => [55]
    )
    .unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(table_ref, data, 0_usize);
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor, table_ref);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let res_col: Vec<i64> = vec![];
    let column_fields = expr
        .get_column_result_fields()
        .iter()
        .map(|v| v.into())
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(res_col))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_multiple_rows() {
    let table_ref: TableRef = "sxt.t".parse().unwrap();
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr { table_ref },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::zero(),
        )),
    );

    let data = df!(
        "a" => [1, 2, 3, 4],
        "b" => [0, 5, 0, 5]
    )
    .unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(table_ref, data, 0_usize);
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor, table_ref);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let column_fields = expr
        .get_column_result_fields()
        .iter()
        .map(|v| v.into())
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![1, 3]))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_nonzero_comparison() {
    let table_ref: TableRef = "sxt.t".parse().unwrap();
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr { table_ref },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::from(123u64),
        )),
    );

    let data = df!(
        "a" => [1, 2, 3, 4, 5],
        "b" => [123, 5, 123, 5, 0],
    )
    .unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(table_ref, data, 0_usize);
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor, table_ref);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let column_fields = expr
        .get_column_result_fields()
        .iter()
        .map(|v| v.into())
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![1, 3]))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn verify_fails_if_data_between_prover_and_verifier_differ() {
    let table_ref: TableRef = "sxt.t".parse().unwrap();
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("a").unwrap(),
                ColumnType::BigInt,
            ),
            Identifier::try_new("a").unwrap(),
        )],
        TableExpr { table_ref },
        Box::new(EqualsExpr::new(
            ColumnRef::new(
                table_ref,
                Identifier::try_new("b").unwrap(),
                ColumnType::BigInt,
            ),
            Scalar::zero(),
        )),
    );
    let data = df!(
        "a" => [1, 2, 3, 4],
        "b" => [0, 5, 0, 5],
    )
    .unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(table_ref, data, 0_usize);
    let res = VerifiableQueryResult::new(&expr, &accessor);
    let data = df!(
        "a" => [1, 2, 3, 4],
        "b" => [0, 2, 0, 5],
    )
    .unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(table_ref, data, 0_usize);
    assert!(res.verify(&expr, &accessor).is_err());
}

fn test_random_tables_with_given_offset(offset_generators: usize) {
    let descr = RandomTestAccessorDescriptor {
        min_rows: 1,
        max_rows: 20,
        min_value: -3,
        max_value: 3,
    };
    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols = ["a", "b"];
    for _ in 0..10 {
        let table_ref: TableRef = "sxt.t".parse().unwrap();
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let mut accessor = TestAccessor::new();
        accessor.add_table(table_ref, data, offset_generators);

        let val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let expr = FilterExpr::new(
            vec![FilterResultExpr::new(
                ColumnRef::new(
                    table_ref,
                    Identifier::try_new("a").unwrap(),
                    ColumnType::BigInt,
                ),
                Identifier::try_new("a").unwrap(),
            )],
            TableExpr { table_ref },
            Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref,
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ),
                val.to_scalar(),
            )),
        );
        let proof_res = VerifiableQueryResult::new(&expr, &accessor);
        exercise_verification(&proof_res, &expr, &accessor, table_ref);
        let res = proof_res.verify(&expr, &accessor).unwrap().unwrap();
        let expected = accessor.query_table(table_ref, |df| {
            df.clone()
                .lazy()
                .filter(col("b").eq(val))
                .select([col("a")])
                .collect()
                .unwrap()
        });
        assert_eq!(res, expected);
    }
}

#[test]
fn we_can_query_random_tables_with_a_zero_offset() {
    test_random_tables_with_given_offset(0);
}

#[test]
fn we_can_query_random_tables_with_a_non_zero_offset() {
    test_random_tables_with_given_offset(121);
}

#[test]
fn we_can_query_random_tables_with_multiple_selected_rows() {
    let descr = RandomTestAccessorDescriptor {
        min_rows: 1,
        max_rows: 20,
        min_value: -3,
        max_value: 3,
    };
    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols = ["aa", "ab", "b"];
    for _ in 0..10 {
        let table_ref: TableRef = "sxt.t".parse().unwrap();
        let data = make_random_test_accessor_data(&mut rng, &cols, &descr);
        let mut accessor = TestAccessor::new();
        accessor.add_table(table_ref, data, 0_usize);

        let val = Uniform::new(descr.min_value, descr.max_value + 1).sample(&mut rng);
        let expr = FilterExpr::new(
            vec![
                FilterResultExpr::new(
                    ColumnRef::new(
                        table_ref,
                        Identifier::try_new("aa").unwrap(),
                        ColumnType::BigInt,
                    ),
                    Identifier::try_new("aa").unwrap(),
                ),
                FilterResultExpr::new(
                    ColumnRef::new(
                        table_ref,
                        Identifier::try_new("ab").unwrap(),
                        ColumnType::BigInt,
                    ),
                    Identifier::try_new("ab").unwrap(),
                ),
            ],
            TableExpr { table_ref },
            Box::new(EqualsExpr::new(
                ColumnRef::new(
                    table_ref,
                    Identifier::try_new("b").unwrap(),
                    ColumnType::BigInt,
                ),
                val.to_scalar(),
            )),
        );
        let res = VerifiableQueryResult::new(&expr, &accessor);
        exercise_verification(&res, &expr, &accessor, table_ref);
        let res = res.verify(&expr, &accessor).unwrap().unwrap();
        let expected = accessor.query_table(table_ref, |df| {
            df.clone()
                .lazy()
                .filter(col("b").eq(val))
                .select([col("aa"), col("ab")])
                .collect()
                .unwrap()
        });
        assert_eq!(res, expected);
    }
}
