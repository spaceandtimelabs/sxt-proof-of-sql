use super::{EqualsExpr, FilterExpr, FilterResultExpr, TableExpr};

use crate::base::database::{make_schema, TestAccessor};
use crate::sql::proof::{exercise_verification, VerifiableQueryResult};

use arrow::array::Int64Array;
use arrow::record_batch::RecordBatch;
use curve25519_dalek::scalar::Scalar;
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn we_can_prove_an_equality_query_with_no_rows() {
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new("A".to_string())],
        TableExpr {
            name: "T".to_string(),
        },
        Box::new(EqualsExpr::new("B".to_string(), Scalar::zero())),
    );
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "T",
        &HashMap::from([("A".to_string(), vec![]), ("B".to_string(), vec![])]),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let res_col: Vec<i64> = vec![];
    let expected_res =
        RecordBatch::try_new(make_schema(1), vec![Arc::new(Int64Array::from(res_col))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_selected_row() {
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new("A".to_string())],
        TableExpr {
            name: "T".to_string(),
        },
        Box::new(EqualsExpr::new("B".to_string(), Scalar::zero())),
    );
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "T",
        &HashMap::from([("A".to_string(), vec![123]), ("B".to_string(), vec![0])]),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let res_col: Vec<i64> = vec![123];
    let expected_res =
        RecordBatch::try_new(make_schema(1), vec![Arc::new(Int64Array::from(res_col))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_single_non_selected_row() {
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new("A".to_string())],
        TableExpr {
            name: "T".to_string(),
        },
        Box::new(EqualsExpr::new("B".to_string(), Scalar::zero())),
    );
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "T",
        &HashMap::from([("A".to_string(), vec![123]), ("B".to_string(), vec![55])]),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let res_col: Vec<i64> = vec![];
    let expected_res =
        RecordBatch::try_new(make_schema(1), vec![Arc::new(Int64Array::from(res_col))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_multiple_rows() {
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new("A".to_string())],
        TableExpr {
            name: "T".to_string(),
        },
        Box::new(EqualsExpr::new("B".to_string(), Scalar::zero())),
    );
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "T",
        &HashMap::from([
            ("A".to_string(), vec![1, 2, 3, 4]),
            ("B".to_string(), vec![0, 5, 0, 5]),
        ]),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let expected_res =
        RecordBatch::try_new(make_schema(1), vec![Arc::new(Int64Array::from(vec![1, 3]))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_prove_an_equality_query_with_a_nonzero_comparison() {
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new("A".to_string())],
        TableExpr {
            name: "T".to_string(),
        },
        Box::new(EqualsExpr::new("B".to_string(), Scalar::from(123u64))),
    );
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "T",
        &HashMap::from([
            ("A".to_string(), vec![1, 2, 3, 4, 5]),
            ("B".to_string(), vec![123, 5, 123, 5, 0]),
        ]),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor);

    exercise_verification(&res, &expr, &accessor);

    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let expected_res =
        RecordBatch::try_new(make_schema(1), vec![Arc::new(Int64Array::from(vec![1, 3]))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn verify_fails_if_data_between_prover_and_verifier_differ() {
    let expr = FilterExpr::new(
        vec![FilterResultExpr::new("A".to_string())],
        TableExpr {
            name: "T".to_string(),
        },
        Box::new(EqualsExpr::new("B".to_string(), Scalar::zero())),
    );
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "T",
        &HashMap::from([
            ("A".to_string(), vec![1, 2, 3, 4]),
            ("B".to_string(), vec![0, 5, 0, 5]),
        ]),
    );
    let res = VerifiableQueryResult::new(&expr, &accessor);
    let mut accessor = TestAccessor::new();
    accessor.add_table(
        "T",
        &HashMap::from([
            ("A".to_string(), vec![1, 2, 3, 4]),
            ("B".to_string(), vec![0, 2, 0, 5]),
        ]),
    );
    assert!(res.verify(&expr, &accessor).is_err());
}
