use alloc::string::{String, ToString};
use serde::{Deserialize, Deserializer, Serialize};
use snafu::Snafu;

#[derive(Snafu, Debug, Eq, PartialEq)]
#[snafu(display("Decimal precision is not valid: {precision}"))]
/// Decimal precision exceeds the allowed limit,
/// e.g. precision above 75/76/whatever set by Scalar
/// or non-positive aka `InvalidPrecision`
pub struct InvalidPrecisionError {
    precision: String,
}

#[derive(Eq, PartialEq, Debug, Clone, Hash, Serialize, Copy)]
/// limit-enforced precision
pub struct Precision(u8);
pub(super) const MAX_SUPPORTED_PRECISION: u8 = 75;

impl Precision {
    /// Constructor for creating a Precision instance
    pub fn new(value: u8) -> Result<Self, InvalidPrecisionError> {
        if value > MAX_SUPPORTED_PRECISION || value == 0 {
            Err(InvalidPrecisionError {
                precision: value.to_string(),
            })
        } else {
            Ok(Precision(value))
        }
    }

    /// Gets the precision as a u8 for this decimal
    #[must_use]
    pub fn value(&self) -> u8 {
        self.0
    }
}

impl TryFrom<i16> for Precision {
    type Error = InvalidPrecisionError;
    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Precision::new(value.try_into().map_err(|_| InvalidPrecisionError {
            precision: value.to_string(),
        })?)
    }
}

impl TryFrom<u64> for Precision {
    type Error = InvalidPrecisionError;
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Precision::new(value.try_into().map_err(|_| InvalidPrecisionError {
            precision: value.to_string(),
        })?)
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
