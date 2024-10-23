use crate::base::{
    math::{precision::MAX_SUPPORTED_PRECISION, BigDecimalExt, Precision},
    scalar::Curve25519Scalar,
};
use bigdecimal::BigDecimal;

#[test]
fn we_cannot_scale_past_max_precision() {
    let decimal = "12345678901234567890123456789012345678901234567890123456789012345678900.0"
        .parse::<BigDecimal>()
        .unwrap();

    let target_scale = 5;

    assert!(decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX)).unwrap(),
            target_scale
        )
        .is_err());
}

#[test]
fn we_can_match_decimals_with_negative_scale() {
    let decimal = "120.00".parse::<BigDecimal>().unwrap();
    let target_scale = -1;
    let expected = [12, 0, 0, 0];
    let result = decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
            target_scale,
        )
        .unwrap();
    assert_eq!(result, Curve25519Scalar::from(expected));
}

#[test]
fn we_can_match_integers_with_negative_scale() {
    let decimal = "12300".parse::<BigDecimal>().unwrap();
    let target_scale = -2;
    let expected_limbs = [123, 0, 0, 0];

    let limbs = decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX)).unwrap(),
            target_scale,
        )
        .unwrap();

    assert_eq!(limbs, Curve25519Scalar::from(expected_limbs));
}

#[test]
fn we_can_match_negative_decimals() {
    let decimal = "-123.45".parse::<BigDecimal>().unwrap();
    let target_scale = 2;
    let expected_limbs = [12345, 0, 0, 0];
    let limbs = decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX)).unwrap(),
            target_scale,
        )
        .unwrap();
    assert_eq!(limbs, -Curve25519Scalar::from(expected_limbs));
}

#[allow(clippy::cast_possible_wrap)]
#[test]
fn we_can_match_decimals_at_extrema() {
    // a big decimal cannot scale up past the supported precision
    let decimal = "1234567890123456789012345678901234567890123456789012345678901234567890.0"
        .parse::<BigDecimal>()
        .unwrap();
    let target_scale = 6; // now precision exceeds maximum
    assert!(decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX),).unwrap(),
            target_scale
        )
        .is_err());

    // maximum decimal value we can support
    let decimal = "99999999999999999999999999999999999999999999999999999999999999999999999999.0"
        .parse::<BigDecimal>()
        .unwrap();
    let target_scale = 1;
    assert!(decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
            target_scale
        )
        .is_ok());

    // scaling larger than max will fail
    let decimal = "999999999999999999999999999999999999999999999999999999999999999999999999999.0"
        .parse::<BigDecimal>()
        .unwrap();
    let target_scale = 1;
    assert!(decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
            target_scale
        )
        .is_err());

    // smallest possible decimal value we can support (either signed/unsigned)
    let decimal = "0.000000000000000000000000000000000000000000000000000000000000000000000000001"
        .parse::<BigDecimal>()
        .unwrap();
    let target_scale = MAX_SUPPORTED_PRECISION as i8;
    assert!(decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX),).unwrap(),
            target_scale
        )
        .is_ok());

    // this is ok because it can be scaled to 75 precision
    let decimal = "0.1".parse::<BigDecimal>().unwrap();
    let target_scale = MAX_SUPPORTED_PRECISION as i8;
    assert!(decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
            target_scale
        )
        .is_ok());

    // this exceeds max precision
    let decimal = "1.0".parse::<BigDecimal>().unwrap();
    let target_scale = 75;
    assert!(decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(u8::try_from(decimal.precision()).unwrap_or(u8::MAX),).unwrap(),
            target_scale
        )
        .is_err());

    // but this is ok
    let decimal = "1.0".parse::<BigDecimal>().unwrap();
    let target_scale = 74;
    assert!(decimal
        .try_into_scalar_with_precision_and_scale::<Curve25519Scalar>(
            Precision::new(MAX_SUPPORTED_PRECISION).unwrap(),
            target_scale
        )
        .is_ok());
}
