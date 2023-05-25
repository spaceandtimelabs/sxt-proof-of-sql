use super::compute_evaluation_vector;

use crate::base::polynomial::ArkScalar;
use crate::base::slice_ops;
use ark_poly::MultilinearExtension;

#[test]
fn we_compute_the_correct_evaluation_vector_for_a_small_example() {
    let mut v = [ArkScalar::zero(); 2];
    compute_evaluation_vector(&mut v, &[ArkScalar::from(3u64)]);
    let expected_v = [
        ArkScalar::one() - ArkScalar::from(3u64),
        ArkScalar::from(3u64),
    ];
    assert_eq!(v, expected_v);

    let mut v = [ArkScalar::zero(); 4];
    compute_evaluation_vector(&mut v, &[ArkScalar::from(3u64), ArkScalar::from(4u64)]);
    let expected_v = [
        (ArkScalar::one() - ArkScalar::from(4u64)) * (ArkScalar::one() - ArkScalar::from(3u64)),
        (ArkScalar::one() - ArkScalar::from(4u64)) * ArkScalar::from(3u64),
        ArkScalar::from(4u64) * (ArkScalar::one() - ArkScalar::from(3u64)),
        ArkScalar::from(4u64) * ArkScalar::from(3u64),
    ];
    assert_eq!(v, expected_v);
}

#[test]
fn we_compute_the_evaluation_vectors_not_a_power_of_2() {
    let mut v = [ArkScalar::zero(); 1];
    compute_evaluation_vector(&mut v, &[ArkScalar::from(3u64)]);
    let expected_v = [ArkScalar::one() - ArkScalar::from(3u64)];
    assert_eq!(v, expected_v);

    let mut v = [ArkScalar::zero(); 3];
    compute_evaluation_vector(&mut v, &[ArkScalar::from(3u64), ArkScalar::from(4u64)]);
    let expected_v = [
        (ArkScalar::one() - ArkScalar::from(4u64)) * (ArkScalar::one() - ArkScalar::from(3u64)),
        (ArkScalar::one() - ArkScalar::from(4u64)) * ArkScalar::from(3u64),
        ArkScalar::from(4u64) * (ArkScalar::one() - ArkScalar::from(3u64)),
    ];
    assert_eq!(v, expected_v);
}

#[test]
fn we_get_the_same_result_using_evaluation_vector_as_direct_evaluation() {
    let xs = [
        ArkScalar::from(3u64),
        ArkScalar::from(7u64),
        ArkScalar::from(2u64),
        ArkScalar::from(9u64),
        ArkScalar::from(21u64),
        ArkScalar::from(10u64),
        ArkScalar::from(5u64),
        ArkScalar::from(92u64),
    ];
    let point = [
        ArkScalar::from(81u64),
        ArkScalar::from(33u64),
        ArkScalar::from(22u64),
    ];
    let mut v = [ArkScalar::zero(); 8];
    compute_evaluation_vector(&mut v, &point);
    let eval = slice_ops::inner_product(&xs, &v);

    let poly = ark_poly::DenseMultilinearExtension::from_evaluations_slice(
        3,
        &ArkScalar::unwrap_slice(&xs),
    );
    let expected_eval = ArkScalar(poly.evaluate(&ArkScalar::unwrap_slice(&point)).unwrap());
    assert_eq!(eval, expected_eval);
}
