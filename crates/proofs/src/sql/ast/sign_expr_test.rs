use super::SignExpr;
use crate::base::database::TestAccessor;

use super::FilterExpr;
use crate::base::scalar::ArkScalar;
use crate::record_batch;
use crate::sql::ast::test_utility::{col, cols_result, tab};
use crate::sql::proof::VerifiableQueryResult;

#[test]
fn we_handle_the_sign_decomposition_of_a_constant_column() {
    let data = record_batch!(
        "a" => [123, 123, 123],
        "b" => [1, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(SignExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);
    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let expected_res = record_batch!(
        "b" => &[] as &[i64],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn verification_fails_on_values_outside_of_the_acceptable_range() {
    let data = record_batch!(
        "a" => [123, 123, 123],
        "b" => [1, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let bigval = ArkScalar::from(3) * ArkScalar::from(u64::MAX);
    let where_clause = Box::new(SignExpr::new(col(t, "a", &accessor), bigval));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);
    assert!(res.verify(&expr, &accessor).is_err());
}

#[test]
fn we_handle_the_sign_decomposition_of_a_constant_column_of_negative_numbers() {
    let data = record_batch!(
        "a" => [-123, -123, -123],
        "b" => [1, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(SignExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);
    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let expected_res = record_batch!(
        "b" => [1, 2, 3],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn verification_of_a_constant_sign_decomposition_fails_if_commitments_dont_match() {
    let data = record_batch!(
        "a" => [123, 123, 123],
        "b" => [1, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(SignExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);

    let data = record_batch!(
        "a" => [1234, 1234, 1234],
        "b" => [1, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(SignExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    assert!(res.verify(&expr, &accessor).is_err());
}

#[test]
fn verification_of_a_constant_sign_decomposition_fails_if_signs_dont_match() {
    let data = record_batch!(
        "a" => [123, 123, 123],
        "b" => [1, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(SignExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);

    let data = record_batch!(
        "a" => [-123, -123, -123],
        "b" => [1, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(SignExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    assert!(res.verify(&expr, &accessor).is_err());
}

#[test]
fn verification_fails_if_a_bit_distribution_is_invalid() {
    let data = record_batch!(
        "a" => [1, 1],
        "b" => [1, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(SignExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let mut res = VerifiableQueryResult::new(&expr, &accessor);
    res.proof.as_mut().unwrap().bit_distributions[0].vary_mask[0] = 3;
    assert!(res.verify(&expr, &accessor).is_err());
}
