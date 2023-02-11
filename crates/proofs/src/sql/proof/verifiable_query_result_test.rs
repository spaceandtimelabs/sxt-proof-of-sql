use super::{ProofCounts, TestQueryExpr, VerifiableQueryResult};

use crate::base::database::TestAccessor;

use arrow::array::Int64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

#[test]
fn we_can_verify_queries_on_an_empty_table() {
    let counts = ProofCounts {
        sumcheck_variables: 0,
        result_columns: 1,
        ..Default::default()
    };
    let expr = TestQueryExpr {
        counts,
        ..Default::default()
    };
    let accessor = TestAccessor::new();
    let res = VerifiableQueryResult::new(&expr, &accessor);
    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let schema = Schema::new(vec![Field::new("a1", DataType::Int64, false)]);
    let schema = Arc::new(schema);
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(Vec::<i64>::new()))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn empty_verification_fails_if_the_result_contains_non_null_members() {
    let counts = ProofCounts {
        sumcheck_variables: 0,
        result_columns: 1,
        ..Default::default()
    };
    let expr = TestQueryExpr {
        counts,
        ..Default::default()
    };
    let accessor = TestAccessor::new();
    let res = VerifiableQueryResult {
        provable_result: Some(Default::default()),
        ..Default::default()
    };
    assert!(res.verify(&expr, &accessor).is_err());
}

#[test]
#[should_panic]
fn prove_panics_if_the_expr_has_no_result_columns() {
    let counts = ProofCounts {
        result_columns: 0,
        ..Default::default()
    };
    let expr = TestQueryExpr {
        counts,
        ..Default::default()
    };
    let accessor = TestAccessor::new();
    let _res = VerifiableQueryResult::new(&expr, &accessor);
}

#[test]
#[should_panic]
fn verify_panics_if_the_expr_has_no_result_columns() {
    let counts = ProofCounts {
        result_columns: 0,
        ..Default::default()
    };
    let expr = TestQueryExpr {
        counts,
        ..Default::default()
    };
    let accessor = TestAccessor::new();
    let res: VerifiableQueryResult = Default::default();
    let _res = res.verify(&expr, &accessor);
}
