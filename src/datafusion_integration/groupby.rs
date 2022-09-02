use crate::{
    base::{
        datafusion::{PhysicalExprTuple, ProvablePhysicalExprTuple},
        proof::ProofResult,
    },
    datafusion_integration::wrappers::{unwrap_vec_physicalexprtuple, wrap_vec_physicalexprtuple},
};
use datafusion::physical_plan::aggregates::PhysicalGroupBy;
use std::fmt::Debug;

#[derive(Debug)]
pub struct ProvablePhysicalGroupBy {
    /// Raw PhysicalGroupBy underneath it
    raw: PhysicalGroupBy,
    /// Distinct (Physical Expr, Alias) in the grouping set
    expr: Vec<ProvablePhysicalExprTuple>,
    /// Corresponding NULL expressions for expr
    null_expr: Vec<ProvablePhysicalExprTuple>,
}

impl ProvablePhysicalGroupBy {
    pub fn raw(&self) -> PhysicalGroupBy {
        self.raw.clone()
    }

    /// Create a new `ProvablePhysicalGroupBy`
    pub fn try_new(raw: &PhysicalGroupBy) -> ProofResult<Self> {
        let wrapped_expr: Vec<ProvablePhysicalExprTuple> = wrap_vec_physicalexprtuple(raw.expr())?;
        let wrapped_null_expr: Vec<ProvablePhysicalExprTuple> =
            wrap_vec_physicalexprtuple(raw.null_expr())?;
        Ok(Self {
            raw: raw.clone(),
            expr: wrapped_expr,
            null_expr: wrapped_null_expr,
        })
    }

    /// Create a GROUPING SET with only a single group. This is the "standard"
    /// case when building a plan from an expression such as `GROUP BY a,b,c`
    pub fn try_new_single(expr: Vec<ProvablePhysicalExprTuple>) -> ProofResult<Self> {
        let raw_expr: Vec<PhysicalExprTuple> = unwrap_vec_physicalexprtuple(&expr)?;
        let raw = PhysicalGroupBy::new_single(raw_expr);
        Self::try_new(&raw)
    }

    /// Returns true if this GROUP BY contains NULL expressions
    pub fn contains_null(&self) -> bool {
        self.raw.contains_null()
    }

    /// Returns the group expressions
    pub fn expr(&self) -> &[ProvablePhysicalExprTuple] {
        &self.expr
    }

    /// Returns the null expressions
    pub fn null_expr(&self) -> &[ProvablePhysicalExprTuple] {
        &self.null_expr
    }

    /// Returns the group null masks
    pub fn groups(&self) -> &[Vec<bool>] {
        self.raw.groups()
    }

    /// Returns true if this `PhysicalGroupBy` has no group expressions
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }
}
