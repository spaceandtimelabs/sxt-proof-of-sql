#![allow(non_snake_case)]

use crate::base::proof::Commitment;
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_append() {
    let a = vec![
        Scalar::from(100_u32),
        Scalar::from(200_u32),
        Scalar::from(300_u32),
        Scalar::from(400_u32),
    ];

    let b = vec![
        Scalar::from(12_u32),
        Scalar::from(0_u32),
        -Scalar::from(32_u32),
        Scalar::from(17_u32),
    ];

    let c = vec![Scalar::from(3000_u32), Scalar::from(10000_u32)];
    let d = vec![Scalar::from(60000_u32), Scalar::from(0_u32)];

    let c_a1 = Commitment::from(&a[..]);
    let c_b1 = Commitment::from(&b[..]);

    let c_a2 = c_a1.update_append_commitment(&c);
    let c_b2 = c_b1.update_append_commitment(&d);

    let mut ac = a.clone();
    ac.extend(c);

    let mut bd = b.clone();
    bd.extend(d);

    let c_ac = Commitment::from(&ac[..]);
    let c_bd = Commitment::from(&bd[..]);

    assert_eq!(c_a2, c_ac);
    assert_eq!(c_b2, c_bd);
}
