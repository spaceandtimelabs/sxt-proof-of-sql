use super::CompositePolynomial;
use crate::base::scalar::Curve25519Scalar;
use alloc::rc::Rc;

#[test]
fn test_composite_polynomial_evaluation() {
    let a: Vec<Curve25519Scalar> = vec![
        -Curve25519Scalar::from(7u32),
        Curve25519Scalar::from(2u32),
        -Curve25519Scalar::from(6u32),
        Curve25519Scalar::from(17u32),
    ];
    let b: Vec<Curve25519Scalar> = vec![
        Curve25519Scalar::from(2u32),
        -Curve25519Scalar::from(8u32),
        Curve25519Scalar::from(4u32),
        Curve25519Scalar::from(1u32),
    ];
    let c: Vec<Curve25519Scalar> = vec![
        Curve25519Scalar::from(1u32),
        Curve25519Scalar::from(3u32),
        -Curve25519Scalar::from(5u32),
        -Curve25519Scalar::from(9u32),
    ];
    let mut prod = CompositePolynomial::new(2);
    prod.add_product([Rc::new(a), Rc::new(b)], Curve25519Scalar::from(3u32));
    prod.add_product([Rc::new(c)], Curve25519Scalar::from(2u32));
    let prod00 = prod.evaluate(&[Curve25519Scalar::from(0u32), Curve25519Scalar::from(0u32)]);
    let prod10 = prod.evaluate(&[Curve25519Scalar::from(1u32), Curve25519Scalar::from(0u32)]);
    let prod01 = prod.evaluate(&[Curve25519Scalar::from(0u32), Curve25519Scalar::from(1u32)]);
    let prod11 = prod.evaluate(&[Curve25519Scalar::from(1u32), Curve25519Scalar::from(1u32)]);
    let calc00 = -Curve25519Scalar::from(40u32);
    let calc10 = -Curve25519Scalar::from(42u32);
    let calc01 = -Curve25519Scalar::from(82u32);
    let calc11 = Curve25519Scalar::from(33u32);
    assert_eq!(prod00, calc00);
    assert_eq!(prod10, calc10);
    assert_eq!(prod01, calc01);
    assert_eq!(prod11, calc11);
}

#[allow(clippy::identity_op)]
#[test]
fn test_composite_polynomial_hypercube_sum() {
    let a: Vec<Curve25519Scalar> = vec![
        -Curve25519Scalar::from(7u32),
        Curve25519Scalar::from(2u32),
        -Curve25519Scalar::from(6u32),
        Curve25519Scalar::from(17u32),
    ];
    let b: Vec<Curve25519Scalar> = vec![
        Curve25519Scalar::from(2u32),
        -Curve25519Scalar::from(8u32),
        Curve25519Scalar::from(4u32),
        Curve25519Scalar::from(1u32),
    ];
    let c: Vec<Curve25519Scalar> = vec![
        Curve25519Scalar::from(1u32),
        Curve25519Scalar::from(3u32),
        -Curve25519Scalar::from(5u32),
        -Curve25519Scalar::from(9u32),
    ];
    let mut prod = CompositePolynomial::new(2);
    prod.add_product([Rc::new(a), Rc::new(b)], Curve25519Scalar::from(3u32));
    prod.add_product([Rc::new(c)], Curve25519Scalar::from(2u32));
    let sum = prod.hypercube_sum(4);
    assert_eq!(
        sum,
        Curve25519Scalar::from(
            3 * ((-7) * 2 + 2 * (-8) + (-6) * 4 + 17 * 1) + 2 * (1 + 3 + (-5) + (-9))
        )
    );
}
