use crate::base::{
    polynomial::{
        compute_evaluation_vector, compute_truncated_lagrange_basis_inner_product,
        compute_truncated_lagrange_basis_sum,
    },
    scalar::ArkScalar,
};
use num_traits::Zero;
use std::iter;

#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_0_variables() {
    let point: Vec<ArkScalar> = vec![];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        ArkScalar::from(1u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        ArkScalar::from(0u8)
    );
}
#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_1_variables() {
    let point: Vec<ArkScalar> = vec![ArkScalar::from(2u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(2, &point),
        ArkScalar::from(1u8) // This is (1-2) + (2)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        -ArkScalar::from(1u8) // This is (1-2)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        ArkScalar::from(0u8)
    );
}
#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_2_variables() {
    let point = vec![ArkScalar::from(2u8), ArkScalar::from(5u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(4, &point),
        ArkScalar::from(1u8) // This is (1-2)(1-5)+(2)(1-5)+(1-2)(5)+(2)(5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(3, &point),
        -ArkScalar::from(9u8) // This is (1-2)(1-5)+(2)(1-5)+(1-2)(5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(2, &point),
        -ArkScalar::from(4u8) // This is (1-2)(1-5)+(2)(1-5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        ArkScalar::from(4u8) // This is (1-2)(1-5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        ArkScalar::from(0u8)
    );
}

