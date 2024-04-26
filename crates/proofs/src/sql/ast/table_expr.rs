use crate::base::database::TableRef;
use serde::{Deserialize, Serialize};

/// Expression for an SQL table
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableExpr {
    /// TODO: add docs
    pub table_ref: TableRef,
}
