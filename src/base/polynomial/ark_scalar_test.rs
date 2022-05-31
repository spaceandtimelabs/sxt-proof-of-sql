use crate::base::polynomial::ark_scalar::*;

use ark_ff::fields::MontConfig;
use ark_ff::BigInt;
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_dalek_interop_1() {
    let x = Scalar::from(1u64);
    let xp = to_ark_scalar(&x);
    assert_eq!(from_ark_scalar(&xp), x);
}

#[test]
fn test_dalek_interop_m1() {
    let x = Scalar::from(123u64);
    let mx = -x;
    let xp = to_ark_scalar(&x);
    let mxp = -xp;
    assert_eq!(mxp, to_ark_scalar(&mx));
    assert_eq!(from_ark_scalar(&mxp), mx);
}

#[test]
fn test_add() {
    let one = to_ark_scalar(&Scalar::from(1u64));
    let two = to_ark_scalar(&Scalar::from(2u64));
    let sum = one + two;
    let expected_sum = to_ark_scalar(&Scalar::from(3u64));
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_mod() {
    let pm1: BigInt<4> =
        BigInt!("7237005577332262213973186563042994240857116359379907606001950938285454250988");
    let x = ArkScalarConfig::from_bigint(pm1).unwrap();
    let one = to_ark_scalar(&Scalar::from(1u64));
    let zero = to_ark_scalar(&Scalar::from(0u64));
    let xp1 = x + one;
    assert_eq!(xp1, zero);
}

#[test]
fn test_vector_conversion() {
    let one = Scalar::from(1u64);
    let two = Scalar::from(2u64);
    let xs = [one, two];
    let mut xsp: [ArkScalar; 2] = [to_ark_scalar(&Scalar::from(0u64)); 2];
    to_ark_scalars(&mut xsp, &xs);
    assert_eq!(from_ark_scalar(&xsp[0]), one);
    assert_eq!(from_ark_scalar(&xsp[1]), two);
}
