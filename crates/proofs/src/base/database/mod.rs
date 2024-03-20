#![warn(missing_docs)]
//! Module with database related functionality. In particular, this module contains the
//! accessor traits and the `OwnedTable` type along with some utility functions to convert
//! between Arrow and `OwnedTable`.
mod accessor;
pub use accessor::{CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor};

mod column;
pub use column::{Column, ColumnField, ColumnRef, ColumnType, INT128_PRECISION, INT128_SCALE};

mod table_ref;
pub use table_ref::TableRef;

mod arrow_array_to_column_conversion;
pub use arrow_array_to_column_conversion::*;

mod record_batch_dataframe_conversion;
pub use record_batch_dataframe_conversion::*;

mod record_batch_utility;
pub use record_batch_utility::*;

#[warn(missing_docs)]
#[cfg(any(test, feature = "test"))]
#[cfg(feature = "blitzar")]
mod record_batch_test_accessor;
#[cfg(any(test, feature = "test"))]
#[cfg(feature = "blitzar")]
pub use record_batch_test_accessor::RecordBatchTestAccessor;

#[warn(missing_docs)]
#[cfg(all(test, feature = "blitzar"))]
mod record_batch_test_accessor_test;

#[cfg(any(test, feature = "test"))]
mod test_accessor_utility;
#[cfg(any(test, feature = "test"))]
pub use test_accessor_utility::{make_random_test_accessor_data, RandomTestAccessorDescriptor};

#[warn(missing_docs)]
mod owned_column;
pub use owned_column::*;
#[warn(missing_docs)]
mod owned_table;
pub use owned_table::*;
#[cfg(test)]
#[warn(missing_docs)]
mod owned_table_test;

#[warn(missing_docs)]
mod owned_and_arrow_conversions;
pub use owned_and_arrow_conversions::*;
#[cfg(test)]
#[warn(missing_docs)]
mod owned_and_arrow_conversions_test;

#[warn(missing_docs)]
#[cfg(any(test, feature = "test"))]
mod test_accessor;
#[cfg(any(test, feature = "test"))]
pub use test_accessor::{TestAccessor, UnimplementedTestAccessor};

#[warn(missing_docs)]
#[cfg(any(test, feature = "test"))]
mod owned_table_test_accessor;
#[cfg(any(test, feature = "test"))]
pub use owned_table_test_accessor::OwnedTableTestAccessor;
#[cfg(all(test, feature = "blitzar"))]
mod owned_table_test_accessor_test;

mod scalar_and_i256_conversions;
