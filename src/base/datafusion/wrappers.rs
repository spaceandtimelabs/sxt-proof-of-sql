use crate::base::{datafusion::Provable, proof::ProofResult};
use datafusion::{
    physical_expr::PhysicalExpr,
    physical_plan::{ColumnarValue, ExecutionPlan},
};
use std::fmt::{Debug, Display};

pub trait ProvablePhysicalExpr: PhysicalExpr + Provable + Debug + Display {
    // Output of a physical expression
    fn output(&self) -> ProofResult<ColumnarValue>;
}

pub trait ProvableExecutionPlan: ExecutionPlan + Provable + Debug + Display {}
