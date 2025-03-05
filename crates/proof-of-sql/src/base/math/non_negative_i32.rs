use crate::alloc::string::ToString;
use alloc::{fmt::Display, string::String};
#[cfg(test)]
use proptest::prelude::*;
use serde::{Deserialize, Serialize};

/// Type-safe non-negative integer, exists for the sole purpose
/// of preventing negative values from being used as fixed-size
/// binary slice widths, due to arrow's unfortunate use of i32 as byte width.
/// for the fixed-size binary type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct NonNegativeI32(#[cfg_attr(test, proptest(strategy = "1..2048_i32"))] i32);

/// Error type for `NonNegativeI32::new`.
#[derive(Debug)]
pub enum WidthError {
    /// The width was less than 1.
    NegativeWidth(i32),
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn fixed_binary_column_details() -> impl Strategy<Value = (NonNegativeI32, Vec<u8>)> {
    (any::<NonNegativeI32>(), 0..100usize).prop_flat_map(|(width, num_rows)| {
        let len = width.width() as usize;
        (
            Just(width),
            proptest::collection::vec(any::<u8>(), len * num_rows),
        )
    })
}

impl core::fmt::Display for WidthError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WidthError::NegativeWidth(n) => write!(f, "negative width: {n}"),
        }
    }
}

impl NonNegativeI32 {
    /// Returns an error if `x` is negative. Otherwise returns the wrapped value.
    pub fn new(x: i32) -> Result<Self, WidthError> {
        if x < 1 {
            Err(WidthError::NegativeWidth(x))
        } else {
            Ok(Self(x))
        }
    }

    /// Returns the wrapped value.
    #[must_use]
    pub fn width(&self) -> i32 {
        self.0
    }

    /// Returns the wrapped value.
    #[allow(
        clippy::cast_sign_loss,
        reason = "i32 is guaranteed to be non-negative by constructor"
    )]
    #[must_use]
    pub fn width_as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Display for NonNegativeI32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<i32> for NonNegativeI32 {
    type Error = WidthError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            Err(WidthError::NegativeWidth(value))
        } else {
            Ok(Self::new(value).expect("Value should be non-negative"))
        }
    }
}

// This exists because `TryFrom<arrow::datatypes::DataType>` for `ColumnType` error is String
impl From<WidthError> for String {
    fn from(error: WidthError) -> Self {
        error.to_string()
    }
}
