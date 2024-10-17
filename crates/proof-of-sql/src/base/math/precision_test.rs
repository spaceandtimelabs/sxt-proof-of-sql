use crate::base::math::Precision;
use serde_json;

#[test]
fn we_can_deserialize_valid_precision() {
    let json = "50"; // A valid value within the range
    let precision: Result<Precision, _> = serde_json::from_str(json);
    assert!(precision.is_ok());
    assert_eq!(precision.unwrap().value(), 50);
}

#[test]
fn we_can_deserialize_valid_precision_inclusive() {
    let json = "75"; // A valid value within the range
    let precision: Result<Precision, _> = serde_json::from_str(json);
    assert!(precision.is_ok());
    assert_eq!(precision.unwrap().value(), 75);
}

#[test]
fn we_cannot_deserialize_invalid_precision() {
    let json = "76"; // An invalid value outside the range
    let precision: Result<Precision, _> = serde_json::from_str(json);
    assert!(precision.is_err());
}

// Test deserialization of a non-numeric value
#[test]
fn we_cannot_deserialize_non_numeric_precision() {
    let json = "\"not a number\"";
    let precision: Result<Precision, _> = serde_json::from_str(json);
    assert!(precision.is_err());
}
