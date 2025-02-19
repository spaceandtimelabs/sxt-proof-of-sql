//! This module provides conversions and utilities for working with Arrow data structures.

/// Module for handling conversion from Arrow arrays to columns.
pub mod arrow_array_to_column_conversion;

/// Module for converting between owned and Arrow data structures.
pub mod owned_and_arrow_conversions;

#[cfg(test)]
/// Tests for owned and Arrow conversions.
mod owned_and_arrow_conversions_test;

/// Module for converting record batches.
pub mod record_batch_conversion;

/// Module for record batch error definitions.
pub mod record_batch_errors;

/// Module for scalar and i256 conversions.
pub mod scalar_and_i256_conversions;

/// Module for handling conversions between columns and Arrow arrays.
pub mod column_arrow_conversions;
