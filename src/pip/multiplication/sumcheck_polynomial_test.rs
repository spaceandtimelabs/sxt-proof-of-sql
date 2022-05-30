use crate::pip::multiplication::sumcheck_polynomial::*;

use curve25519_dalek::scalar::Scalar;

#[test]
fn test_poly1() {
    let one = Scalar::from(1u64);
    let zero = Scalar::from(0u64);
    let a_vec = [
        Scalar::from(3u64),
        Scalar::from(7u64),
    ];
    let b_vec = [
        Scalar::from(2u64),
        Scalar::from(4u64),
    ];
    let ab_vec = [
        Scalar::from(6u64),
        Scalar::from(28u64),
    ];
    let r_vec = [
        Scalar::from(62345u64),
        Scalar::from(234234u64),
    ];
    let p = make_sumcheck_polynomial(1, &a_vec, &b_vec, &ab_vec, &r_vec);
    let sum = p.evaluate(&[zero]) + p.evaluate(&[one]);
    assert_eq!(sum, zero);

    let not_ab_vec = [
        Scalar::from(3u64),
        Scalar::from(28u64),
    ];
    let p = make_sumcheck_polynomial(1, &a_vec, &b_vec, &not_ab_vec, &r_vec);
    let sum = p.evaluate(&[zero]) + p.evaluate(&[one]);
    assert_ne!(sum, zero);

    let not_ab_vec = [
        Scalar::from(6u64),
        Scalar::from(21u64),
    ];
    let p = make_sumcheck_polynomial(1, &a_vec, &b_vec, &not_ab_vec, &r_vec);
    let sum = p.evaluate(&[zero]) + p.evaluate(&[one]);
    assert_ne!(sum, zero);
}
