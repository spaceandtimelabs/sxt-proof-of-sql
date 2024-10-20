//! This module defines math utilities used in Proof of SQL.
/// Handles parsing between decimal tokens received from the lexer into native `Decimal75` Proof of SQL type.
pub mod decimal;
#[cfg(test)]
mod decimal_tests;
mod log;
pub(crate) use log::log2_up;
/// TODO: add docs
pub(crate) mod permutation;

mod intermediate_decimal_ext;
pub(crate) use intermediate_decimal_ext::IntermediateDecimalExt;
