//! This module provides conversions and utilities for working with Arrow data structures.

#[cfg(feature = "arrow")]
/// Module for handling conversion from Arrow arrays to columns.
pub mod arrow_array_to_column_conversion;

#[cfg(feature = "arrow")]
/// Module for converting between owned and Arrow data structures.
pub mod owned_and_arrow_conversions;

#[cfg(all(test, feature = "arrow"))]
/// Tests for owned and Arrow conversions.
mod owned_and_arrow_conversions_test;

#[cfg(feature = "arrow")]
/// Module for converting record batches.
pub mod record_batch_conversion;

#[cfg(feature = "arrow")]
/// Module for record batch error definitions.
pub mod record_batch_errors;

#[cfg(feature = "arrow")]
/// Utility functions for record batches.
pub mod record_batch_utility;

#[cfg(feature = "arrow")]
/// Module for scalar and i256 conversions.
pub mod scalar_and_i256_conversions;

#[cfg(feature = "arrow")]
/// Module for handling conversions between columns and Arrow arrays.
pub mod column_arrow_conversions;

#[cfg(feature = "arrow")]
/// Module for managing Arrow query results,
pub mod arrow_query_result;

#[cfg(all(test, feature = "arrow"))]
mod final_round_builder_arrow_tests;

#[cfg(all(test, feature = "arrow"))]
mod provable_query_result_test;

#[cfg(all(test, feature = "arrow", feature = "test"))]
mod test_accessor_utility;
