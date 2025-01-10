use super::ScalarExt;
use crate::base::scalar::{test_scalar::TestScalar, Scalar};
use bnum::types::U256;
use core::str::FromStr;
use num_traits::Inv;
use rand::{rngs::StdRng, Rng, SeedableRng};

const MAX_TEST_SCALAR_SIGNED_VALUE_AS_STRING: &str =
    "3618502788666131106986593281521497120428558179689953803000975469142727125494";

fn random_u256(seed: u64) -> U256 {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut bytes = [0u64; 4];
    rng.fill(&mut bytes);
    U256::from(bytes)
}

#[test]
fn we_can_get_test_scalar_constants_from_z_p() {
    assert_eq!(TestScalar::from(0), TestScalar::ZERO);
    assert_eq!(TestScalar::from(1), TestScalar::ONE);
    assert_eq!(TestScalar::from(2), TestScalar::TWO);
    // -1/2 == least upper bound
    assert_eq!(-TestScalar::TWO.inv().unwrap(), TestScalar::MAX_SIGNED);
    assert_eq!(TestScalar::from(10), TestScalar::TEN);
}

#[test]
fn we_can_convert_u256_to_test_scalar_with_wrapping() {
    // ARRANGE
    let u256_value = U256::TWO;

    // ACT
    let test_scalar = TestScalar::from_wrapping(u256_value);

    // ASSERT
    assert_eq!(test_scalar, TestScalar::TWO);
}

#[test]
fn we_can_convert_u256_to_test_scalar_with_wrapping_of_large_value() {
    // ARRANGE
    let u256_value =
        U256::from_str(MAX_TEST_SCALAR_SIGNED_VALUE_AS_STRING).unwrap() * U256::TWO + U256::ONE;

    // ACT
    let test_scalar = TestScalar::from_wrapping(u256_value);

    // ASSERT
    assert_eq!(test_scalar, TestScalar::ZERO);
}

#[test]
fn we_can_convert_test_scalar_to_u256_with_wrapping() {
    // ARRANGE
    let test_scalar = TestScalar::TWO;

    // ACT
    let u256_value = test_scalar.into_u256_wrapping();

    // ASSERT
    assert_eq!(u256_value, U256::TWO);
}

#[test]
fn we_can_convert_test_scalar_to_u256_with_wrapping_of_negative_value() {
    // ARRANGE
    let test_scalar = -TestScalar::ONE;

    // ACT
    let u256: bnum::BUint<4> = test_scalar.into_u256_wrapping();

    // ASSERT
    assert_eq!(
        u256,
        U256::from_str(MAX_TEST_SCALAR_SIGNED_VALUE_AS_STRING).unwrap() * U256::TWO
    );
}

#[test]
fn we_can_convert_u256_to_test_scalar_with_wrapping_of_random_u256() {
    // ARRANGE
    let random_u256 = random_u256(100);
    let random_u256_after_wrapping = random_u256
        % (U256::from_str(MAX_TEST_SCALAR_SIGNED_VALUE_AS_STRING).unwrap() * U256::TWO + U256::ONE);
    assert_ne!(random_u256, random_u256_after_wrapping);

    // ACT
    let test_scalar = TestScalar::from_wrapping(random_u256);
    let expected_scalar = TestScalar::from_wrapping(random_u256_after_wrapping);

    // ASSERT
    assert_eq!(test_scalar, expected_scalar);
}
