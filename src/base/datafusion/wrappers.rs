use crate::{
    base::{
        datafusion::Provable,
        proof::{IntoProofResult, ProofResult},
    },
    datafusion_integration::CoalescePartitionsExecWrapper,
};
use async_trait::async_trait;
use datafusion::{
    arrow::{array::ArrayRef, record_batch::RecordBatch},
    execution::context::TaskContext,
    physical_expr::PhysicalExpr,
    physical_plan::ExecutionPlan,
};
use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

pub trait ProvablePhysicalExpr: PhysicalExpr + Provable + Debug + Display {
    // Return the raw expression
    fn try_raw(&self) -> ProofResult<Arc<dyn PhysicalExpr>>;
    // Set num of rows to convert ScalarValues into ArrayRefs
    fn set_num_rows(&self, num_rows: usize) -> ProofResult<()>;
    // Output of a physical expression as ArrayRef
    fn array_output(&self) -> ProofResult<ArrayRef>;
}

#[async_trait]
pub trait ProvableExecutionPlan: ExecutionPlan + Provable + Debug {
    // Return the raw plan
    fn try_raw(&self) -> ProofResult<Arc<dyn ExecutionPlan>>;
    // Compute output of an execution plan and store it
    async fn execute_and_collect(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> ProofResult<()>;
    // Return output of an execution plan
    fn output(&self) -> ProofResult<RecordBatch>;
}

/// Execute the [ProvableExecutionPlan] and collect the results in memory
pub async fn collect(
    plan: &Arc<dyn ProvableExecutionPlan>,
    context: Arc<TaskContext>,
) -> ProofResult<RecordBatch> {
    match (*plan).output_partitioning().partition_count() {
        0 => RecordBatch::try_new((*plan).schema(), vec![]).into_proof_result(),
        1 => {
            (*plan).execute_and_collect(0, context).await?;
            (*plan).output()
        }
        _ => {
            // merge into a single partition
            let new_plan = CoalescePartitionsExecWrapper::try_new_from_children((*plan).clone())?;
            // CoalescePartitionsExecWrapper must produce a single partition
            assert_eq!(1, new_plan.output_partitioning().partition_count());
            new_plan.execute_and_collect(0, context).await?;
            new_plan.output()
        }
    }
}
