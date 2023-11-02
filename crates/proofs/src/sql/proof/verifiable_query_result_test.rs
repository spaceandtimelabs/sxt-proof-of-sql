use super::{ProofCounts, TestQueryExpr, VerifiableQueryResult};
use crate::base::database::TestAccessor;
use arrow::{
    array::Int64Array,
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use std::sync::Arc;

#[test]
fn we_can_verify_queries_on_an_empty_table() {
    let counts = ProofCounts {
        result_columns: 1,
        ..Default::default()
    };
    let expr = TestQueryExpr {
        counts,
        ..Default::default()
    };
    let accessor = TestAccessor::new();
    let res = VerifiableQueryResult::new(&expr, &accessor);
    let res = res.verify(&expr, &accessor).unwrap().into_record_batch();
    let schema = Schema::new(vec![Field::new("a1", DataType::Int64, false)]);
    let schema = Arc::new(schema);
    let expected_res =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(Vec::<i64>::new()))]).unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn empty_verification_fails_if_the_result_contains_non_null_members() {
    let counts = ProofCounts {
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
