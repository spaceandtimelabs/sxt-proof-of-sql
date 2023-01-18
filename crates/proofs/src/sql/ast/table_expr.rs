use crate::base::database::TableRef;

/// Expression for an SQL table
#[derive(Debug, PartialEq, Eq)]
pub struct TableExpr {
    pub table_ref: TableRef,
}
