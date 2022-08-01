use std::sync::Arc;

use datafusion::physical_expr::{
    expressions::{Column, NegativeExpr},
    PhysicalExpr,
};

use crate::base::{
    datafusion::{Provable, ProvablePhysicalExpr},
    proof::{ProofError, ProofResult},
};

use super::{ColumnWrapper, NegativeExprWrapper}; //, ProjectionExecWrapper};

pub fn wrap_physical_expr(
    expr: &Arc<dyn PhysicalExpr>,
) -> ProofResult<(Arc<dyn ProvablePhysicalExpr>, Arc<dyn Provable>)> {
    let any = (**expr).as_any();
    if any.is::<NegativeExpr>() {
        let wrapped = Arc::new(NegativeExprWrapper::try_new(
            any.downcast_ref::<NegativeExpr>().unwrap(),
        )?);
        return Ok((wrapped.clone(), wrapped));
    }
    if any.is::<Column>() {
        let wrapped = Arc::new(ColumnWrapper::try_new(
            any.downcast_ref::<Column>().unwrap(),
        )?);
        return Ok((wrapped.clone(), wrapped));
    }
    Err(ProofError::UnimplementedError)
}
