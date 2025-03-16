use crate::base::{
    commitment::naive_commitment::NaiveCommitment,
    scalar::{test_scalar::TestScalar, Scalar},
};
use alloc::vec::Vec;

// PartialEq Tests

#[test]
fn we_can_compare_columns_with_equal_length() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = [1i64, 10, -5, 0, 11]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let commitment_a = NaiveCommitment(column_a.clone());
    let commitment_b = NaiveCommitment(column_b);
    let commitment_c = NaiveCommitment(column_a);
    assert_ne!(commitment_a, commitment_b);
    assert_eq!(commitment_a, commitment_c);
}

#[test]
fn we_can_compare_columns_with_different_length() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 11, 0]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = [1i64, 10, -5, 0, 11]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_c: Vec<TestScalar> = [1i64, 10, -5]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let commitment_a = NaiveCommitment(column_a.clone());
    let commitment_b = NaiveCommitment(column_b);
    let commitment_c = NaiveCommitment(column_c);
    assert_eq!(commitment_a, commitment_b);
    // Confirm < and > are handled the same.
    assert_eq!(commitment_b, commitment_a);
    assert_ne!(commitment_b, commitment_c);
    // Confirm < and > are handled the same.
    assert_ne!(commitment_c, commitment_b);
}

#[test]
fn we_can_compare_columns_with_at_least_one_empty() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = Vec::new();
    let commitment_a = NaiveCommitment(column_a);
    let commitment_b = NaiveCommitment(column_b.clone());
    let commitment_c = NaiveCommitment(column_b);
    assert_ne!(commitment_a, commitment_b);
    assert_eq!(commitment_b, commitment_c);
}

// PartialEq Tests End

// Add Tests

#[test]
fn we_can_add_naive_commitments() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = [2i64, -10, -5, 5, 100]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_sum: Vec<TestScalar> = [3i64, 0, -10, 5, 110]
        .iter()
        .map(core::convert::Into::into)
        .collect();

    let commitment_a = NaiveCommitment(column_a);
    let commitment_b = NaiveCommitment(column_b);
    let commitment_sum = NaiveCommitment(column_sum);

    // Confirm homeomorphic property
    assert_eq!(commitment_a.clone() + commitment_b.clone(), commitment_sum);
    // Check commutativity
    assert_eq!(commitment_b + commitment_a, commitment_sum);
}

#[test]
fn we_can_add_naive_commitments_with_one_empty() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = Vec::new();

    let commitment_a = NaiveCommitment(column_a);
    let commitment_b = NaiveCommitment(column_b);

    // Confirm homeomorphic property
    assert_eq!(commitment_a.clone() + commitment_b.clone(), commitment_a);
    // Check commutativity
    assert_eq!(commitment_b + commitment_a.clone(), commitment_a);
}

#[test]
fn we_can_add_naive_commitments_with_both_empty() {
    let column_a: Vec<TestScalar> = Vec::new();

    let commitment_a = NaiveCommitment(column_a);

    // Confirm homeomorphic property
    assert_eq!(commitment_a.clone() + commitment_a.clone(), commitment_a);
}

// Add Tests End

// Sub Tests

#[test]
fn we_can_subtract_naive_commitments() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = [2i64, -10, -5, 5, 100]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_difference: Vec<TestScalar> = [-1i64, 20, 0, -5, -90]
        .iter()
        .map(core::convert::Into::into)
        .collect();

    let commitment_a = NaiveCommitment(column_a);
    let commitment_b = NaiveCommitment(column_b);
    let commitment_difference = NaiveCommitment(column_difference);

    // Confirm homeomorphic property
    assert_eq!(commitment_a - commitment_b, commitment_difference);
}

#[test]
fn we_can_subtract_naive_commitments_with_one_empty() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = Vec::new();
    let column_b_minus_a = [-1i64, -10, 5, 0, -10]
        .iter()
        .map(core::convert::Into::into)
        .collect();

    let commitment_a = NaiveCommitment(column_a.clone());
    let commitment_b = NaiveCommitment(column_b);
    let commitment_b_minus_a = NaiveCommitment(column_b_minus_a);
    let commitment_a_minus_b = NaiveCommitment(column_a);

    // Confirm homeomorphic property
    assert_eq!(
        commitment_a.clone() - commitment_b.clone(),
        commitment_a_minus_b
    );
    assert_eq!(commitment_b - commitment_a, commitment_b_minus_a);
}

