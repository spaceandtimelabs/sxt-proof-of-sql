use snafu::Snafu;

#[derive(Snafu, Debug, PartialEq)]
pub enum Error {
    #[snafu(display("plan yet not supported"))]
    NotSupported,
    #[snafu(display("column not found"))]
    ColumnNotFound,
    #[snafu(display("table not found"))]
    TableNotFound,
}
