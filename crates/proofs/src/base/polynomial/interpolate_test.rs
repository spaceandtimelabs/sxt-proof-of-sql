/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::base::polynomial::interpolate::*;

use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_poly::Polynomial;
use ark_std::vec::Vec;
use ark_std::UniformRand;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::{from_ark_scalar, ArkScalar};

#[test]
fn test_interpolate_uni_poly_for_random_polynomials() {
    let mut prng = ark_std::test_rng();

    // test a polynomial with 20 known points, i.e., with degree 19
    let poly = DensePolynomial::<ArkScalar>::rand(20 - 1, &mut prng);
    let evals = (0..20)
        .map(|i| from_ark_scalar(&poly.evaluate(&ArkScalar::from(i as u64))))
        .collect::<Vec<Scalar>>();
    let query = ArkScalar::rand(&mut prng);
    let value = interpolate_uni_poly(&evals, from_ark_scalar(&query));
    let expected_value = from_ark_scalar(&poly.evaluate(&query));
    assert_eq!(value, expected_value);

    // test a polynomial with 33 known points, i.e., with degree 32
    let poly = DensePolynomial::<ArkScalar>::rand(33 - 1, &mut prng);
    let evals = (0..33)
        .map(|i| from_ark_scalar(&poly.evaluate(&ArkScalar::from(i as u64))))
        .collect::<Vec<Scalar>>();
    let query = ArkScalar::rand(&mut prng);
    let value = interpolate_uni_poly(&evals, from_ark_scalar(&query));
    let expected_value = from_ark_scalar(&poly.evaluate(&query));
    assert_eq!(value, expected_value);

    // test a polynomial with 64 known points, i.e., with degree 63
    let poly = DensePolynomial::<ArkScalar>::rand(64 - 1, &mut prng);
    let evals = (0..64)
        .map(|i| from_ark_scalar(&poly.evaluate(&ArkScalar::from(i as u64))))
        .collect::<Vec<Scalar>>();
    let query = ArkScalar::rand(&mut prng);
    let value = interpolate_uni_poly(&evals, from_ark_scalar(&query));
    let expected_value = from_ark_scalar(&poly.evaluate(&query));
    assert_eq!(value, expected_value);
}

#[test]
fn interpolate_uni_poly_gives_zero_for_no_evaluations() {
    let evaluations = vec![];
    assert_eq!(
        interpolate_uni_poly(&evaluations, ArkScalar::from(10)),
        ArkScalar::from(0)
    );
    assert_eq!(
        interpolate_uni_poly(&evaluations, ArkScalar::from(100)),
        ArkScalar::from(0)
    );
}

#[test]
fn interpolate_uni_poly_gives_constant_for_degree_0_polynomial() {
    let evaluations = vec![ArkScalar::from(77)];
    assert_eq!(
        interpolate_uni_poly(&evaluations, ArkScalar::from(10)),
        ArkScalar::from(77)
    );
    assert_eq!(
        interpolate_uni_poly(&evaluations, ArkScalar::from(100)),
        ArkScalar::from(77)
    );
}

#[test]
fn interpolate_uni_poly_gives_correct_result_for_linear_polynomial() {
    let evaluations = vec![ArkScalar::from(2), ArkScalar::from(3), ArkScalar::from(4)];
    assert_eq!(
        interpolate_uni_poly(&evaluations, ArkScalar::from(10)),
        ArkScalar::from(12)
    );
    assert_eq!(
        interpolate_uni_poly(&evaluations, ArkScalar::from(100)),
        ArkScalar::from(102)
    );
}

#[test]
fn interpolate_uni_poly_gives_correct_value_for_known_evaluation() {
    let evaluations = vec![
        ArkScalar::from(777),
        ArkScalar::from(123),
        ArkScalar::from(2357),
        ArkScalar::from(1),
        ArkScalar::from(2),
        ArkScalar::from(3),
    ];
    for i in 0..evaluations.len() {
        assert_eq!(
            interpolate_uni_poly(&evaluations, ArkScalar::from(i as u32)),
            evaluations[i]
        );
    }
}
