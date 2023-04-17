use std::iter;

use curve25519_dalek::scalar::Scalar;
use rand::{distributions::Uniform, prelude::Distribution, rngs::StdRng, SeedableRng};

use crate::sql::proof::{
    compute_evaluation_vector, compute_truncated_lagrange_basis_inner_product,
    compute_truncated_lagrange_basis_sum,
};

#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_0_variables() {
    let point = vec![];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        Scalar::from(1u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        Scalar::from(0u8)
    );
}
#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_1_variables() {
    let point = vec![Scalar::from(2u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(2, &point),
        Scalar::from(1u8) // This is (1-2) + (2)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        -Scalar::from(1u8) // This is (1-2)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        Scalar::from(0u8)
    );
}
#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_2_variables() {
    let point = vec![Scalar::from(2u8), Scalar::from(5u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(4, &point),
        Scalar::from(1u8) // This is (1-2)(1-5)+(2)(1-5)+(1-2)(5)+(2)(5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(3, &point),
        -Scalar::from(9u8) // This is (1-2)(1-5)+(2)(1-5)+(1-2)(5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(2, &point),
        -Scalar::from(4u8) // This is (1-2)(1-5)+(2)(1-5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        Scalar::from(4u8) // This is (1-2)(1-5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        Scalar::from(0u8)
    );
}

#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_3_variables() {
    let point = vec![Scalar::from(2u8), Scalar::from(5u8), Scalar::from(7u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(8, &point),
        Scalar::from(1u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(7, &point),
        -Scalar::from(69u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(6, &point),
        -Scalar::from(34u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(5, &point),
        Scalar::from(22u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(4, &point),
        -Scalar::from(6u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(3, &point),
        Scalar::from(54u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(2, &point),
        Scalar::from(24u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        -Scalar::from(24u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        Scalar::from(0u8)
    );
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_0_variables() {
    let a = vec![];
    let b = vec![];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        Scalar::from(1u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        Scalar::from(0u32)
    );
}
#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_1_variables() {
    let a = vec![Scalar::from(2u8)];
    let b = vec![Scalar::from(3u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(2, &a, &b),
        Scalar::from(8u32) // This is (2-1)(3-1) + (2)(3)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        Scalar::from(2u32) // This is (2-1)(3-1)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        Scalar::from(0u32)
    );
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_3_variables() {
    let a = vec![Scalar::from(2u8), Scalar::from(5u8), Scalar::from(7u8)];
    let b = vec![Scalar::from(3u8), Scalar::from(11u8), Scalar::from(13u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(8, &a, &b),
        Scalar::from(123880u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(7, &a, &b),
        Scalar::from(93850u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(6, &a, &b),
        Scalar::from(83840u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(5, &a, &b),
        Scalar::from(62000u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(4, &a, &b),
        Scalar::from(54720u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(3, &a, &b),
        Scalar::from(30960u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(2, &a, &b),
        Scalar::from(23040u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        Scalar::from(5760u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        Scalar::from(0u32)
    );
}

#[test]
fn compute_truncated_lagrange_basis_sum_matches_sum_of_result_from_compute_evaluation_vector() {
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(2, 10);
    for _ in 0..20 {
        let variables = dist.sample(&mut rng);
        let length = Uniform::new((1 << (variables - 1)) + 1, 1 << variables).sample(&mut rng);
        let point: Vec<_> = iter::repeat_with(|| Scalar::random(&mut rng))
            .take(variables)
            .collect();
        let mut eval_vec = vec![Scalar::zero(); length];
        compute_evaluation_vector(&mut eval_vec, &point);
        // ---------------- This is the actual test --------------------
        assert_eq!(
            compute_truncated_lagrange_basis_sum(length, &point),
            eval_vec.iter().sum()
        );
        // -----------------------------------------------------------
    }
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_matches_inner_product_of_results_compute_evaluation_vector(
) {
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(2, 10);
    for _ in 0..20 {
        let variables = dist.sample(&mut rng);
        let length = Uniform::new((1 << (variables - 1)) + 1, 1 << variables).sample(&mut rng);
        let a: Vec<_> = iter::repeat_with(|| Scalar::random(&mut rng))
            .take(variables)
            .collect();
        let b: Vec<_> = iter::repeat_with(|| Scalar::random(&mut rng))
            .take(variables)
            .collect();
        let mut eval_vec_a = vec![Scalar::zero(); length];
        let mut eval_vec_b = vec![Scalar::zero(); length];
        compute_evaluation_vector(&mut eval_vec_a, &a);
        compute_evaluation_vector(&mut eval_vec_b, &b);
        // ---------------- This is the actual test --------------------
        assert_eq!(
            compute_truncated_lagrange_basis_inner_product(length, &a, &b),
            eval_vec_a.iter().zip(eval_vec_b).map(|(x, y)| x * y).sum()
        );
        // -----------------------------------------------------------
    }
}
