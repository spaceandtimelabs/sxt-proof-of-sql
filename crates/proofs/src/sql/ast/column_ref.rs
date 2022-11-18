/// Expression for an SQL table
#[derive(Debug, PartialEq, Eq)]
pub struct ColumnRef {
    pub column_name: String,
    pub table_name: String,
    pub namespace: Option<String>,
}
