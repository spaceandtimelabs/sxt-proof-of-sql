use super::compute_evaluation_vector;
use crate::base::{scalar::Curve25519Scalar, slice_ops};
use ark_poly::MultilinearExtension;
use num_traits::{One, Zero};

#[test]
fn we_compute_the_correct_evaluation_vector_for_a_small_example() {
    let mut v = [Curve25519Scalar::zero(); 2];
    compute_evaluation_vector(&mut v, &[Curve25519Scalar::from(3u64)]);
    let expected_v = [
        Curve25519Scalar::one() - Curve25519Scalar::from(3u64),
        Curve25519Scalar::from(3u64),
    ];
    assert_eq!(v, expected_v);

    let mut v = [Curve25519Scalar::zero(); 4];
    compute_evaluation_vector(
        &mut v,
        &[Curve25519Scalar::from(3u64), Curve25519Scalar::from(4u64)],
    );
    let expected_v = [
        (Curve25519Scalar::one() - Curve25519Scalar::from(4u64))
            * (Curve25519Scalar::one() - Curve25519Scalar::from(3u64)),
        (Curve25519Scalar::one() - Curve25519Scalar::from(4u64)) * Curve25519Scalar::from(3u64),
        Curve25519Scalar::from(4u64) * (Curve25519Scalar::one() - Curve25519Scalar::from(3u64)),
        Curve25519Scalar::from(4u64) * Curve25519Scalar::from(3u64),
    ];
    assert_eq!(v, expected_v);
}

#[test]
fn we_compute_the_evaluation_vectors_not_a_power_of_2() {
    let mut v = [Curve25519Scalar::zero(); 1];
    compute_evaluation_vector(&mut v, &[Curve25519Scalar::from(3u64)]);
    let expected_v = [Curve25519Scalar::one() - Curve25519Scalar::from(3u64)];
    assert_eq!(v, expected_v);

    let mut v = [Curve25519Scalar::zero(); 3];
    compute_evaluation_vector(
        &mut v,
        &[Curve25519Scalar::from(3u64), Curve25519Scalar::from(4u64)],
    );
    let expected_v = [
        (Curve25519Scalar::one() - Curve25519Scalar::from(4u64))
            * (Curve25519Scalar::one() - Curve25519Scalar::from(3u64)),
        (Curve25519Scalar::one() - Curve25519Scalar::from(4u64)) * Curve25519Scalar::from(3u64),
        Curve25519Scalar::from(4u64) * (Curve25519Scalar::one() - Curve25519Scalar::from(3u64)),
    ];
    assert_eq!(v, expected_v);
}
#[test]
fn we_compute_the_evaluation_vectors_of_any_length() {
    let mut full_vec = [Curve25519Scalar::zero(); 16];
    let evaluation_point = [
        Curve25519Scalar::from(2u64),
        Curve25519Scalar::from(3u64),
        Curve25519Scalar::from(5u64),
        Curve25519Scalar::from(7u64),
    ];
    compute_evaluation_vector(&mut full_vec, &evaluation_point);
    for i in 0..16 {
        let mut v = vec![Curve25519Scalar::zero(); i];
        compute_evaluation_vector(&mut v, &evaluation_point);
        assert_eq!(v, &full_vec[..i]);
    }
}

#[test]
fn we_compute_the_evaluation_vector_for_an_empty_point() {
    let mut v = [Curve25519Scalar::zero(); 1];
    compute_evaluation_vector(&mut v, &[]);
    let expected_v = [Curve25519Scalar::one()];
    assert_eq!(v, expected_v);
}

#[test]
fn we_get_the_same_result_using_evaluation_vector_as_direct_evaluation() {
    let xs = [
        Curve25519Scalar::from(3u64),
        Curve25519Scalar::from(7u64),
        Curve25519Scalar::from(2u64),
        Curve25519Scalar::from(9u64),
        Curve25519Scalar::from(21u64),
        Curve25519Scalar::from(10u64),
        Curve25519Scalar::from(5u64),
        Curve25519Scalar::from(92u64),
    ];
    let point = [
        Curve25519Scalar::from(81u64),
        Curve25519Scalar::from(33u64),
        Curve25519Scalar::from(22u64),
    ];
    let mut v = [Curve25519Scalar::zero(); 8];
    compute_evaluation_vector(&mut v, &point);
    let eval = slice_ops::inner_product(&xs, &v);

    let poly = ark_poly::DenseMultilinearExtension::from_evaluations_slice(
        3,
        &Curve25519Scalar::unwrap_slice(&xs),
    );
    let expected_eval = Curve25519Scalar::new(
        poly.evaluate(&Curve25519Scalar::unwrap_slice(&point))
            .unwrap(),
    );
    assert_eq!(eval, expected_eval);
}
