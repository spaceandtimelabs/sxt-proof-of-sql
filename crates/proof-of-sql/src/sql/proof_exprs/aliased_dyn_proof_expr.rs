use super::DynProofExpr;
use proof_of_sql_parser::Identifier;
use serde::{Deserialize, Serialize};

/// A `DynProofExpr` with an alias.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AliasedDynProofExpr {
    pub expr: DynProofExpr,
    pub alias: Identifier,
}