#[test]
fn we_can_subtract_naive_commitments_with_both_empty() {
    let column_a: Vec<TestScalar> = Vec::new();

    let commitment_a = NaiveCommitment(column_a);

    // Confirm homeomorphic property
    assert_eq!(commitment_a.clone() - commitment_a.clone(), commitment_a);
}

// Sub Tests End

// AddAssign Tests

#[expect(clippy::similar_names)]
#[test]
fn we_can_add_assign_naive_commitments() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = [2i64, -10, -5, 5, 100]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_sum: Vec<TestScalar> = [3i64, 0, -10, 5, 110]
        .iter()
        .map(core::convert::Into::into)
        .collect();

    let commitment_a = NaiveCommitment(column_a.clone());
    let commitment_b = NaiveCommitment(column_b.clone());
    let mut commitment_a_mutable = NaiveCommitment(column_a);
    let mut commitment_b_mutable = NaiveCommitment(column_b);
    let commitment_sum = NaiveCommitment(column_sum);

    // Add assign a + b and b + a
    commitment_a_mutable += commitment_b;
    commitment_b_mutable += commitment_a;

    // Confirm homeomorphic property
    assert_eq!(commitment_a_mutable, commitment_sum);
    // Check commutativity
    assert_eq!(commitment_b_mutable, commitment_sum);
}

#[expect(clippy::similar_names)]
#[test]
fn we_can_add_assign_naive_commitments_with_one_empty() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = Vec::new();

    let commitment_a = NaiveCommitment(column_a.clone());
    let commitment_b = NaiveCommitment(column_b.clone());
    let mut commitment_a_mutable = NaiveCommitment(column_a.clone());
    let mut commitment_b_mutable = NaiveCommitment(column_b);
    let commitment_sum = NaiveCommitment(column_a.clone());

    // Add assign a + b and b + a
    commitment_a_mutable += commitment_b;
    commitment_b_mutable += commitment_a;

    // Confirm homeomorphic property
    assert_eq!(commitment_a_mutable, commitment_sum);
    // Check commutativity
    assert_eq!(commitment_b_mutable, commitment_sum);
}

#[test]
fn we_can_add_assign_naive_commitments_with_both_empty() {
    let column_a: Vec<TestScalar> = Vec::new();

    let commitment_a = NaiveCommitment(column_a.clone());
    let mut commitment_a_mutable = NaiveCommitment(column_a.clone());
    let commitment_sum = NaiveCommitment(column_a);
    commitment_a_mutable += commitment_a.clone();
    // Confirm homeomorphic property
    assert_eq!(commitment_a_mutable, commitment_sum);
}

// AddAssign Tests End

// SubAssign Tests

#[test]
fn we_can_sub_assign_naive_commitments() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = [2i64, -10, -5, 5, 100]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_difference: Vec<TestScalar> = [-1i64, 20, 0, -5, -90]
        .iter()
        .map(core::convert::Into::into)
        .collect();

    let commitment_b = NaiveCommitment(column_b.clone());
    let mut commitment_a_mutable = NaiveCommitment(column_a);
    let commitment_difference = NaiveCommitment(column_difference);

    // Sub assign a - b
    commitment_a_mutable -= commitment_b;

    // Confirm homeomorphic property
    assert_eq!(commitment_a_mutable, commitment_difference);
}

