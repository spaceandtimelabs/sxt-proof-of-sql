use std::sync::Arc;

use datafusion::{
    physical_expr::{
        expressions::{BinaryExpr, Column, Literal, NegativeExpr},
        AggregateExpr, PhysicalExpr,
    },
    physical_plan::{
        aggregates::AggregateExec, coalesce_batches::CoalesceBatchesExec,
        coalesce_partitions::CoalescePartitionsExec, expressions::Count, file_format::CsvExec,
        projection::ProjectionExec, repartition::RepartitionExec, ExecutionPlan,
    },
};

use crate::base::{
    datafusion::{
        PhysicalExprTuple, Provable, ProvableAggregateExpr, ProvableExecutionPlan,
        ProvablePhysicalExpr, ProvablePhysicalExprTuple,
    },
    proof::{ProofError, ProofResult},
};

use super::{
    AggregateExecWrapper, BinaryExprWrapper, CoalesceBatchesExecWrapper,
    CoalescePartitionsExecWrapper, ColumnWrapper, CountWrapper, CsvExecWrapper, LiteralWrapper,
    NegativeExprWrapper, ProjectionExecWrapper, RepartitionExecWrapper,
};

macro_rules! wrap_aggregate_expr_ind {
    ($any:expr, $raw:ty, $provable:ty) => {
        if $any.is::<$raw>() {
            let wrapped = Arc::new(<$provable>::try_new($any.downcast_ref::<$raw>().unwrap())?);
            return Ok((wrapped.clone(), wrapped.clone(), wrapped));
        }
    };
}

#[allow(clippy::type_complexity)]
pub fn wrap_aggregate_expr(
    expr: &Arc<dyn AggregateExpr>,
) -> ProofResult<(
    Arc<dyn ProvableAggregateExpr>,
    Arc<dyn AggregateExpr>,
    Arc<dyn Provable>,
)> {
    let any = (**expr).as_any();
    wrap_aggregate_expr_ind!(any, Count, CountWrapper);
    Err(ProofError::GeneralError)
}

macro_rules! wrap_physical_expr_ind {
    ($any:expr, $raw:ty, $provable:ty) => {
        if $any.is::<$raw>() {
            let wrapped = Arc::new(<$provable>::try_new($any.downcast_ref::<$raw>().unwrap())?);
            return Ok((wrapped.clone(), wrapped.clone(), wrapped));
        }
    };
}

#[allow(clippy::type_complexity)]
pub fn wrap_physical_expr(
    expr: &Arc<dyn PhysicalExpr>,
) -> ProofResult<(
    Arc<dyn ProvablePhysicalExpr>,
    Arc<dyn PhysicalExpr>,
    Arc<dyn Provable>,
)> {
    let any = (**expr).as_any();
    wrap_physical_expr_ind!(any, Column, ColumnWrapper);
    wrap_physical_expr_ind!(any, Literal, LiteralWrapper);
    wrap_physical_expr_ind!(any, NegativeExpr, NegativeExprWrapper);
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
    wrap_exec_plan_ind!(any, AggregateExec, AggregateExecWrapper);
    Err(ProofError::UnexecutedError)
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
    unwrap_exec_plan_if_wrapped_ind_try!(any, AggregateExec, AggregateExecWrapper);
    Err(ProofError::TypeError)
}

pub fn wrap_vec_physicalexprtuple(
    raw: &[PhysicalExprTuple],
) -> ProofResult<Vec<ProvablePhysicalExprTuple>> {
    raw.iter()
        .map(|field| Ok((wrap_physical_expr(&field.0)?.0, field.1.clone())))
        .into_iter()
        .collect::<ProofResult<Vec<ProvablePhysicalExprTuple>>>()
}

pub fn unwrap_vec_physicalexprtuple(
    wrapped: &[ProvablePhysicalExprTuple],
) -> ProofResult<Vec<PhysicalExprTuple>> {
    wrapped
        .iter()
        .map(|field| Ok((field.0.try_raw()?, field.1.clone())))
        .into_iter()
        .collect::<ProofResult<Vec<PhysicalExprTuple>>>()
}
