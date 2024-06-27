use super::ProvableExprPlan;
use crate::base::commitment::Commitment;
use proof_of_sql_parser::Identifier;
use serde::{Deserialize, Serialize};

/// A `ProvableExprPlan` with an alias.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AliasedProvableExprPlan<C: Commitment> {
    pub expr: ProvableExprPlan<C>,
    pub alias: Identifier,
}
