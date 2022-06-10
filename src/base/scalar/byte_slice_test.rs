use crate::base::scalar::byte_slice::*;

use curve25519_dalek::scalar::Scalar;

#[test]
fn test_as_byte_slice() {
    let m1 = -Scalar::one();
    let m2 = -Scalar::from(2u64);

    let xs = [];
    let slice = as_byte_slice(&xs);
    assert_eq!(slice.len(), 0);

    let xs = [m1];
    let slice = as_byte_slice(&xs);
    assert_eq!(slice, m1.as_bytes());

    let xs = [m1, m2];
    let slice = as_byte_slice(&xs);
    assert_eq!(&slice[0..32], m1.as_bytes());
    assert_eq!(&slice[32..64], m2.as_bytes());
}