#[expect(clippy::similar_names)]
#[test]
fn we_can_sub_assign_naive_commitments_with_one_empty() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_b: Vec<TestScalar> = Vec::new();
    let column_b_minus_a = [-1i64, -10, 5, 0, -10]
        .iter()
        .map(core::convert::Into::into)
        .collect();

    let commitment_a = NaiveCommitment(column_a.clone());
    let commitment_b = NaiveCommitment(column_b.clone());
    let mut commitment_a_mutable = NaiveCommitment(column_a.clone());
    let mut commitment_b_mutable = NaiveCommitment(column_b);
    let commitment_b_minus_a = NaiveCommitment(column_b_minus_a);
    let commitment_a_minus_b = NaiveCommitment(column_a);

    // Sub assign a - b and b - a
    commitment_a_mutable -= commitment_b;
    commitment_b_mutable -= commitment_a;

    // Confirm homeomorphic property
    assert_eq!(commitment_a_mutable, commitment_a_minus_b);
    // Check commutativity
    assert_eq!(commitment_b_mutable, commitment_b_minus_a);
}

#[test]
fn we_can_sub_assign_naive_commitments_with_both_empty() {
    let column_a: Vec<TestScalar> = Vec::new();

    let commitment_a = NaiveCommitment(column_a.clone());
    let mut commitment_a_mutable = NaiveCommitment(column_a.clone());
    commitment_a_mutable -= commitment_a.clone();
    // Confirm homeomorphic property
    assert_eq!(commitment_a_mutable, commitment_a);
}

// SubAssign Tests End

// Neg Tests

#[test]
fn we_can_negate_naive_commitments() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_negation: Vec<TestScalar> = [-1i64, -10, 5, 0, -10]
        .iter()
        .map(core::convert::Into::into)
        .collect();

    let commitment_a = NaiveCommitment(column_a);
    let commitment_negation = NaiveCommitment(column_negation);

    // Confirm homeomorphic property
    assert_eq!(-commitment_a, commitment_negation);
}

#[test]
fn we_can_negate_empty_naive_commitments() {
    let column_a: Vec<TestScalar> = Vec::new();

    let commitment_a = NaiveCommitment(column_a.clone());
    let commitment_negation = NaiveCommitment(column_a);

    // Confirm homeomorphic property
    assert_eq!(-commitment_a, commitment_negation);
}

// Neg Tests End

// Scalar Multiplication Tests

#[test]
fn we_can_do_scalar_multiplication() {
    let column_a: Vec<TestScalar> = [1i64, 10, -5, 0, 10]
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let column_empty: Vec<TestScalar> = Vec::new();
    let scalar: TestScalar = (-2i64).into();
    let zero = TestScalar::ZERO;
    let column_a_multiplied_by_scalar: Vec<TestScalar> = [-2i64, -20, 10, 0, -20]
        .iter()
        .map(core::convert::Into::into)
        .collect();

    let commitment_a = NaiveCommitment(column_a.clone());
    let commitment_empty = NaiveCommitment(column_empty);
    let commitment_a_multiplied_by_scalar = NaiveCommitment(column_a_multiplied_by_scalar);

    assert_eq!(&commitment_a * scalar, commitment_a_multiplied_by_scalar);
    assert_eq!(&commitment_a * zero, commitment_empty);
    assert_eq!(scalar * &commitment_a, commitment_a_multiplied_by_scalar);
    assert_eq!(zero * &commitment_a, commitment_empty);
    assert_eq!(&commitment_empty * scalar, commitment_empty);
    assert_eq!(&commitment_empty * zero, commitment_empty);
    assert_eq!(scalar * &commitment_empty, commitment_empty);
    assert_eq!(zero * &commitment_empty, commitment_empty);
    assert_eq!(
        commitment_a.clone() * scalar,
        commitment_a_multiplied_by_scalar
    );
    assert_eq!(commitment_a.clone() * zero, commitment_empty);
    assert_eq!(
        scalar * commitment_a.clone(),
        commitment_a_multiplied_by_scalar
    );
    assert_eq!(zero * commitment_a.clone(), commitment_empty);
    assert_eq!(commitment_empty.clone() * scalar, commitment_empty);
    assert_eq!(commitment_empty.clone() * zero, commitment_empty);
    assert_eq!(scalar * commitment_empty.clone(), commitment_empty);
    assert_eq!(zero * commitment_empty.clone(), commitment_empty);
}

// Scalar Multiplication Tests End
