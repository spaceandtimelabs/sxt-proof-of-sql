//! This module contains utilities for working with the library
/// Parse DDLs and find bigdecimal columns
pub mod parse;
/// Utility for reading a parquet file and writing to a blob which represents a `TableCommitment`
pub mod parquet_to_commitment_blob;
#[cfg(test)]
mod parquet_to_commitment_blob_integration_tests;
