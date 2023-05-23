use crate::base::{polynomial::ark_scalar::*, scalar::ToArkScalar};
use ark_ff::BigInt;

#[test]
fn test_dalek_interop_1() {
    let x = curve25519_dalek::scalar::Scalar::from(1u64);
    let xp = ToArkScalar::to_ark_scalar(&x);
    assert_eq!(xp.into_dalek_scalar(), x);
}

#[test]
fn test_dalek_interop_m1() {
    let x = curve25519_dalek::scalar::Scalar::from(123u64);
    let mx = -x;
    let xp = ToArkScalar::to_ark_scalar(&x);
    let mxp = -xp;
    assert_eq!(mxp, ToArkScalar::to_ark_scalar(&mx));
    assert_eq!(mxp.into_dalek_scalar(), mx);
}

#[test]
fn test_add() {
    let one = ToArkScalar::to_ark_scalar(&curve25519_dalek::scalar::Scalar::from(1u64));
    let two = ToArkScalar::to_ark_scalar(&curve25519_dalek::scalar::Scalar::from(2u64));
    let sum = one + two;
    let expected_sum = ToArkScalar::to_ark_scalar(&curve25519_dalek::scalar::Scalar::from(3u64));
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_mod() {
    let pm1: BigInt<4> =
        BigInt!("7237005577332262213973186563042994240857116359379907606001950938285454250988");
    let x = ArkScalar::from_bigint(pm1).unwrap();
    let one = ToArkScalar::to_ark_scalar(&curve25519_dalek::scalar::Scalar::from(1u64));
    let zero = ToArkScalar::to_ark_scalar(&curve25519_dalek::scalar::Scalar::from(0u64));
    let xp1 = x + one;
    assert_eq!(xp1, zero);
}