#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_3_variables() {
    let point = vec![
        ArkScalar::from(2u8),
        ArkScalar::from(5u8),
        ArkScalar::from(7u8),
    ];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(8, &point),
        ArkScalar::from(1u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(7, &point),
        -ArkScalar::from(69u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(6, &point),
        -ArkScalar::from(34u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(5, &point),
        ArkScalar::from(22u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(4, &point),
        -ArkScalar::from(6u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(3, &point),
        ArkScalar::from(54u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(2, &point),
        ArkScalar::from(24u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        -ArkScalar::from(24u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        ArkScalar::from(0u8)
    );
}

#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_3_variables_using_dalek_scalar() {
    let point = vec![
        ArkScalar::from(2u8),
        ArkScalar::from(5u8),
        ArkScalar::from(7u8),
    ];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(8, &point),
        ArkScalar::from(1u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(7, &point),
        -ArkScalar::from(69u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(6, &point),
        -ArkScalar::from(34u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(5, &point),
        ArkScalar::from(22u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(4, &point),
        -ArkScalar::from(6u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(3, &point),
        ArkScalar::from(54u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(2, &point),
        ArkScalar::from(24u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        -ArkScalar::from(24u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        ArkScalar::from(0u8)
    );
}

#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_3_variables_using_i32() {
    let point: Vec<i32> = vec![2, 5, 7];
    assert_eq!(compute_truncated_lagrange_basis_sum(8, &point), 1);
    assert_eq!(compute_truncated_lagrange_basis_sum(7, &point), -69);
    assert_eq!(compute_truncated_lagrange_basis_sum(6, &point), -34);
    assert_eq!(compute_truncated_lagrange_basis_sum(5, &point), 22);
    assert_eq!(compute_truncated_lagrange_basis_sum(4, &point), -6);
    assert_eq!(compute_truncated_lagrange_basis_sum(3, &point), 54);
    assert_eq!(compute_truncated_lagrange_basis_sum(2, &point), 24);
    assert_eq!(compute_truncated_lagrange_basis_sum(1, &point), -24);
    assert_eq!(compute_truncated_lagrange_basis_sum(0, &point), 0);
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_0_variables() {
    let a: Vec<ArkScalar> = vec![];
    let b = vec![];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        ArkScalar::from(1u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        ArkScalar::from(0u32)
    );
}
#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_1_variables() {
    let a = vec![ArkScalar::from(2u8)];
    let b = vec![ArkScalar::from(3u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(2, &a, &b),
        ArkScalar::from(8u32) // This is (2-1)(3-1) + (2)(3)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        ArkScalar::from(2u32) // This is (2-1)(3-1)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        ArkScalar::from(0u32)
    );
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_3_variables() {
    let a = vec![
        ArkScalar::from(2u8),
        ArkScalar::from(5u8),
        ArkScalar::from(7u8),
    ];
    let b = vec![
        ArkScalar::from(3u8),
        ArkScalar::from(11u8),
        ArkScalar::from(13u8),
    ];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(8, &a, &b),
        ArkScalar::from(123880u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(7, &a, &b),
        ArkScalar::from(93850u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(6, &a, &b),
        ArkScalar::from(83840u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(5, &a, &b),
        ArkScalar::from(62000u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(4, &a, &b),
        ArkScalar::from(54720u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(3, &a, &b),
        ArkScalar::from(30960u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(2, &a, &b),
        ArkScalar::from(23040u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        ArkScalar::from(5760u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        ArkScalar::from(0u32)
    );
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_3_variables_using_dalek_scalar(
) {
    let a = vec![
        ArkScalar::from(2u8),
        ArkScalar::from(5u8),
        ArkScalar::from(7u8),
    ];
    let b = vec![
        ArkScalar::from(3u8),
        ArkScalar::from(11u8),
        ArkScalar::from(13u8),
    ];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(8, &a, &b),
        ArkScalar::from(123880u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(7, &a, &b),
        ArkScalar::from(93850u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(6, &a, &b),
        ArkScalar::from(83840u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(5, &a, &b),
        ArkScalar::from(62000u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(4, &a, &b),
        ArkScalar::from(54720u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(3, &a, &b),
        ArkScalar::from(30960u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(2, &a, &b),
        ArkScalar::from(23040u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        ArkScalar::from(5760u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        ArkScalar::from(0u32)
    );
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_3_variables_using_i32()
{
    let a: Vec<i32> = vec![2, 5, 7];
    let b: Vec<i32> = vec![3, 11, 13];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(8, &a, &b),
        123880
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(7, &a, &b),
        93850
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(6, &a, &b),
        83840
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(5, &a, &b),
        62000
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(4, &a, &b),
        54720
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(3, &a, &b),
        30960
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(2, &a, &b),
        23040
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        5760
    );
    assert_eq!(compute_truncated_lagrange_basis_inner_product(0, &a, &b), 0);
}

#[test]
fn compute_truncated_lagrange_basis_sum_matches_sum_of_result_from_compute_evaluation_vector() {
    use ark_std::rand::{
        distributions::{Distribution, Uniform},
        rngs::StdRng,
        SeedableRng,
    };

    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(2, 10);
    for _ in 0..20 {
        let variables = dist.sample(&mut rng);
        let length = Uniform::new((1 << (variables - 1)) + 1, 1 << variables).sample(&mut rng);
        let point: Vec<_> = iter::repeat_with(|| ArkScalar::rand(&mut rng))
            .take(variables)
            .collect();
        let mut eval_vec = vec![ArkScalar::zero(); length];
        compute_evaluation_vector(&mut eval_vec, &point);
        // ---------------- This is the actual test --------------------
        assert_eq!(
            compute_truncated_lagrange_basis_sum(length, &point),
            eval_vec.into_iter().sum()
        );
        // -----------------------------------------------------------
    }
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_matches_inner_product_of_results_compute_evaluation_vector(
) {
    use ark_std::rand::{
        distributions::{Distribution, Uniform},
        rngs::StdRng,
        SeedableRng,
    };

    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(2, 10);
    for _ in 0..20 {
        let variables = dist.sample(&mut rng);
        let length = Uniform::new((1 << (variables - 1)) + 1, 1 << variables).sample(&mut rng);
        let a: Vec<_> = iter::repeat_with(|| ArkScalar::rand(&mut rng))
            .take(variables)
            .collect();
        let b: Vec<_> = iter::repeat_with(|| ArkScalar::rand(&mut rng))
            .take(variables)
            .collect();
        let mut eval_vec_a = vec![ArkScalar::zero(); length];
        let mut eval_vec_b = vec![ArkScalar::zero(); length];
        compute_evaluation_vector(&mut eval_vec_a, &a);
        compute_evaluation_vector(&mut eval_vec_b, &b);
        // ---------------- This is the actual test --------------------
        assert_eq!(
            compute_truncated_lagrange_basis_inner_product(length, &a, &b),
            eval_vec_a
                .into_iter()
                .zip(eval_vec_b.into_iter())
                .map(|(x, y)| x * y)
                .sum()
        );
        // -----------------------------------------------------------
    }
}
