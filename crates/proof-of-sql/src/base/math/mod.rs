//! This module defines math utilities used in Proof of SQL.
/// Handles parsing between decimal tokens received from the lexer into native `Decimal75` Proof of SQL type.
pub mod decimal;
pub use decimal::MAX_DECIMAL_PRECISION;
/// Module containing [I256] type.
pub mod i256;
mod log;
pub(crate) use log::log2_up;
/// TODO: add docs
pub(crate) mod permutation;

mod big_decimal_ext;
pub(crate) use big_decimal_ext::BigDecimalExt;
