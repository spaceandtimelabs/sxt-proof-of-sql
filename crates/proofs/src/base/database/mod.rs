mod accessor;
pub use accessor::{CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor};

mod column;
pub use column::{Column, ColumnField, ColumnRef, ColumnType};

mod table;
pub use table::TableRef;

#[cfg(any(test, feature = "test"))]
mod test_accessor_column;
#[cfg(any(test, feature = "test"))]
pub use test_accessor_column::{TestAccessorColumn, TestAccessorColumns};

#[cfg(any(test, feature = "test"))]
mod test_accessor;
#[cfg(any(test, feature = "test"))]
pub use test_accessor::TestAccessor;

#[cfg(test)]
mod test_accessor_test;

#[cfg(any(test, feature = "test"))]
mod test_accessor_utility;
#[cfg(any(test, feature = "test"))]
pub use test_accessor_utility::{
    data_frame_to_accessors, data_frame_to_record_batch, make_random_test_accessor_data,
    RandomTestAccessorDescriptor,
};
