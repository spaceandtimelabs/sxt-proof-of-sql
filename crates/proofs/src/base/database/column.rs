use arrow::datatypes::DataType;

/// Represents a read-only view of a column in an in-memory,
/// column-oriented database.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
pub enum Column<'a> {
    BigInt(&'a [i64]),
}

/// Represents the supported data types of a column in an in-memory,
/// column-oriented database.
///
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum ColumnType {
    BigInt,
}

/// Convert ColumnType values to some arrow DataType
impl From<&ColumnType> for DataType {
    fn from(column_type: &ColumnType) -> Self {
        match column_type {
            ColumnType::BigInt => DataType::Int64,
        }
    }
}

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ColumnRef {
    pub column_name: String,
    pub table_name: String,
    pub schema: Option<String>,
    pub column_type: ColumnType,
}
