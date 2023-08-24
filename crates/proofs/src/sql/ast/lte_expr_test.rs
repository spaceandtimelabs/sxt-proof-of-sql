use super::{prover_evaluate_equals_zero, prover_evaluate_or, FilterExpr, LteExpr};

use crate::base::bit::BitDistribution;
use crate::base::database::TestAccessor;
use crate::base::scalar::ArkScalar;
use crate::record_batch;
use crate::sql::ast::test_utility::{col, cols_result, tab};
use crate::sql::proof::{ProofBuilder, QueryProof, VerifiableQueryResult};
use bumpalo::Bump;
use num_traits::Zero;

#[test]
fn we_can_compare_a_constant_column() {
    let data = record_batch!(
        "a" => [123_i64, 123, 123],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(LteExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);
    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let expected_res = record_batch!(
        "b" => &[] as &[i64],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_a_constant_column_of_negative_columns() {
    let data = record_batch!(
        "a" => [-123_i64, -123, -123],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(LteExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);
    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let expected_res = record_batch!(
        "b" => [1_i64, 2, 3],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_compare_a_constant_column_of_zeros() {
    let data = record_batch!(
        "a" => [0_i64, 0, 0],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(LteExpr::new(col(t, "a", &accessor), 0.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);
    let res = res.verify(&expr, &accessor).unwrap().unwrap();
    let expected_res = record_batch!(
        "b" => [1_i64, 2, 3],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn the_sign_can_be_0_or_1_for_a_constant_column_of_zeros() {
    let data = record_batch!(
        "a" => [0_i64, 0, 0],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(LteExpr::new(col(t, "a", &accessor), 0.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let alloc = Bump::new();
    let mut builder = ProofBuilder::new(3, 2);

    let lhs = [ArkScalar::zero(); 3];
    let equals_zero = prover_evaluate_equals_zero(&mut builder, &alloc, &lhs);

    let mut bit_distribution = BitDistribution {
        or_all: [0; 4],
        vary_mask: [0; 4],
    };
    bit_distribution.or_all[3] = 1 << 63;
    assert!(bit_distribution.sign_bit());
    builder.produce_bit_distribution(bit_distribution);
    let sign = [true; 3];
    prover_evaluate_or(&mut builder, &alloc, equals_zero, &sign);
    builder.set_result_indexes(&[0, 1, 2]);

    let result_cols = cols_result(t, &["b"], &accessor);
    let selection = [true; 3];
    result_cols[0].prover_evaluate(&mut builder, &alloc, &accessor, &selection);

    let (proof, res) = QueryProof::new_from_builder(builder, 0);
    let res = proof.verify(&expr, &accessor, &res).unwrap().unwrap();
    let expected_res = record_batch!(
        "b" => [1_i64, 2, 3],
    );
    assert_eq!(res, expected_res);
}

#[test]
fn verification_fails_if_commitments_dont_match_for_a_constant_column() {
    let data = record_batch!(
        "a" => [123_i64, 123, 123],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(LteExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    let res = VerifiableQueryResult::new(&expr, &accessor);

    let data = record_batch!(
        "a" => [321_i64, 321, 321],
        "b" => [1_i64, 2, 3],
    );
    let t = "sxt.t".parse().unwrap();
    let mut accessor = TestAccessor::new();
    accessor.add_table(t, data, 0);
    let where_clause = Box::new(LteExpr::new(col(t, "a", &accessor), 5.into()));
    let expr = FilterExpr::new(cols_result(t, &["b"], &accessor), tab(t), where_clause);
    assert!(res.verify(&expr, &accessor).is_err());
}
