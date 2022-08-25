use std::sync::Arc;

use datafusion::{
    physical_expr::{
        expressions::{BinaryExpr, Column, NegativeExpr},
        PhysicalExpr,
    },
    physical_plan::{
        coalesce_batches::CoalesceBatchesExec, coalesce_partitions::CoalescePartitionsExec,
        file_format::CsvExec, projection::ProjectionExec, repartition::RepartitionExec,
        ExecutionPlan,
    },
};

use crate::base::{
    datafusion::{Provable, ProvableExecutionPlan, ProvablePhysicalExpr},
    proof::{ProofError, ProofResult},
};

use super::{
    BinaryExprWrapper, CoalesceBatchesExecWrapper, CoalescePartitionsExecWrapper, ColumnWrapper,
    CsvExecWrapper, NegativeExprWrapper, ProjectionExecWrapper, RepartitionExecWrapper,
};

macro_rules! wrap_physical_expr_ind {
    ($any:expr, $raw:ty, $provable:ty) => {
        if $any.is::<$raw>() {
            let wrapped = Arc::new(<$provable>::try_new($any.downcast_ref::<$raw>().unwrap())?);
            return Ok((wrapped.clone(), wrapped));
        }
    };
}

pub fn wrap_physical_expr(
    expr: &Arc<dyn PhysicalExpr>,
) -> ProofResult<(Arc<dyn ProvablePhysicalExpr>, Arc<dyn Provable>)> {
    let any = (**expr).as_any();
    wrap_physical_expr_ind!(any, NegativeExpr, NegativeExprWrapper);
    wrap_physical_expr_ind!(any, Column, ColumnWrapper);
    wrap_physical_expr_ind!(any, BinaryExpr, BinaryExprWrapper);
    Err(ProofError::UnimplementedError)
}

macro_rules! wrap_exec_plan_ind {
    ($any:expr, $raw:ty, $provable:ty) => {
        if $any.is::<$raw>() {
            let wrapped = Arc::new(<$provable>::try_new_from_raw(
                $any.downcast_ref::<$raw>().unwrap(),
            )?);
            return Ok((wrapped.clone(), wrapped.clone(), wrapped));
        }
    };
}

#[allow(clippy::type_complexity)]
pub fn wrap_exec_plan(
    expr: &Arc<dyn ExecutionPlan>,
) -> ProofResult<(
    Arc<dyn ProvableExecutionPlan>,
    Arc<dyn ExecutionPlan>,
    Arc<dyn Provable>,
)> {
    let any = (**expr).as_any();
    wrap_exec_plan_ind!(any, ProjectionExec, ProjectionExecWrapper);
    wrap_exec_plan_ind!(any, CoalescePartitionsExec, CoalescePartitionsExecWrapper);
    wrap_exec_plan_ind!(any, CoalesceBatchesExec, CoalesceBatchesExecWrapper);
    wrap_exec_plan_ind!(any, RepartitionExec, RepartitionExecWrapper);
    wrap_exec_plan_ind!(any, CsvExec, CsvExecWrapper);
    Err(ProofError::UnimplementedError)
}

macro_rules! unwrap_exec_plan_if_wrapped_ind {
    ($any:expr, $raw:ty, $provable:ty) => {
        if $any.is::<$provable>() {
            let raw: $raw = (*$any.downcast_ref::<$provable>().unwrap()).raw_spec();
            return Ok(Arc::new(raw));
        }
    };
}

macro_rules! unwrap_exec_plan_if_wrapped_ind_try {
    ($any:expr, $raw:ty, $provable:ty) => {
        if $any.is::<$provable>() {
            let raw: $raw = (*$any.downcast_ref::<$provable>().unwrap()).try_raw_spec()?;
            return Ok(Arc::new(raw));
        }
    };
}

pub fn unwrap_exec_plan_if_wrapped(
    plan: &Arc<dyn ExecutionPlan>,
) -> ProofResult<Arc<dyn ExecutionPlan>> {
    let any = (**plan).as_any();
    unwrap_exec_plan_if_wrapped_ind_try!(any, ProjectionExec, ProjectionExecWrapper);
    unwrap_exec_plan_if_wrapped_ind!(any, CoalesceBatchesExec, CoalesceBatchesExecWrapper);
    unwrap_exec_plan_if_wrapped_ind!(any, CoalescePartitionsExec, CoalescePartitionsExecWrapper);
    unwrap_exec_plan_if_wrapped_ind_try!(any, RepartitionExec, RepartitionExecWrapper);
    unwrap_exec_plan_if_wrapped_ind!(any, CsvExec, CsvExecWrapper);
    Err(ProofError::TypeError)
}
