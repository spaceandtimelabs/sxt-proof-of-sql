use super::DynProofExpr;
use crate::base::commitment::Commitment;
use sqlparser::ast::Ident;
use serde::{Deserialize, Serialize};

/// A `DynProofExpr` with an alias.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AliasedDynProofExpr<C: Commitment> {
    pub expr: DynProofExpr<C>,
    pub alias: Ident,
}
