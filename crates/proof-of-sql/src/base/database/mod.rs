//! Module with database related functionality. In particular, this module contains the
//! accessor traits and the `OwnedTable` type along with some utility functions to convert
//! between Arrow and `OwnedTable`.
mod accessor;
pub use accessor::{CommitmentAccessor, DataAccessor, MetadataAccessor, SchemaAccessor};

mod column;
pub use column::{Column, ColumnField, ColumnRef, ColumnType};

#[allow(dead_code)]
pub(crate) mod slice_operation;

mod slice_decimal_operation;

mod column_type_operation;
pub use column_type_operation::{
    try_add_subtract_column_types, try_divide_column_types, try_multiply_column_types,
};

mod column_arithmetic_operation;
pub(super) use column_arithmetic_operation::{AddOp, ArithmeticOp, DivOp, MulOp, SubOp};

mod column_comparison_operation;
pub(super) use column_comparison_operation::{ComparisonOp, EqualOp, GreaterThanOp, LessThanOp};

mod column_index_operation;
pub(super) use column_index_operation::apply_column_to_indexes;

mod column_repetition_operation;
pub(super) use column_repetition_operation::{ColumnRepeatOp, ElementwiseRepeatOp, RepetitionOp};

mod column_operation_error;
pub use column_operation_error::{ColumnOperationError, ColumnOperationResult};

mod table_operation_error;
pub use table_operation_error::{TableOperationError, TableOperationResult};

mod columnar_value;
pub use columnar_value::ColumnarValue;

mod literal_value;
pub use literal_value::LiteralValue;

mod error;

mod table_ref;
#[cfg(feature = "arrow")]
pub use crate::base::arrow::{
    arrow_array_to_column_conversion::{ArrayRefExt, ArrowArrayToColumnConversionError},
    owned_and_arrow_conversions::OwnedArrowConversionError,
    record_batch_utility::ToArrow,
    scalar_and_i256_conversions,
};
pub use table_ref::TableRef;

#[cfg(feature = "arrow")]
pub mod arrow_schema_utility;

mod owned_column;
pub use owned_column::OwnedColumn;

mod owned_column_error;
pub(crate) use owned_column_error::ColumnCoercionError;
pub use owned_column_error::{OwnedColumnError, OwnedColumnResult};

/// TODO: add docs
pub(crate) mod owned_column_operation;

mod owned_table;
pub use owned_table::OwnedTable;
pub(crate) use owned_table::{OwnedTableError, TableCoercionError};
#[cfg(test)]
mod owned_table_test;
pub mod owned_table_utility;

mod table;
#[cfg(test)]
pub(crate) use table::TableError;
pub use table::{Table, TableOptions};
#[cfg(test)]
mod table_test;
pub mod table_utility;

mod table_evaluation;
pub use table_evaluation::TableEvaluation;

/// TODO: add docs
pub(crate) mod expression_evaluation;
mod expression_evaluation_error;
#[cfg(test)]
mod expression_evaluation_test;
pub use expression_evaluation_error::{ExpressionEvaluationError, ExpressionEvaluationResult};

mod test_accessor;
pub use test_accessor::TestAccessor;

#[cfg(test)]
mod test_schema_accessor;
#[cfg(test)]
pub(crate) use test_schema_accessor::TestSchemaAccessor;

mod owned_table_test_accessor;
pub use owned_table_test_accessor::OwnedTableTestAccessor;
#[cfg(all(test, feature = "blitzar"))]
mod owned_table_test_accessor_test;

mod table_test_accessor;
pub use table_test_accessor::TableTestAccessor;
#[cfg(all(test, feature = "blitzar"))]
mod table_test_accessor_test;

/// TODO: add docs
pub(crate) mod filter_util;
#[cfg(test)]
mod filter_util_test;

pub(crate) mod group_by_util;
#[cfg(test)]
mod group_by_util_test;

pub(crate) mod union_util;

pub(crate) mod order_by_util;
#[cfg(test)]
mod order_by_util_test;

#[allow(dead_code)]
pub(crate) mod join_util;
