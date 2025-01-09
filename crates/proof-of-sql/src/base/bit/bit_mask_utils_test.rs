use super::bit_mask_utils::make_bit_mask;
use crate::base::{
    bit::bit_mask_utils::is_bit_mask_negative_representation,
    scalar::{test_scalar::TestScalar, Scalar},
};
use bnum::types::U256;

#[test]
fn we_can_make_positive_bit_mask() {
    // ARRANGE
    let positive_scalar = TestScalar::TWO;

    // ACT
    let bit_mask = make_bit_mask(positive_scalar);

    // ASSERT
    assert_eq!(bit_mask, (U256::ONE << 255) + U256::TWO);
}

#[test]
fn we_can_make_negative_bit_mask() {
    // ARRANGE
    let negative_scalar = -TestScalar::TWO;

    // ACT
    let bit_mask = make_bit_mask(negative_scalar);

    // ASSERT
    assert_eq!(bit_mask, (U256::ONE << 255) - U256::TWO);
}

#[test]
fn we_can_verify_positive_bit_mask_is_positive_representation() {
    // ARRANGE
    let positive_scalar = TestScalar::TWO;
    let bit_mask = make_bit_mask(positive_scalar);

    // ACT
    let is_positive = !is_bit_mask_negative_representation(bit_mask);

    // ASSERT
    assert!(is_positive);
}

#[test]
fn we_can_verify_negative_bit_mask_is_negative_representation() {
    // ARRANGE
    let negative_scalar = -TestScalar::TWO;
    let bit_mask = make_bit_mask(negative_scalar);

    // ACT
    let is_negative = is_bit_mask_negative_representation(bit_mask);

    // ASSERT
    assert!(is_negative);
}
