use crate::base::database::ColumnType;

/// Reference of a SQL column
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColumnRef {
    pub column_name: String,
    pub table_name: String,
    pub namespace: Option<String>,
    pub column_type: ColumnType,
}
