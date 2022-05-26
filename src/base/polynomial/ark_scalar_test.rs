use crate::base::polynomial::ark_scalar::ArkScalar;

use ark_ff::BigInt;

#[test]
fn test_add() {
    let one = ArkScalar::new(BigInt::new([1, 0, 0, 0]));
    let two = ArkScalar::new(BigInt::new([2, 0, 0, 0]));
    let sum = one + two;
    let expected_sum = ArkScalar::new(BigInt::new([3, 0, 0, 0]));
    assert!(sum == expected_sum);
}

#[test]
fn test_mod() {
    const pm1 : BigInt<4> = BigInt!("7237005577332262213973186563042994240857116359379907606001950938285454250988");
    let x = ArkScalar::new(pm1);
    let one = ArkScalar::new(BigInt::new([1, 0, 0, 0]));
    let zero = ArkScalar::new(BigInt::new([0, 0, 0, 0]));
    let xp1 = x + one;
    assert!(xp1 == zero);
}
