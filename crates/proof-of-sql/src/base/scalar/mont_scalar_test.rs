use crate::base::scalar::test_scalar::TestScalar;

#[test]
fn test_add() {
    let one = TestScalar::from(1u64);
    let two = TestScalar::from(2u64);
    let sum = one + two;
    let expected_sum = TestScalar::from(3u64);
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_mod() {
    let pm1: ark_ff::BigInt<4> = ark_ff::BigInt!(
        "7237005577332262213973186563042994240857116359379907606001950938285454250988"
    );
    let x = TestScalar::from(pm1.0);
    let one = TestScalar::from(1u64);
    let zero = TestScalar::from(0u64);
    let xp1 = x + one;
    assert_eq!(xp1, zero);
}
