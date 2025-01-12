use alloc::string::String;
use snafu::Snafu;

/// Errors that can occur during proof plan serialization.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum ProofPlanSerializationError {
    /// Error indicating that the operation is not supported.
    #[snafu(display("Not supported"))]
    NotSupported,
    /// Error indicating that there are more than 255 results in the filter.
    #[snafu(display("More than 255 results in filter."))]
    TooManyResults,
    /// Error indicating that there are more than 255 tables referenced in the plan.
    #[snafu(display("More than 255 tables referenced in the plan."))]
    TooManyTables,
    /// Error indicating that there are more than 255 columns referenced in the plan.
    #[snafu(display("More than 255 columns referenced in the plan."))]
    TooManyColumns,
    /// Error indicating that the table was not found.
    #[snafu(display("Table not found"))]
    TableNotFound,
    /// Error indicating that the column was not found.
    #[snafu(display("Column not found"))]
    ColumnNotFound,

    /// Error indicating as an invalid number format.
    #[snafu(display("Invalid number format: {value:?}"))]
    InvalidNumberFormat { value: String },
}
