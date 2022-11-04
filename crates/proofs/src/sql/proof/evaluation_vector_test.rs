use super::compute_evaluation_vector;

use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::DenseMultilinearExtension;
use crate::base::scalar::inner_product;

#[test]
fn we_compute_the_correct_evaluation_vector_for_a_small_example() {
    let v = compute_evaluation_vector(&[Scalar::from(3u64)]);
    let expected_v = vec![Scalar::one() - Scalar::from(3u64), Scalar::from(3u64)];
    assert_eq!(v, expected_v);

    let v = compute_evaluation_vector(&[Scalar::from(3u64), Scalar::from(4u64)]);
    let expected_v = vec![
        (Scalar::one() - Scalar::from(4u64)) * (Scalar::one() - Scalar::from(3u64)),
        (Scalar::one() - Scalar::from(4u64)) * Scalar::from(3u64),
        Scalar::from(4u64) * (Scalar::one() - Scalar::from(3u64)),
        Scalar::from(4u64) * Scalar::from(3u64),
    ];
    assert_eq!(v, expected_v);
}

#[test]
fn we_get_the_same_result_using_evaluation_vector_as_direct_evaluation() {
    let xs = [
        Scalar::from(3u64),
        Scalar::from(7u64),
        Scalar::from(2u64),
        Scalar::from(9u64),
        Scalar::from(21u64),
        Scalar::from(10u64),
        Scalar::from(5u64),
        Scalar::from(92u64),
    ];
    let point = [
        Scalar::from(81u64),
        Scalar::from(33u64),
        Scalar::from(22u64),
    ];
    let v = compute_evaluation_vector(&point);
    let eval = inner_product(&xs, &v);

    let poly = DenseMultilinearExtension::from_evaluations_slice(3, &xs);
    let expected_eval = poly.evaluate(&point).unwrap();
    assert_eq!(eval, expected_eval);
}
