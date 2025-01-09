use crate::base::{
    map::IndexSet,
    scalar::{
        mont_scalar::BN254Scalar, test_scalar::TestScalar, Curve25519Scalar, Scalar,
        ScalarConversionError,
    },
};
use alloc::{format, string::ToString, vec::Vec};
use byte_slice_cast::AsByteSlice;
use num_bigint::BigInt;
use num_traits::{Inv, One, Zero};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng,
};
use rand_core::SeedableRng;

#[test]
fn test_dalek_interop_1() {
    let x = curve25519_dalek::scalar::Scalar::from(1u64);
    let xp = Curve25519Scalar::from(1u64);
    assert_eq!(curve25519_dalek::scalar::Scalar::from(xp), x);
}

#[test]
fn test_dalek_interop_m1() {
    let x = curve25519_dalek::scalar::Scalar::from(123u64);
    let mx = -x;
    let xp = Curve25519Scalar::from(123u64);
    let mxp = -xp;
    assert_eq!(mxp, Curve25519Scalar::from(-123i64));
    assert_eq!(curve25519_dalek::scalar::Scalar::from(mxp), mx);
}

#[test]
fn test_curve25519_add() {
    let one = Curve25519Scalar::from(1u64);
    let two = Curve25519Scalar::from(2u64);
    let sum = one + two;
    let expected_sum = Curve25519Scalar::from(3u64);
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_bn254_add() {
    let one = BN254Scalar::from(1u64);
    let two = BN254Scalar::from(2u64);
    let sum = one + two;
    let expected_sum = BN254Scalar::from(3u64);
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_bn254_add_assign() {
    let mut sum = BN254Scalar::from(1u64);
    sum += BN254Scalar::from(1u64);
    let expected_sum = BN254Scalar::from(2u64);
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_bn254_sub() {
    let three = BN254Scalar::from(3u64);
    let two = BN254Scalar::from(2u64);
    let diff = three - two;
    let expected_diff = BN254Scalar::from(1u64);
    assert_eq!(diff, expected_diff);
}

#[test]
fn test_bn254_sub_assign() {
    let mut sum = BN254Scalar::from(1u64);
    sum -= BN254Scalar::from(1u64);
    let expected_sum = BN254Scalar::from(0u64);
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_bn254_mul() {
    let two = BN254Scalar::from(2u64);
    let product = two * two;
    let expected_product = BN254Scalar::from(4u64);
    assert_eq!(product, expected_product);
}

#[test]
fn test_bn254_mul_assign() {
    let mut product = BN254Scalar::from(2u64);
    product *= BN254Scalar::from(2u64);
    let expected_product = BN254Scalar::from(4u64);
    assert_eq!(product, expected_product);
}

#[test]
fn test_bn254_neg() {
    let one = BN254Scalar::from(1u64);
    let neg_one = -one;
    let sum = one + neg_one;
    let expected_sum = BN254Scalar::from(0u64);
    assert_eq!(sum, expected_sum);
}

#[test]
fn test_bn254_sum() {
    // Test sum of an empty iterator
    let values: Vec<BN254Scalar> = vec![];
    let sum: BN254Scalar = values.into_iter().sum();
    let expected_sum = BN254Scalar::from(0u64);
    assert_eq!(sum, expected_sum, "Sum of empty iterator is zero");

    // Test sum of a single element
    let values = vec![BN254Scalar::from(1u64)];
    let sum: BN254Scalar = values.into_iter().sum();
    let expected_sum = BN254Scalar::from(1u64);
    assert_eq!(sum, expected_sum, "Sum of single element is the element");

    // Test sum of multiple elements
    let values = vec![
        BN254Scalar::from(1u64),
        BN254Scalar::from(2u64),
        BN254Scalar::from(3u64),
    ];
    let sum: BN254Scalar = values.into_iter().sum();
    let expected_sum = BN254Scalar::from(6u64);
    assert_eq!(
        sum, expected_sum,
        "Sum of multiple elements is the sum of the elements"
    );
}

#[test]
fn test_bn254_product() {
    // Test sum of an empty iterator
    let values: Vec<BN254Scalar> = vec![];
    let sum: BN254Scalar = values.into_iter().product();
    let expected_sum = BN254Scalar::from(1u64);
    assert_eq!(sum, expected_sum, "Product of empty iterator is one");

    // Test sum of a single element
    let values = vec![BN254Scalar::from(1u64)];
    let sum: BN254Scalar = values.into_iter().sum();
    let expected_sum = BN254Scalar::from(1u64);
    assert_eq!(
        sum, expected_sum,
        "Product of single element is the element"
    );

    // Test sum of multiple elements
    let values = vec![
        BN254Scalar::from(1u64),
        BN254Scalar::from(2u64),
        BN254Scalar::from(3u64),
    ];
    let sum: BN254Scalar = values.into_iter().sum();
    let expected_sum = BN254Scalar::from(6u64);
    assert_eq!(
        sum, expected_sum,
        "Product of multiple elements is the product of the elements"
    );
}

#[test]
fn test_curve25519_mod() {
    let pm1: ark_ff::BigInt<4> = ark_ff::BigInt!(
        "7237005577332262213973186563042994240857116359379907606001950938285454250988"
    );
    let x = Curve25519Scalar::from(pm1.0);
    let one = Curve25519Scalar::from(1u64);
    let zero = Curve25519Scalar::from(0u64);
    let xp1 = x + one;
    assert_eq!(xp1, zero);
}

#[test]
fn test_bn254_mod() {
    let pm1: ark_ff::BigInt<4> = ark_ff::BigInt!(
        "21888242871839275222246405745257275088548364400416034343698204186575808495616"
    );
    let x = BN254Scalar::from(pm1.0);
    let one = BN254Scalar::from(1u64);
    let zero = BN254Scalar::from(0u64);
    let xp1 = x + one;
    assert_eq!(xp1, zero);
}

#[test]
fn test_curve25519_scalar_serialization() {
    let s = [
        Curve25519Scalar::from(1u8),
        -Curve25519Scalar::from(1u8),
        Curve25519Scalar::from(123),
        Curve25519Scalar::from(0),
        Curve25519Scalar::from(255),
        Curve25519Scalar::from(1234),
        Curve25519Scalar::from(12345),
        Curve25519Scalar::from(2357),
        Curve25519Scalar::from(999),
        Curve25519Scalar::from(123_456_789),
    ];
    let serialized = serde_json::to_string(&s).unwrap();
    let deserialized: [Curve25519Scalar; 10] = serde_json::from_str(&serialized).unwrap();
    assert_eq!(s, deserialized);
}

#[test]
fn test_bn254_scalar_serialization() {
    let s = [
        BN254Scalar::from(1u8),
        -BN254Scalar::from(1u8),
        BN254Scalar::from(123),
        BN254Scalar::from(0),
        BN254Scalar::from(255),
        BN254Scalar::from(1234),
        BN254Scalar::from(12345),
        BN254Scalar::from(2357),
        BN254Scalar::from(999),
        BN254Scalar::from(123_456_789),
    ];
    let serialized = serde_json::to_string(&s).unwrap();
    let deserialized: [BN254Scalar; 10] = serde_json::from_str(&serialized).unwrap();
    assert_eq!(s, deserialized);
}

#[test]
fn test_curve25519_scalar_display() {
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{}", Curve25519Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "1000000000000000000000000000000014DEF9DEA2F79CD65812631A5C4A12CA",
        format!("{}", Curve25519Scalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "0x0000...C123",
        format!("{:#}", Curve25519Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "0x1000...12CA",
        format!("{:#}", Curve25519Scalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "+0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", Curve25519Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "-0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", Curve25519Scalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "+0x0000...C123",
        format!("{:+#}", Curve25519Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "-0x0000...C123",
        format!("{:+#}", Curve25519Scalar::from(-0x00AB_C123))
    );
}

#[test]
fn test_bn254_scalar_display() {
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{}", BN254Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "30644E72E131A029B85045B68181585D2833E84879B9709143E1F593EF543EDE",
        format!("{}", BN254Scalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "0x0000...C123",
        format!("{:#}", BN254Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "0x3064...3EDE",
        format!("{:#}", BN254Scalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "+0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", BN254Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "-0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", BN254Scalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "+0x0000...C123",
        format!("{:+#}", BN254Scalar::from(0x00AB_C123))
    );
    assert_eq!(
        "-0x0000...C123",
        format!("{:+#}", BN254Scalar::from(-0x00AB_C123))
    );
}

#[test]
fn test_curve25519_scalar_mid() {
    assert_eq!(
        Curve25519Scalar::MAX_SIGNED,
        -Curve25519Scalar::one() * Curve25519Scalar::from(2).inv().unwrap()
    );
}

#[test]
fn test_bn254_scalar_mid() {
    assert_eq!(
        BN254Scalar::MAX_SIGNED,
        -BN254Scalar::one() * BN254Scalar::from(2).inv().unwrap()
    );
}

#[test]
fn test_bn254_scalar_impl() {
    assert_eq!(BN254Scalar::ZERO, BN254Scalar::zero());
    assert_eq!(BN254Scalar::ONE, BN254Scalar::one());
}

#[test]
fn test_curve25519_scalar_to_bool() {
    assert!(!bool::try_from(Curve25519Scalar::ZERO).unwrap());
    assert!(bool::try_from(Curve25519Scalar::ONE).unwrap());
}

#[test]
fn test_bn254_scalar_to_bool() {
    assert!(!bool::try_from(BN254Scalar::ZERO).unwrap());
    assert!(bool::try_from(BN254Scalar::ONE).unwrap());
}

#[test]
fn test_curve25519_scalar_to_bool_overflow() {
    matches!(
        bool::try_from(Curve25519Scalar::from(2)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        bool::try_from(Curve25519Scalar::from(-1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        bool::try_from(Curve25519Scalar::from(-2)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_bn254_scalar_to_bool_overflow() {
    matches!(
        bool::try_from(BN254Scalar::from(2)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        bool::try_from(BN254Scalar::from(-1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        bool::try_from(BN254Scalar::from(-2)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i8() {
    assert_eq!(i8::try_from(Curve25519Scalar::from(0)).unwrap(), 0);
    assert_eq!(i8::try_from(Curve25519Scalar::ONE).unwrap(), 1);
    assert_eq!(i8::try_from(Curve25519Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i8::try_from(Curve25519Scalar::from(i8::MAX)).unwrap(),
        i8::MAX
    );
    assert_eq!(
        i8::try_from(Curve25519Scalar::from(i8::MIN)).unwrap(),
        i8::MIN
    );
}

#[test]
fn test_bn254_scalar_to_i8() {
    assert_eq!(i8::try_from(BN254Scalar::from(0)).unwrap(), 0);
    assert_eq!(i8::try_from(BN254Scalar::ONE).unwrap(), 1);
    assert_eq!(i8::try_from(BN254Scalar::from(-1)).unwrap(), -1);
    assert_eq!(i8::try_from(BN254Scalar::from(i8::MAX)).unwrap(), i8::MAX);
    assert_eq!(i8::try_from(BN254Scalar::from(i8::MIN)).unwrap(), i8::MIN);
}

#[test]
fn test_curve25519_scalar_to_i8_overflow() {
    matches!(
        i8::try_from(Curve25519Scalar::from(i128::from(i8::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i8::try_from(Curve25519Scalar::from(i128::from(i8::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_bn254_scalar_to_i8_overflow() {
    matches!(
        i8::try_from(BN254Scalar::from(i128::from(i8::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i8::try_from(BN254Scalar::from(i128::from(i8::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i16() {
    assert_eq!(i16::try_from(Curve25519Scalar::from(0)).unwrap(), 0);
    assert_eq!(i16::try_from(Curve25519Scalar::ONE).unwrap(), 1);
    assert_eq!(i16::try_from(Curve25519Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i16::try_from(Curve25519Scalar::from(i16::MAX)).unwrap(),
        i16::MAX
    );
    assert_eq!(
        i16::try_from(Curve25519Scalar::from(i16::MIN)).unwrap(),
        i16::MIN
    );
}

#[test]
fn test_bn254_scalar_to_i16() {
    assert_eq!(i16::try_from(BN254Scalar::from(0)).unwrap(), 0);
    assert_eq!(i16::try_from(BN254Scalar::ONE).unwrap(), 1);
    assert_eq!(i16::try_from(BN254Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i16::try_from(BN254Scalar::from(i16::MAX)).unwrap(),
        i16::MAX
    );
    assert_eq!(
        i16::try_from(BN254Scalar::from(i16::MIN)).unwrap(),
        i16::MIN
    );
}

#[test]
fn test_curve25519_scalar_to_i16_overflow() {
    matches!(
        i16::try_from(Curve25519Scalar::from(i128::from(i16::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i16::try_from(Curve25519Scalar::from(i128::from(i16::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_bn254_scalar_to_i16_overflow() {
    matches!(
        i16::try_from(BN254Scalar::from(i128::from(i16::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i16::try_from(BN254Scalar::from(i128::from(i16::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i32() {
    assert_eq!(i32::try_from(Curve25519Scalar::from(0)).unwrap(), 0);
    assert_eq!(i32::try_from(Curve25519Scalar::ONE).unwrap(), 1);
    assert_eq!(i32::try_from(Curve25519Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i32::try_from(Curve25519Scalar::from(i32::MAX)).unwrap(),
        i32::MAX
    );
    assert_eq!(
        i32::try_from(Curve25519Scalar::from(i32::MIN)).unwrap(),
        i32::MIN
    );
}

#[test]
fn test_bn254_scalar_to_i32() {
    assert_eq!(i32::try_from(BN254Scalar::from(0)).unwrap(), 0);
    assert_eq!(i32::try_from(BN254Scalar::ONE).unwrap(), 1);
    assert_eq!(i32::try_from(BN254Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i32::try_from(BN254Scalar::from(i32::MAX)).unwrap(),
        i32::MAX
    );
    assert_eq!(
        i32::try_from(BN254Scalar::from(i32::MIN)).unwrap(),
        i32::MIN
    );
}

#[test]
fn test_curve25519_scalar_to_i32_overflow() {
    matches!(
        i32::try_from(Curve25519Scalar::from(i128::from(i32::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i32::try_from(Curve25519Scalar::from(i128::from(i32::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_bn254_scalar_to_i32_overflow() {
    matches!(
        i32::try_from(BN254Scalar::from(i128::from(i32::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i32::try_from(BN254Scalar::from(i128::from(i32::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i64() {
    assert_eq!(i64::try_from(Curve25519Scalar::from(0)).unwrap(), 0);
    assert_eq!(i64::try_from(Curve25519Scalar::ONE).unwrap(), 1);
    assert_eq!(i64::try_from(Curve25519Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i64::try_from(Curve25519Scalar::from(i64::MAX)).unwrap(),
        i64::MAX
    );
    assert_eq!(
        i64::try_from(Curve25519Scalar::from(i64::MIN)).unwrap(),
        i64::MIN
    );
}

#[test]
fn test_bn254_scalar_to_i64() {
    assert_eq!(i64::try_from(BN254Scalar::from(0)).unwrap(), 0);
    assert_eq!(i64::try_from(BN254Scalar::ONE).unwrap(), 1);
    assert_eq!(i64::try_from(BN254Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i64::try_from(BN254Scalar::from(i64::MAX)).unwrap(),
        i64::MAX
    );
    assert_eq!(
        i64::try_from(BN254Scalar::from(i64::MIN)).unwrap(),
        i64::MIN
    );
}

#[test]
fn test_curve25519_scalar_to_i64_overflow() {
    matches!(
        i64::try_from(Curve25519Scalar::from(i128::from(i64::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i64::try_from(Curve25519Scalar::from(i128::from(i64::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_bn254_scalar_to_i64_overflow() {
    matches!(
        i64::try_from(BN254Scalar::from(i128::from(i64::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i64::try_from(BN254Scalar::from(i128::from(i64::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i128() {
    assert_eq!(i128::try_from(Curve25519Scalar::from(0)).unwrap(), 0);
    assert_eq!(i128::try_from(Curve25519Scalar::ONE).unwrap(), 1);
    assert_eq!(i128::try_from(Curve25519Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i128::try_from(Curve25519Scalar::from(i128::MAX)).unwrap(),
        i128::MAX
    );
    assert_eq!(
        i128::try_from(Curve25519Scalar::from(i128::MIN)).unwrap(),
        i128::MIN
    );
}

#[test]
fn test_bn254_scalar_to_i128() {
    assert_eq!(i128::try_from(BN254Scalar::from(0)).unwrap(), 0);
    assert_eq!(i128::try_from(BN254Scalar::ONE).unwrap(), 1);
    assert_eq!(i128::try_from(BN254Scalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i128::try_from(BN254Scalar::from(i128::MAX)).unwrap(),
        i128::MAX
    );
    assert_eq!(
        i128::try_from(BN254Scalar::from(i128::MIN)).unwrap(),
        i128::MIN
    );
}

#[test]
fn test_curve25519_scalar_to_i128_overflow() {
    matches!(
        i128::try_from(Curve25519Scalar::from(i128::MAX) + Curve25519Scalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i128::try_from(Curve25519Scalar::from(i128::MIN) - Curve25519Scalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_bn254_scalar_to_i128_overflow() {
    matches!(
        i128::try_from(BN254Scalar::from(i128::MAX) + BN254Scalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i128::try_from(BN254Scalar::from(i128::MIN) - BN254Scalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_bigint() {
    assert_eq!(BigInt::from(Curve25519Scalar::ZERO), BigInt::from(0_i8));
    assert_eq!(BigInt::from(Curve25519Scalar::ONE), BigInt::from(1_i8));
    assert_eq!(BigInt::from(-Curve25519Scalar::ONE), BigInt::from(-1_i8));
    assert_eq!(
        BigInt::from(Curve25519Scalar::from(i128::MAX)),
        BigInt::from(i128::MAX)
    );
    assert_eq!(
        BigInt::from(Curve25519Scalar::from(i128::MIN)),
        BigInt::from(i128::MIN)
    );
}

#[test]
fn test_bn254_scalar_to_bigint() {
    assert_eq!(BigInt::from(BN254Scalar::ZERO), BigInt::from(0_i8));
    assert_eq!(BigInt::from(BN254Scalar::ONE), BigInt::from(1_i8));
    assert_eq!(BigInt::from(-BN254Scalar::ONE), BigInt::from(-1_i8));
    assert_eq!(
        BigInt::from(BN254Scalar::from(i128::MAX)),
        BigInt::from(i128::MAX)
    );
    assert_eq!(
        BigInt::from(BN254Scalar::from(i128::MIN)),
        BigInt::from(i128::MIN)
    );
}

#[test]
fn test_curve25519_scalar_from_bigint() {
    assert_eq!(
        Curve25519Scalar::try_from(BigInt::from(0_i8)).unwrap(),
        Curve25519Scalar::ZERO
    );
    assert_eq!(
        Curve25519Scalar::try_from(BigInt::from(1_i8)).unwrap(),
        Curve25519Scalar::ONE
    );
    assert_eq!(
        Curve25519Scalar::try_from(BigInt::from(-1_i8)).unwrap(),
        -Curve25519Scalar::ONE
    );
}

#[test]
fn test_bn254_scalar_from_bigint() {
    assert_eq!(
        BN254Scalar::try_from(BigInt::from(0_i8)).unwrap(),
        BN254Scalar::ZERO
    );
    assert_eq!(
        BN254Scalar::try_from(BigInt::from(1_i8)).unwrap(),
        BN254Scalar::ONE
    );
    assert_eq!(
        BN254Scalar::try_from(BigInt::from(-1_i8)).unwrap(),
        -BN254Scalar::ONE
    );
}

#[test]
fn the_zero_integer_maps_to_the_curve25519_zero_scalar() {
    assert_eq!(Curve25519Scalar::from(0_u32), Curve25519Scalar::zero());
    assert_eq!(Curve25519Scalar::from(0_u64), Curve25519Scalar::zero());
    assert_eq!(Curve25519Scalar::from(0_u128), Curve25519Scalar::zero());
    assert_eq!(Curve25519Scalar::from(0_i32), Curve25519Scalar::zero());
    assert_eq!(Curve25519Scalar::from(0_i64), Curve25519Scalar::zero());
    assert_eq!(Curve25519Scalar::from(0_i128), Curve25519Scalar::zero());
}

#[test]
fn the_zero_integer_maps_to_the_bn254_zero_scalar() {
    assert_eq!(BN254Scalar::from(0_u32), BN254Scalar::zero());
    assert_eq!(BN254Scalar::from(0_u64), BN254Scalar::zero());
    assert_eq!(BN254Scalar::from(0_u128), BN254Scalar::zero());
    assert_eq!(BN254Scalar::from(0_i32), BN254Scalar::zero());
    assert_eq!(BN254Scalar::from(0_i64), BN254Scalar::zero());
    assert_eq!(BN254Scalar::from(0_i128), BN254Scalar::zero());
}

#[test]
fn bools_map_to_curve25519_scalar_properly() {
    assert_eq!(Curve25519Scalar::from(true), Curve25519Scalar::one());
    assert_eq!(Curve25519Scalar::from(false), Curve25519Scalar::zero());
}

#[test]
fn bools_map_to_bn254_scalar_properly() {
    assert_eq!(BN254Scalar::from(true), BN254Scalar::one());
    assert_eq!(BN254Scalar::from(false), BN254Scalar::zero());
}

#[test]
fn the_one_integer_maps_to_the_curve25519_one_scalar() {
    assert_eq!(Curve25519Scalar::from(1_u32), Curve25519Scalar::one());
    assert_eq!(Curve25519Scalar::from(1_u64), Curve25519Scalar::one());
    assert_eq!(Curve25519Scalar::from(1_u128), Curve25519Scalar::one());
    assert_eq!(Curve25519Scalar::from(1_i32), Curve25519Scalar::one());
    assert_eq!(Curve25519Scalar::from(1_i64), Curve25519Scalar::one());
    assert_eq!(Curve25519Scalar::from(1_i128), Curve25519Scalar::one());
}

#[test]
fn the_one_integer_maps_to_the_bn254_one_scalar() {
    assert_eq!(BN254Scalar::from(1_u32), BN254Scalar::one());
    assert_eq!(BN254Scalar::from(1_u64), BN254Scalar::one());
    assert_eq!(BN254Scalar::from(1_u128), BN254Scalar::one());
    assert_eq!(BN254Scalar::from(1_i32), BN254Scalar::one());
    assert_eq!(BN254Scalar::from(1_i64), BN254Scalar::one());
    assert_eq!(BN254Scalar::from(1_i128), BN254Scalar::one());
}

#[test]
fn the_curve25519_zero_scalar_is_the_additive_identity() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for _ in 0..1000 {
        let a = Curve25519Scalar::from(rng.gen::<i128>());
        let b = Curve25519Scalar::from(rng.gen::<i128>());
        assert_eq!(a + b, b + a);
        assert_eq!(a + Curve25519Scalar::zero(), a);
        assert_eq!(b + Curve25519Scalar::zero(), b);
        assert_eq!(
            Curve25519Scalar::zero() + Curve25519Scalar::zero(),
            Curve25519Scalar::zero()
        );
    }
}

#[test]
fn the_bn254_zero_scalar_is_the_additive_identity() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for _ in 0..1000 {
        let a = BN254Scalar::from(rng.gen::<i128>());
        let b = BN254Scalar::from(rng.gen::<i128>());
        assert_eq!(a + b, b + a);
        assert_eq!(a + BN254Scalar::zero(), a);
        assert_eq!(b + BN254Scalar::zero(), b);
        assert_eq!(
            BN254Scalar::zero() + BN254Scalar::zero(),
            BN254Scalar::zero()
        );
    }
}

#[test]
fn the_curve25519_one_scalar_is_the_multiplicative_identity() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for _ in 0..1000 {
        let a = Curve25519Scalar::from(rng.gen::<i128>());
        let b = Curve25519Scalar::from(rng.gen::<i128>());
        assert_eq!(a * b, b * a);
        assert_eq!(a * Curve25519Scalar::one(), a);
        assert_eq!(b * Curve25519Scalar::one(), b);
        assert_eq!(
            Curve25519Scalar::one() * Curve25519Scalar::one(),
            Curve25519Scalar::one()
        );
    }
}

#[test]
fn the_bn254_one_scalar_is_the_multiplicative_identity() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for _ in 0..1000 {
        let a = BN254Scalar::from(rng.gen::<i128>());
        let b = BN254Scalar::from(rng.gen::<i128>());
        assert_eq!(a * b, b * a);
        assert_eq!(a * BN254Scalar::one(), a);
        assert_eq!(b * BN254Scalar::one(), b);
        assert_eq!(BN254Scalar::one() * BN254Scalar::one(), BN254Scalar::one());
    }
}

#[test]
fn the_empty_string_will_be_mapped_to_the_curve25519_zero_scalar() {
    assert_eq!(Curve25519Scalar::from(""), Curve25519Scalar::zero());
    assert_eq!(
        Curve25519Scalar::from(<&str>::default()),
        Curve25519Scalar::zero()
    );
}

#[test]
fn the_empty_string_will_be_mapped_to_the_bn254_zero_scalar() {
    assert_eq!(BN254Scalar::from(""), BN254Scalar::zero());
    assert_eq!(BN254Scalar::from(<&str>::default()), BN254Scalar::zero());
}

#[test]
fn two_different_strings_map_to_different_scalars() {
    let s = "abc12";
    assert_ne!(Curve25519Scalar::from(s), Curve25519Scalar::zero());
    assert_ne!(Curve25519Scalar::from(s), Curve25519Scalar::from("abc123"));
    assert_ne!(BN254Scalar::from(s), BN254Scalar::zero());
    assert_ne!(BN254Scalar::from(s), BN254Scalar::from("abc123"));
}

#[test]
fn the_empty_buffer_will_be_mapped_to_the_zero_scalar() {
    let buf = Vec::<u8>::default();
    assert_eq!(Curve25519Scalar::from(&buf[..]), Curve25519Scalar::zero());
    assert_eq!(BN254Scalar::from(&buf[..]), BN254Scalar::zero());
}

#[test]
fn byte_arrays_with_the_same_content_but_different_types_map_to_different_curve25519_scalars() {
    let array = [1_u8, 2_u8, 34_u8];
    assert_ne!(
        Curve25519Scalar::from(array.as_byte_slice()),
        Curve25519Scalar::zero()
    );
    assert_ne!(
        Curve25519Scalar::from(array.as_byte_slice()),
        Curve25519Scalar::from([1_u32, 2_u32, 34_u32].as_byte_slice())
    );
}

#[test]
fn byte_arrays_with_the_same_content_but_different_types_map_to_different_bn254_scalars() {
    let array = [1_u8, 2_u8, 34_u8];
    assert_ne!(
        BN254Scalar::from(array.as_byte_slice()),
        BN254Scalar::zero()
    );
    assert_ne!(
        BN254Scalar::from(array.as_byte_slice()),
        BN254Scalar::from([1_u32, 2_u32, 34_u32].as_byte_slice())
    );
}

#[test]
fn strings_of_arbitrary_size_map_to_different_curve25519_scalars() {
    let mut prev_scalars = IndexSet::default();
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, 100);

    for i in 0..100 {
        let s = format!(
            "{}_{}_{}",
            dist.sample(&mut rng),
            i,
            "testing string to scalar".repeat(dist.sample(&mut rng))
        );
        assert!(prev_scalars.insert(Curve25519Scalar::from(s.as_str())));
    }
}

#[test]
fn strings_of_arbitrary_size_map_to_different_bn254_scalars() {
    let mut prev_scalars = IndexSet::default();
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, 100);

    for i in 0..100 {
        let s = format!(
            "{}_{}_{}",
            dist.sample(&mut rng),
            i,
            "testing string to scalar".repeat(dist.sample(&mut rng))
        );
        assert!(prev_scalars.insert(BN254Scalar::from(s.as_str())));
    }
}

#[allow(clippy::cast_sign_loss)]
#[test]
fn byte_arrays_of_arbitrary_size_map_to_different_curve25519_scalars() {
    let mut prev_scalars = IndexSet::default();
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, 100);

    for _ in 0..100 {
        let v = (0..dist.sample(&mut rng))
            .map(|_v| (dist.sample(&mut rng) % 255) as u8)
            .collect::<Vec<u8>>();
        assert!(prev_scalars.insert(Curve25519Scalar::from(&v[..])));
    }
}

#[allow(clippy::cast_sign_loss)]
#[test]
fn byte_arrays_of_arbitrary_size_map_to_different_bn254_scalars() {
    let mut prev_scalars = IndexSet::default();
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, 100);

    for _ in 0..100 {
        let v = (0..dist.sample(&mut rng))
            .map(|_v| (dist.sample(&mut rng) % 255) as u8)
            .collect::<Vec<u8>>();
        assert!(prev_scalars.insert(BN254Scalar::from(&v[..])));
    }
}

#[test]
fn the_curve25519_string_hash_implementation_uses_the_full_range_of_bits() {
    let max_iters = 20;
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, i32::MAX);

    for i in 0..252 {
        let mut curr_iters = 0;
        let mut bset = IndexSet::default();

        loop {
            let s: Curve25519Scalar = dist.sample(&mut rng).to_string().as_str().into();
            let bytes = s.to_bytes_le(); //Note: this is the only spot that these tests are different from the to_curve25519_scalar tests.

            let is_ith_bit_set = bytes[i / 8] & (1 << (i % 8)) != 0;

            bset.insert(is_ith_bit_set);

            if bset == IndexSet::from_iter([false, true]) {
                break;
            }

            // this guarantees that, if the above test fails,
            // we'll be able to identify it's failing
            assert!(curr_iters <= max_iters);

            curr_iters += 1;
        }
    }
}

#[test]
fn the_bn254_string_hash_implementation_uses_the_full_range_of_bits() {
    let max_iters = 20;
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, i32::MAX);

    for i in 0..252 {
        let mut curr_iters = 0;
        let mut bset = IndexSet::default();

        loop {
            let s: BN254Scalar = dist.sample(&mut rng).to_string().as_str().into();
            let bytes = s.to_bytes_le(); //Note: this is the only spot that these tests are different from the to_curve25519_scalar tests.

            let is_ith_bit_set = bytes[i / 8] & (1 << (i % 8)) != 0;

            bset.insert(is_ith_bit_set);

            if bset == IndexSet::from_iter([false, true]) {
                break;
            }

            // this guarantees that, if the above test fails,
            // we'll be able to identify it's failing
            assert!(curr_iters <= max_iters);

            curr_iters += 1;
        }
    }
}

#[test]
fn test_bigint_to_scalar_overflow() {
    assert_eq!(
        TestScalar::try_from(
            "3618502788666131106986593281521497120428558179689953803000975469142727125494"
                .parse::<BigInt>()
                .unwrap()
        )
        .unwrap(),
        TestScalar::MAX_SIGNED
    );
    assert_eq!(
        TestScalar::try_from(
            "-3618502788666131106986593281521497120428558179689953803000975469142727125494"
                .parse::<BigInt>()
                .unwrap()
        )
        .unwrap(),
        -TestScalar::MAX_SIGNED
    );

    assert!(matches!(
        TestScalar::try_from(
            "3618502788666131106986593281521497120428558179689953803000975469142727125495"
                .parse::<BigInt>()
                .unwrap()
        ),
        Err(ScalarConversionError::Overflow { .. })
    ));
    assert!(matches!(
        TestScalar::try_from(
            "-3618502788666131106986593281521497120428558179689953803000975469142727125495"
                .parse::<BigInt>()
                .unwrap()
        ),
        Err(ScalarConversionError::Overflow { .. })
    ));
}
