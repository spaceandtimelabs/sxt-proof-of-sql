use crate::base::polynomial::ArkScalar;

use super::{CompositePolynomial, DenseMultilinearExtension};
use ark_std::rc::Rc;
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_composite_polynomial_evaluation() {
    let a: Vec<ArkScalar> = vec![
        -ArkScalar::from(7u32),
        ArkScalar::from(2u32),
        -ArkScalar::from(6u32),
        ArkScalar::from(17u32),
    ];
    let b: Vec<ArkScalar> = vec![
        ArkScalar::from(2u32),
        -ArkScalar::from(8u32),
        ArkScalar::from(4u32),
        ArkScalar::from(1u32),
    ];
    let c: Vec<ArkScalar> = vec![
        ArkScalar::from(1u32),
        ArkScalar::from(3u32),
        -ArkScalar::from(5u32),
        -ArkScalar::from(9u32),
    ];
    let fa = DenseMultilinearExtension::from_evaluations_slice(2, &a);
    let fb = DenseMultilinearExtension::from_evaluations_slice(2, &b);
    let fc = DenseMultilinearExtension::from_evaluations_slice(2, &c);
    let mut prod = CompositePolynomial::new(2);
    prod.add_product([Rc::new(fa), Rc::new(fb)], Scalar::from(3u32));
    prod.add_product([Rc::new(fc)], Scalar::from(2u32));
    let prod00 = prod.evaluate(&[Scalar::from(0u32), Scalar::from(0u32)]);
    let prod10 = prod.evaluate(&[Scalar::from(1u32), Scalar::from(0u32)]);
    let prod01 = prod.evaluate(&[Scalar::from(0u32), Scalar::from(1u32)]);
    let prod11 = prod.evaluate(&[Scalar::from(1u32), Scalar::from(1u32)]);
    let calc00 = -Scalar::from(40u32);
    let calc10 = -Scalar::from(42u32);
    let calc01 = -Scalar::from(82u32);
    let calc11 = Scalar::from(33u32);
    assert_eq!(prod00, calc00);
    assert_eq!(prod10, calc10);
    assert_eq!(prod01, calc01);
    assert_eq!(prod11, calc11);
}
