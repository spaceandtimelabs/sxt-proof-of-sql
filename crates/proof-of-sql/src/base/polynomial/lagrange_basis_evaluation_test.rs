use crate::base::{
    polynomial::{
        compute_evaluation_vector, compute_rho_eval,
        compute_truncated_lagrange_basis_inner_product, compute_truncated_lagrange_basis_sum,
    },
    scalar::test_scalar::TestScalar,
};
use ark_std::UniformRand;
use core::iter;
use num_traits::Zero;

#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_0_variables() {
    let point: Vec<TestScalar> = vec![];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        TestScalar::from(1u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        TestScalar::from(0u8)
    );
}
#[test]
fn compute_rho_eval_gives_correct_values_with_0_variables() {
    let point: Vec<TestScalar> = vec![];
    assert_eq!(compute_rho_eval(1, &point), TestScalar::from(0u8));
    assert_eq!(compute_rho_eval(0, &point), TestScalar::from(0u8));
}
#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_1_variables() {
    let point: Vec<TestScalar> = vec![TestScalar::from(2u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(2, &point),
        TestScalar::from(1u8) // This is (1-2) + (2)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        -TestScalar::from(1u8) // This is (1-2)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        TestScalar::from(0u8)
    );
}
#[test]
fn compute_rho_eval_gives_correct_values_with_1_variables() {
    let point: Vec<TestScalar> = vec![TestScalar::from(2u8)];
    assert_eq!(
        compute_rho_eval(2, &point),
        TestScalar::from(2u8) // This is 0 * (1-2) + 1 * (2)
    );
    assert_eq!(
        compute_rho_eval(1, &point),
        -TestScalar::from(0u8) // This is 0 * (1-2)
    );
    assert_eq!(compute_rho_eval(0, &point), TestScalar::from(0u8));
}
#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_2_variables() {
    let point = vec![TestScalar::from(2u8), TestScalar::from(5u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(4, &point),
        TestScalar::from(1u8) // This is (1-2)(1-5)+(2)(1-5)+(1-2)(5)+(2)(5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(3, &point),
        -TestScalar::from(9u8) // This is (1-2)(1-5)+(2)(1-5)+(1-2)(5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(2, &point),
        -TestScalar::from(4u8) // This is (1-2)(1-5)+(2)(1-5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        TestScalar::from(4u8) // This is (1-2)(1-5)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        TestScalar::from(0u8)
    );
}
#[test]
fn compute_rho_eval_gives_correct_values_with_2_variables() {
    let point = vec![TestScalar::from(2u8), TestScalar::from(5u8)];
    assert_eq!(
        compute_rho_eval(4, &point),
        TestScalar::from(12u8) // This is 0 * (1-2)(1-5) + 1 * (2)(1-5) + 2 * (1-2)(5) + 3 * (2)(5)
    );
    assert_eq!(
        compute_rho_eval(3, &point),
        -TestScalar::from(18u8) // This is 0 * (1-2)(1-5) + 1 * (2)(1-5) + 2 * (1-2)(5)
    );
    assert_eq!(
        compute_rho_eval(2, &point),
        -TestScalar::from(8u8) // This is 0 * (1-2)(1-5) + 1 * (2)(1-5)
    );
    assert_eq!(
        compute_rho_eval(1, &point),
        TestScalar::from(0u8) // This is 0 * (1-2)(1-5)
    );
    assert_eq!(compute_rho_eval(0, &point), TestScalar::from(0u8));
}

