//! This module provides conversions and utilities for working with Arrow data structures.

/// Module for handling conversion from Arrow arrays to columns.
pub mod arrow_array_to_column_conversion;

/// Module for converting between owned and Arrow data structures.
pub mod owned_and_arrow_conversions;

#[cfg(test)]
/// Tests for owned and Arrow conversions.
mod owned_and_arrow_conversions_test;

/// Module for scalar and i256 conversions.
pub mod scalar_and_i256_conversions;
