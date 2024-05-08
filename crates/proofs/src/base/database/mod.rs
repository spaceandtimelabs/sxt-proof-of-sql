//! Module with database related functionality. In particular, this module contains the
//! accessor traits and the `OwnedTable` type along with some utility functions to convert
//! between Arrow and `OwnedTable`.
mod accessor;
pub use accessor::{CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor};

mod column;
pub use column::{Column, ColumnField, ColumnRef, ColumnType};
pub(crate) use column::{INT128_PRECISION, INT128_SCALE};

mod literal_value;
pub use literal_value::LiteralValue;

mod table_ref;
pub use table_ref::TableRef;

mod arrow_array_to_column_conversion;
pub use arrow_array_to_column_conversion::{ArrayRefExt, ArrowArrayToColumnConversionError};

mod record_batch_dataframe_conversion;
pub(crate) use record_batch_dataframe_conversion::{
    dataframe_to_record_batch, record_batch_to_dataframe,
};

mod record_batch_utility;
pub use record_batch_utility::ToArrow;

#[cfg(any(test, feature = "test"))]
#[cfg(feature = "blitzar")]
mod record_batch_test_accessor;
#[cfg(any(test, feature = "test"))]
#[cfg(feature = "blitzar")]
pub use record_batch_test_accessor::RecordBatchTestAccessor;

#[cfg(all(test, feature = "blitzar"))]
mod record_batch_test_accessor_test;

#[cfg(any(test, feature = "test"))]
mod test_accessor_utility;
#[cfg(any(test, feature = "test"))]
pub use test_accessor_utility::{make_random_test_accessor_data, RandomTestAccessorDescriptor};

mod owned_column;
pub use owned_column::OwnedColumn;
mod owned_table;
pub use owned_table::OwnedTable;
pub(crate) use owned_table::OwnedTableError;
#[cfg(test)]
mod owned_table_test;

mod owned_and_arrow_conversions;
#[cfg(test)]
pub(crate) use owned_and_arrow_conversions::OwnedArrowConversionError;
#[cfg(test)]
mod owned_and_arrow_conversions_test;

#[cfg(any(test, feature = "test"))]
mod test_accessor;
#[cfg(any(test, feature = "test"))]
pub use test_accessor::TestAccessor;
#[cfg(test)]
pub(crate) use test_accessor::UnimplementedTestAccessor;

#[cfg(any(test, feature = "test"))]
mod owned_table_test_accessor;
#[cfg(any(test, feature = "test"))]
pub use owned_table_test_accessor::OwnedTableTestAccessor;
#[cfg(all(test, feature = "blitzar"))]
mod owned_table_test_accessor_test;
/// Contains traits for scalar <-> i256 conversions
pub mod scalar_and_i256_conversions;
