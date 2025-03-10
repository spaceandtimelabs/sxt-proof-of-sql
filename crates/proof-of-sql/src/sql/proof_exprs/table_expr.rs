use crate::base::database::TableRef;
use serde::{Deserialize, Serialize};

/// Expression for an SQL table
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct TableExpr {
    /// The `TableRef` for the table
    pub table_ref: TableRef,
}
