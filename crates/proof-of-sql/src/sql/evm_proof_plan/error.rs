use snafu::Snafu;

/// Represents errors that can occur in the EVM proof plan module.
#[derive(Snafu, Debug, PartialEq)]
pub(super) enum Error {
    /// Error indicating that the plan is not supported.
    #[snafu(display("plan not yet supported"))]
    NotSupported,
    /// Error indicating that the column was not found.
    #[snafu(display("column not found"))]
    ColumnNotFound,
    /// Error indicating that the table was not found.
    #[snafu(display("table not found"))]
    TableNotFound,
    /// Error indicating that table name can not be parsed into `TableRef`.
    #[snafu(display("table name can not be parsed into TableRef"))]
    InvalidTableName,
}
