mod error;
pub use error::{ConversionError, ConversionResult};

mod converter;
pub use converter::Converter;

#[cfg(test)]
mod converter_tests;
