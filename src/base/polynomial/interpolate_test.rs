use crate::base::polynomial::interpolate::*;

use ark_poly::univariate::DensePolynomial;
use ark_poly::DenseUVPolynomial;
use ark_poly::Polynomial;
use ark_std::vec::Vec;
use ark_std::UniformRand;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::{from_ark_scalar, ArkScalar};

#[test]
fn test_interpolate_uni_poly() {
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
}
