use crate::base::scalar::inner_product::*;

use curve25519_dalek::scalar::Scalar;

#[test]
fn test_inner_product() {
    let a = vec![
        Scalar::from(1u64),
        Scalar::from(2u64),
        Scalar::from(3u64),
        Scalar::from(4u64),
    ];
    let b = vec![
        Scalar::from(2u64),
        Scalar::from(3u64),
        Scalar::from(4u64),
        Scalar::from(5u64),
    ];
    assert_eq!(Scalar::from(40u64), inner_product(&a, &b));
}
