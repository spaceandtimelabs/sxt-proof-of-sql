use crate::base::{
    map::IndexSet,
    scalar::{test_scalar::TestScalar, test_scalar_constants, Scalar, ScalarConversionError},
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
fn we_have_correct_constants_for_curve_25519_scalar() {
    test_scalar_constants::<TestScalar>();
}

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

#[test]
fn test_curve25519_scalar_serialization() {
    let s = [
        TestScalar::from(1u8),
        -TestScalar::from(1u8),
        TestScalar::from(123),
        TestScalar::from(0),
        TestScalar::from(255),
        TestScalar::from(1234),
        TestScalar::from(12345),
        TestScalar::from(2357),
        TestScalar::from(999),
        TestScalar::from(123_456_789),
    ];
    let serialized = serde_json::to_string(&s).unwrap();
    let deserialized: [TestScalar; 10] = serde_json::from_str(&serialized).unwrap();
    assert_eq!(s, deserialized);
}

#[test]
fn test_curve25519_scalar_display() {
    assert_eq!(
        "0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{}", TestScalar::from(0x00AB_C123))
    );
    assert_eq!(
        "1000000000000000000000000000000014DEF9DEA2F79CD65812631A5C4A12CA",
        format!("{}", TestScalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "0x0000...C123",
        format!("{:#}", TestScalar::from(0x00AB_C123))
    );
    assert_eq!(
        "0x1000...12CA",
        format!("{:#}", TestScalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "+0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", TestScalar::from(0x00AB_C123))
    );
    assert_eq!(
        "-0000000000000000000000000000000000000000000000000000000000ABC123",
        format!("{:+}", TestScalar::from(-0x00AB_C123))
    );
    assert_eq!(
        "+0x0000...C123",
        format!("{:+#}", TestScalar::from(0x00AB_C123))
    );
    assert_eq!(
        "-0x0000...C123",
        format!("{:+#}", TestScalar::from(-0x00AB_C123))
    );
}

#[test]
fn test_curve25519_scalar_mid() {
    assert_eq!(
        TestScalar::MAX_SIGNED,
        -TestScalar::one() * TestScalar::from(2).inv().unwrap()
    );
}

#[test]
fn test_curve25519_scalar_to_bool() {
    assert!(!bool::try_from(TestScalar::ZERO).unwrap());
    assert!(bool::try_from(TestScalar::ONE).unwrap());
}

#[test]
fn test_curve25519_scalar_to_bool_overflow() {
    matches!(
        bool::try_from(TestScalar::from(2)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        bool::try_from(TestScalar::from(-1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        bool::try_from(TestScalar::from(-2)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i8() {
    assert_eq!(i8::try_from(TestScalar::from(0)).unwrap(), 0);
    assert_eq!(i8::try_from(TestScalar::ONE).unwrap(), 1);
    assert_eq!(i8::try_from(TestScalar::from(-1)).unwrap(), -1);
    assert_eq!(i8::try_from(TestScalar::from(i8::MAX)).unwrap(), i8::MAX);
    assert_eq!(i8::try_from(TestScalar::from(i8::MIN)).unwrap(), i8::MIN);
}

#[test]
fn test_curve25519_scalar_to_i8_overflow() {
    matches!(
        i8::try_from(TestScalar::from(i128::from(i8::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i8::try_from(TestScalar::from(i128::from(i8::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i16() {
    assert_eq!(i16::try_from(TestScalar::from(0)).unwrap(), 0);
    assert_eq!(i16::try_from(TestScalar::ONE).unwrap(), 1);
    assert_eq!(i16::try_from(TestScalar::from(-1)).unwrap(), -1);
    assert_eq!(i16::try_from(TestScalar::from(i16::MAX)).unwrap(), i16::MAX);
    assert_eq!(i16::try_from(TestScalar::from(i16::MIN)).unwrap(), i16::MIN);
}

#[test]
fn test_curve25519_scalar_to_i16_overflow() {
    matches!(
        i16::try_from(TestScalar::from(i128::from(i16::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i16::try_from(TestScalar::from(i128::from(i16::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i32() {
    assert_eq!(i32::try_from(TestScalar::from(0)).unwrap(), 0);
    assert_eq!(i32::try_from(TestScalar::ONE).unwrap(), 1);
    assert_eq!(i32::try_from(TestScalar::from(-1)).unwrap(), -1);
    assert_eq!(i32::try_from(TestScalar::from(i32::MAX)).unwrap(), i32::MAX);
    assert_eq!(i32::try_from(TestScalar::from(i32::MIN)).unwrap(), i32::MIN);
}

#[test]
fn test_curve25519_scalar_to_i32_overflow() {
    matches!(
        i32::try_from(TestScalar::from(i128::from(i32::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i32::try_from(TestScalar::from(i128::from(i32::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i64() {
    assert_eq!(i64::try_from(TestScalar::from(0)).unwrap(), 0);
    assert_eq!(i64::try_from(TestScalar::ONE).unwrap(), 1);
    assert_eq!(i64::try_from(TestScalar::from(-1)).unwrap(), -1);
    assert_eq!(i64::try_from(TestScalar::from(i64::MAX)).unwrap(), i64::MAX);
    assert_eq!(i64::try_from(TestScalar::from(i64::MIN)).unwrap(), i64::MIN);
}

#[test]
fn test_curve25519_scalar_to_i64_overflow() {
    matches!(
        i64::try_from(TestScalar::from(i128::from(i64::MAX) + 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i64::try_from(TestScalar::from(i128::from(i64::MIN) - 1)),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_i128() {
    assert_eq!(i128::try_from(TestScalar::from(0)).unwrap(), 0);
    assert_eq!(i128::try_from(TestScalar::ONE).unwrap(), 1);
    assert_eq!(i128::try_from(TestScalar::from(-1)).unwrap(), -1);
    assert_eq!(
        i128::try_from(TestScalar::from(i128::MAX)).unwrap(),
        i128::MAX
    );
    assert_eq!(
        i128::try_from(TestScalar::from(i128::MIN)).unwrap(),
        i128::MIN
    );
}

#[test]
fn test_curve25519_scalar_to_i128_overflow() {
    matches!(
        i128::try_from(TestScalar::from(i128::MAX) + TestScalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    );
    matches!(
        i128::try_from(TestScalar::from(i128::MIN) - TestScalar::ONE),
        Err(ScalarConversionError::Overflow { .. })
    );
}

#[test]
fn test_curve25519_scalar_to_bigint() {
    assert_eq!(BigInt::from(TestScalar::ZERO), BigInt::from(0_i8));
    assert_eq!(BigInt::from(TestScalar::ONE), BigInt::from(1_i8));
    assert_eq!(BigInt::from(-TestScalar::ONE), BigInt::from(-1_i8));
    assert_eq!(
        BigInt::from(TestScalar::from(i128::MAX)),
        BigInt::from(i128::MAX)
    );
    assert_eq!(
        BigInt::from(TestScalar::from(i128::MIN)),
        BigInt::from(i128::MIN)
    );
}

#[test]
fn test_curve25519_scalar_from_bigint() {
    assert_eq!(
        TestScalar::try_from(BigInt::from(0_i8)).unwrap(),
        TestScalar::ZERO
    );
    assert_eq!(
        TestScalar::try_from(BigInt::from(1_i8)).unwrap(),
        TestScalar::ONE
    );
    assert_eq!(
        TestScalar::try_from(BigInt::from(-1_i8)).unwrap(),
        -TestScalar::ONE
    );
}

#[test]
fn the_zero_integer_maps_to_the_zero_scalar() {
    assert_eq!(TestScalar::from(0_u32), TestScalar::zero());
    assert_eq!(TestScalar::from(0_u64), TestScalar::zero());
    assert_eq!(TestScalar::from(0_u128), TestScalar::zero());
    assert_eq!(TestScalar::from(0_i32), TestScalar::zero());
    assert_eq!(TestScalar::from(0_i64), TestScalar::zero());
    assert_eq!(TestScalar::from(0_i128), TestScalar::zero());
}

#[test]
fn bools_map_to_curve25519_scalar_properly() {
    assert_eq!(TestScalar::from(true), TestScalar::one());
    assert_eq!(TestScalar::from(false), TestScalar::zero());
}

#[test]
fn the_one_integer_maps_to_the_zero_scalar() {
    assert_eq!(TestScalar::from(1_u32), TestScalar::one());
    assert_eq!(TestScalar::from(1_u64), TestScalar::one());
    assert_eq!(TestScalar::from(1_u128), TestScalar::one());
    assert_eq!(TestScalar::from(1_i32), TestScalar::one());
    assert_eq!(TestScalar::from(1_i64), TestScalar::one());
    assert_eq!(TestScalar::from(1_i128), TestScalar::one());
}

#[test]
fn the_zero_scalar_is_the_additive_identity() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for _ in 0..1000 {
        let a = TestScalar::from(rng.gen::<i128>());
        let b = TestScalar::from(rng.gen::<i128>());
        assert_eq!(a + b, b + a);
        assert_eq!(a + TestScalar::zero(), a);
        assert_eq!(b + TestScalar::zero(), b);
        assert_eq!(TestScalar::zero() + TestScalar::zero(), TestScalar::zero());
    }
}

#[test]
fn the_one_scalar_is_the_multiplicative_identity() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for _ in 0..1000 {
        let a = TestScalar::from(rng.gen::<i128>());
        let b = TestScalar::from(rng.gen::<i128>());
        assert_eq!(a * b, b * a);
        assert_eq!(a * TestScalar::one(), a);
        assert_eq!(b * TestScalar::one(), b);
        assert_eq!(TestScalar::one() * TestScalar::one(), TestScalar::one());
    }
}

#[test]
fn the_empty_string_will_be_mapped_to_the_zero_scalar() {
    assert_eq!(TestScalar::from(""), TestScalar::zero());
    assert_eq!(TestScalar::from(<&str>::default()), TestScalar::zero());
}

#[test]
fn two_different_strings_map_to_different_scalars() {
    let s = "abc12";
    assert_ne!(TestScalar::from(s), TestScalar::zero());
    assert_ne!(TestScalar::from(s), TestScalar::from("abc123"));
}

#[test]
fn the_empty_buffer_will_be_mapped_to_the_zero_scalar() {
    let buf = Vec::<u8>::default();
    assert_eq!(TestScalar::from(&buf[..]), TestScalar::zero());
}

#[test]
fn byte_arrays_with_the_same_content_but_different_types_map_to_different_scalars() {
    let array = [1_u8, 2_u8, 34_u8];
    assert_ne!(TestScalar::from(array.as_byte_slice()), TestScalar::zero());
    assert_ne!(
        TestScalar::from(array.as_byte_slice()),
        TestScalar::from([1_u32, 2_u32, 34_u32].as_byte_slice())
    );
}

#[test]
fn strings_of_arbitrary_size_map_to_different_scalars() {
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
        assert!(prev_scalars.insert(TestScalar::from(s.as_str())));
    }
}

#[allow(clippy::cast_sign_loss)]
#[test]
fn byte_arrays_of_arbitrary_size_map_to_different_scalars() {
    let mut prev_scalars = IndexSet::default();
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, 100);

    for _ in 0..100 {
        let v = (0..dist.sample(&mut rng))
            .map(|_v| (dist.sample(&mut rng) % 255) as u8)
            .collect::<Vec<u8>>();
        assert!(prev_scalars.insert(TestScalar::from(&v[..])));
    }
}

#[test]
fn the_string_hash_implementation_uses_the_full_range_of_bits() {
    let max_iters = 20;
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, i32::MAX);

    for i in 0..252 {
        let mut curr_iters = 0;
        let mut bset = IndexSet::default();

        loop {
            let s: TestScalar = dist.sample(&mut rng).to_string().as_str().into();
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
