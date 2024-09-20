use crate::base::database::ColumnType;
use alloc::string::String;
use thiserror::Error;

/// Errors from operations related to `OwnedColumn`s.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum OwnedColumnError {
    /// Can not perform type casting.
    #[error("Can not perform type casting from {from_type:?} to {to_type:?}")]
    TypeCastError {
        /// The type from which we are trying to cast.
        from_type: ColumnType,
        /// The type to which we are trying to cast.
        to_type: ColumnType,
    },
    /// Error in converting scalars to a given column type.
    #[error("Error in converting scalars to a given column type: {0}")]
    ScalarConversionError(String),
    /// Unsupported operation.
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

/// Result type for operations related to `OwnedColumn`s.
pub type OwnedColumnResult<T> = core::result::Result<T, OwnedColumnError>;
