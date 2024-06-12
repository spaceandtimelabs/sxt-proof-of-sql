/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::base::polynomial::interpolate::*;
use crate::base::scalar::Curve25519Scalar;
use ark_std::UniformRand;
use num_traits::Zero;

#[test]
fn test_interpolate_uni_poly_for_random_polynomials() {
    let mut prng = ark_std::test_rng();

    let num_points = vec![0, 1, 2, 3, 4, 5, 10, 20, 32, 33, 64, 65];

    for n in num_points {
        let poly = std::iter::repeat_with(|| Curve25519Scalar::rand(&mut prng))
            .take(n)
            .collect::<Vec<_>>();
        let evals = (0..n)
            .map(|i| {
                let x: Curve25519Scalar = (i as u64).into();
                poly.iter().fold(Zero::zero(), |acc, &c| acc * x + c)
            })
            .collect::<Vec<_>>();
        let query = Curve25519Scalar::rand(&mut prng);
        let value = interpolate_uni_poly(&evals, query);
        let expected_value = poly
            .iter()
            .fold(Curve25519Scalar::zero(), |acc, &c| acc * query + c);
        assert_eq!(value, expected_value);
    }
}

#[test]
fn interpolate_uni_poly_gives_zero_for_no_evaluations() {
    let evaluations = vec![];
    assert_eq!(
        interpolate_uni_poly(&evaluations, Curve25519Scalar::from(10)),
        Curve25519Scalar::from(0)
    );
    assert_eq!(
        interpolate_uni_poly(&evaluations, Curve25519Scalar::from(100)),
        Curve25519Scalar::from(0)
    );
}

#[test]
fn interpolate_uni_poly_gives_constant_for_degree_0_polynomial() {
    let evaluations = vec![Curve25519Scalar::from(77)];
    assert_eq!(
        interpolate_uni_poly(&evaluations, Curve25519Scalar::from(10)),
        Curve25519Scalar::from(77)
    );
    assert_eq!(
        interpolate_uni_poly(&evaluations, Curve25519Scalar::from(100)),
        Curve25519Scalar::from(77)
    );
}

#[test]
fn interpolate_uni_poly_gives_correct_result_for_linear_polynomial() {
    let evaluations = vec![
        Curve25519Scalar::from(2),
        Curve25519Scalar::from(3),
        Curve25519Scalar::from(4),
    ];
    assert_eq!(
        interpolate_uni_poly(&evaluations, Curve25519Scalar::from(10)),
        Curve25519Scalar::from(12)
    );
    assert_eq!(
        interpolate_uni_poly(&evaluations, Curve25519Scalar::from(100)),
        Curve25519Scalar::from(102)
    );
}

#[test]
fn interpolate_uni_poly_gives_correct_value_for_known_evaluation() {
    let evaluations = vec![
        Curve25519Scalar::from(777),
        Curve25519Scalar::from(123),
        Curve25519Scalar::from(2357),
        Curve25519Scalar::from(1),
        Curve25519Scalar::from(2),
        Curve25519Scalar::from(3),
    ];
    for i in 0..evaluations.len() {
        assert_eq!(
            interpolate_uni_poly(&evaluations, Curve25519Scalar::from(i as u32)),
            evaluations[i]
        );
    }
}
