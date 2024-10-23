//! This module defines math utilities used in Proof of SQL.
/// Handles parsing between decimal tokens received from the lexer into native `Decimal75` Proof of SQL type.
mod log;
pub(crate) use log::log2_up;
/// TODO: add docs
pub(crate) mod permutation;

mod big_decimal_ext;
pub(crate) use big_decimal_ext::BigDecimalExt;
#[cfg(test)]
mod big_decimal_ext_test;

mod precision;
pub use precision::{InvalidPrecisionError, Precision};
#[cfg(test)]
mod precision_test;

mod decimal_error;
pub use decimal_error::{DecimalError, DecimalResult};
