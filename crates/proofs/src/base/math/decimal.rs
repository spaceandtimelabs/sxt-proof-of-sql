use serde::{Deserialize, Deserializer, Serialize};

#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Copy)]
/// limit-enforced precision
pub struct Precision(u8);

impl Precision {
    /// Constructor for creating a Precision instance
    pub fn new(value: u8) -> Result<Self, String> {
        if value > 75 {
            Err("Precision exceeds the maximum allowed value of 75".to_string())
        } else {
            Ok(Precision(value))
        }
    }

    /// Getter method to access the inner value
    pub fn value(&self) -> u8 {
        self.0
    }
}

// Custom deserializer for precision since we need to limit its value to 75
impl<'de> Deserialize<'de> for Precision {
    fn deserialize<D>(deserializer: D) -> Result<Precision, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as a u8
        let value = u8::deserialize(deserializer)?;

        // Use the Precision::new method to ensure the value is within the allowed range
        Precision::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn deserialize_valid_precision() {
        let json = "50"; // A valid value within the range
        let precision: Result<Precision, _> = serde_json::from_str(json);
        assert!(precision.is_ok());
        assert_eq!(precision.unwrap().value(), 50);
    }

    #[test]
    fn deserialize_valid_precision_inclusive() {
        let json = "75"; // A valid value within the range
        let precision: Result<Precision, _> = serde_json::from_str(json);
        assert!(precision.is_ok());
        assert_eq!(precision.unwrap().value(), 75);
    }

    #[test]
    fn deserialize_invalid_precision() {
        let json = "76"; // An invalid value outside the range
        let precision: Result<Precision, _> = serde_json::from_str(json);
        assert!(precision.is_err());
    }

    // Test deserialization of a non-numeric value
    #[test]
    fn deserialize_non_numeric_precision() {
        let json = "\"not a number\"";
        let precision: Result<Precision, _> = serde_json::from_str(json);
        assert!(precision.is_err());
    }
}
