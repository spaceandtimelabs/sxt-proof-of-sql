/// Expression for an SQL table
#[derive(Debug, PartialEq, Eq)]
pub struct TableExpr {
    pub name: String,
    pub schema: Option<String>,
}
