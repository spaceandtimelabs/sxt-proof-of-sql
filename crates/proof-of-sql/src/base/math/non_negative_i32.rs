use alloc::fmt::Display;
use serde::{Deserialize, Serialize};

/// Type-safe non-negative integer, exists for the sole purpose
/// of preventing negative values from being used as fixed-size
/// binary slice widths, due to arrow's unfortunate use of i32 as width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct NonNegativeI32(i32);

/// Error type for `NonNegativeI32::new`.
#[derive(Debug)]
pub enum WidthError {
    /// The width was negative.
    NegativeWidth(i32),
}

impl NonNegativeI32 {
    /// Returns an error if `x` is negative. Otherwise returns the wrapped value.
    pub fn new(x: i32) -> Result<Self, WidthError> {
        if x < 0 {
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
