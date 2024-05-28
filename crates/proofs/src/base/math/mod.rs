//! Handles parsing between decimal tokens received from the lexer into native `Decimal75` proofs type.
pub mod decimal;
#[cfg(test)]
mod decimal_tests;
mod log;
pub(crate) use log::log2_up;
