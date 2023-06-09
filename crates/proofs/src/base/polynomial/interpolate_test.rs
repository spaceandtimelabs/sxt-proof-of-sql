/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::base::polynomial::interpolate::*;
use crate::base::polynomial::ArkScalar;
use num_traits::Zero;

#[test]
fn test_interpolate_uni_poly_for_random_polynomials() {
    let mut prng = ark_std::test_rng();

    let num_points = vec![0, 1, 2, 3, 4, 5, 10, 20, 32, 33, 64, 65];

    for n in num_points {
        let poly = std::iter::repeat_with(|| ArkScalar::rand(&mut prng))
            .take(n)
            .collect::<Vec<_>>();
        let evals = (0..n)
            .map(|i| {
                let x: ArkScalar = (i as u64).into();
                poly.iter().fold(Zero::zero(), |acc, &c| acc * x + c)
            })
            .collect::<Vec<_>>();
        let query = ArkScalar::rand(&mut prng);
        let value = interpolate_uni_poly(&evals, query);
        let expected_value = poly
            .iter()
            .fold(ArkScalar::zero(), |acc, &c| acc * query + c);
        assert_eq!(value, expected_value);
    }
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
