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
pub struct NonNegativeI32(#[cfg_attr(test, proptest(strategy = "1..31i32"))] i32);

/// Sepcified byte width is outside of supported range.
#[derive(Debug)]
pub struct WidthError(i32);

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn fixed_binary_column_details() -> impl Strategy<Value = (NonNegativeI32, Vec<u8>)> {
    (any::<NonNegativeI32>(), 0..100usize).prop_flat_map(|(width, num_rows)| {
        let len = width.0 as usize;
        (
            Just(width),
            proptest::collection::vec(any::<u8>(), len * num_rows),
        )
    })
}

impl core::fmt::Display for WidthError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WidthError(n) => write!(f, "negative width: {n}"),
        }
    }
}

impl<'a> From<&'a NonNegativeI32> for usize {
    fn from(val: &'a NonNegativeI32) -> Self {
        val.0 as usize
    }
}

impl From<NonNegativeI32> for i32 {
    fn from(val: NonNegativeI32) -> Self {
        val.0
    }
}

impl From<NonNegativeI32> for usize {
    fn from(val: NonNegativeI32) -> Self {
        val.0 as usize
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
        if value < 0 || value > 31 {
            Err(WidthError(value))
        } else {
            Ok(Self(value))
        }
    }
}

// This exists because `TryFrom<arrow::datatypes::DataType>` for `ColumnType` error is String
impl From<WidthError> for String {
    fn from(error: WidthError) -> Self {
        error.to_string()
    }
}
