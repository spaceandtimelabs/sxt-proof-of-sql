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
pub struct FixedSizeBinaryWidth(#[cfg_attr(test, proptest(strategy = "1..31i32"))] i32);

/// Sepcified byte width is outside of supported range.
#[derive(Debug)]
pub struct WidthError(i32);

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn fixed_binary_column_details() -> impl Strategy<Value = (FixedSizeBinaryWidth, Vec<u8>)>
{
    (any::<FixedSizeBinaryWidth>(), 0..100usize).prop_flat_map(|(width, num_rows)| {
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

impl<'a> From<&'a FixedSizeBinaryWidth> for usize {
    #[allow(clippy::cast_sign_loss)]
    fn from(val: &'a FixedSizeBinaryWidth) -> Self {
        val.0 as usize
    }
}

impl From<FixedSizeBinaryWidth> for i32 {
    fn from(val: FixedSizeBinaryWidth) -> Self {
        val.0
    }
}

impl From<FixedSizeBinaryWidth> for usize {
    #[allow(clippy::cast_sign_loss)]
    fn from(val: FixedSizeBinaryWidth) -> Self {
        val.0 as usize
    }
}

impl Display for FixedSizeBinaryWidth {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<i32> for FixedSizeBinaryWidth {
    type Error = WidthError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if (0..=31).contains(&value) {
            Ok(Self(value))
        } else {
            Err(WidthError(value))
        }
    }
}

// This exists because `TryFrom<arrow::datatypes::DataType>` for `ColumnType` error is String
impl From<WidthError> for String {
    fn from(error: WidthError) -> Self {
        error.to_string()
    }
}

#[cfg(test)]
mod precision_tests {

    use super::*;

    #[test]
    fn we_can_display_widtherror() {
        let e = WidthError(-5);
        assert_eq!(e.to_string(), "negative width: -5");
        let as_string: String = e.into();
        assert_eq!(as_string, "negative width: -5");
    }

    #[test]
    fn we_can_display_nonnegativei32() {
        let val = FixedSizeBinaryWidth::try_from(5).unwrap();
        assert_eq!(val.to_string(), "5");
    }

    #[test]
    fn we_can_convert_nonnegativei32_to_primitives() {
        let val = FixedSizeBinaryWidth::try_from(5).unwrap();
        let as_i32: i32 = val.into();
        assert_eq!(as_i32, 5);
        let as_usize: usize = val.into();
        assert_eq!(as_usize, 5);

        // Test also the conversion from &NonNegativeI32
        let val_ref = &val;
        let as_usize_ref: usize = val_ref.into();
        assert_eq!(as_usize_ref, 5);
    }

    #[test]
    fn we_cannot_construct_nonnegativei32_from_out_of_range() {
        // 32 is out of range (0..=31)
        let res = FixedSizeBinaryWidth::try_from(32);
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert_eq!(err.to_string(), "negative width: 32");

        // negative number is also out of range
        let res = FixedSizeBinaryWidth::try_from(-1);
        assert!(res.is_err());
        let err = res.err().unwrap();
        assert_eq!(err.to_string(), "negative width: -1");
    }

    #[test]
    fn we_can_construct_nonnegativei32_from_in_range() {
        // Minimum
        let zero = FixedSizeBinaryWidth::try_from(0).unwrap();
        assert_eq!(zero.to_string(), "0");

        // Maximum
        let thirty_one = FixedSizeBinaryWidth::try_from(31).unwrap();
        assert_eq!(thirty_one.to_string(), "31");
    }
}