#[test]
fn compute_truncated_lagrange_basis_sum_gives_correct_values_with_3_variables() {
    let point = vec![
        TestScalar::from(2u8),
        TestScalar::from(5u8),
        TestScalar::from(7u8),
    ];
    assert_eq!(
        compute_truncated_lagrange_basis_sum(8, &point),
        TestScalar::from(1u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(7, &point),
        -TestScalar::from(69u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(6, &point),
        -TestScalar::from(34u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(5, &point),
        TestScalar::from(22u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(4, &point),
        -TestScalar::from(6u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(3, &point),
        TestScalar::from(54u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(2, &point),
        TestScalar::from(24u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(1, &point),
        -TestScalar::from(24u8)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_sum(0, &point),
        TestScalar::from(0u8)
    );
}

#[test]
fn compute_rho_eval_gives_correct_values_with_3_variables() {
    let point = vec![
        TestScalar::from(2u8),
        TestScalar::from(5u8),
        TestScalar::from(7u8),
    ];
    assert_eq!(compute_rho_eval(8, &point), TestScalar::from(40u8));
    assert_eq!(compute_rho_eval(7, &point), -TestScalar::from(450u16));
    assert_eq!(compute_rho_eval(6, &point), -TestScalar::from(240u8));
    assert_eq!(compute_rho_eval(5, &point), TestScalar::from(40u8));
    assert_eq!(compute_rho_eval(4, &point), -TestScalar::from(72u8));
    assert_eq!(compute_rho_eval(3, &point), TestScalar::from(108u8));
    assert_eq!(compute_rho_eval(2, &point), TestScalar::from(48u8));
    assert_eq!(compute_rho_eval(1, &point), TestScalar::from(0u8));
    assert_eq!(compute_rho_eval(0, &point), TestScalar::from(0u8));
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
    let a: Vec<TestScalar> = vec![];
    let b = vec![];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        TestScalar::from(1u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        TestScalar::from(0u32)
    );
}
#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_1_variables() {
    let a = vec![TestScalar::from(2u8)];
    let b = vec![TestScalar::from(3u8)];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(2, &a, &b),
        TestScalar::from(8u32) // This is (2-1)(3-1) + (2)(3)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        TestScalar::from(2u32) // This is (2-1)(3-1)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        TestScalar::from(0u32)
    );
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_3_variables() {
    let a = vec![
        TestScalar::from(2u8),
        TestScalar::from(5u8),
        TestScalar::from(7u8),
    ];
    let b = vec![
        TestScalar::from(3u8),
        TestScalar::from(11u8),
        TestScalar::from(13u8),
    ];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(8, &a, &b),
        TestScalar::from(123_880_u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(7, &a, &b),
        TestScalar::from(93850u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(6, &a, &b),
        TestScalar::from(83840u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(5, &a, &b),
        TestScalar::from(62000u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(4, &a, &b),
        TestScalar::from(54720u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(3, &a, &b),
        TestScalar::from(30960u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(2, &a, &b),
        TestScalar::from(23040u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        TestScalar::from(5760u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        TestScalar::from(0u32)
    );
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_3_variables_using_dalek_scalar(
) {
    let a = vec![
        TestScalar::from(2u8),
        TestScalar::from(5u8),
        TestScalar::from(7u8),
    ];
    let b = vec![
        TestScalar::from(3u8),
        TestScalar::from(11u8),
        TestScalar::from(13u8),
    ];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(8, &a, &b),
        TestScalar::from(123_880_u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(7, &a, &b),
        TestScalar::from(93850u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(6, &a, &b),
        TestScalar::from(83840u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(5, &a, &b),
        TestScalar::from(62000u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(4, &a, &b),
        TestScalar::from(54720u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(3, &a, &b),
        TestScalar::from(30960u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(2, &a, &b),
        TestScalar::from(23040u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(1, &a, &b),
        TestScalar::from(5760u32)
    );
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(0, &a, &b),
        TestScalar::from(0u32)
    );
}

#[test]
fn compute_truncated_lagrange_basis_inner_product_gives_correct_values_with_3_variables_using_i32()
{
    let a: Vec<i32> = vec![2, 5, 7];
    let b: Vec<i32> = vec![3, 11, 13];
    assert_eq!(
        compute_truncated_lagrange_basis_inner_product(8, &a, &b),
        123_880
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
        let point: Vec<_> = iter::repeat_with(|| TestScalar::rand(&mut rng))
            .take(variables)
            .collect();
        let mut eval_vec = vec![TestScalar::zero(); length];
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
        let a: Vec<_> = iter::repeat_with(|| TestScalar::rand(&mut rng))
            .take(variables)
            .collect();
        let b: Vec<_> = iter::repeat_with(|| TestScalar::rand(&mut rng))
            .take(variables)
            .collect();
        let mut eval_vec_a = vec![TestScalar::zero(); length];
        let mut eval_vec_b = vec![TestScalar::zero(); length];
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
