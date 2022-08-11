use crate::{
    base::{
        datafusion::{
            DataFusionProof::{self, ExecutionPlanProof as ExecutionPlanProofEnumVariant},
            ExecutionPlanProof::TrivialProof as TrivialProofEnumVariant,
            Provable, ProvableExecutionPlan,
        },
        proof::{
            Commit, Commitment, IntoDataFusionResult, IntoProofResult, PipProve, PipVerify,
            ProofError, ProofResult, Table, Transcript,
        },
    },
    datafusion_integration::wrappers::{unwrap_exec_plan_if_wrapped, wrap_exec_plan},
    pip::execution_plans::TrivialProof,
};
use async_trait::async_trait;
use datafusion::{
    arrow::{datatypes::SchemaRef, record_batch::RecordBatch},
    execution::context::TaskContext,
    physical_plan::{
        coalesce_batches::CoalesceBatchesExec, common::collect, expressions::PhysicalSortExpr,
        metrics::MetricsSet, DisplayFormatType, ExecutionPlan, Partitioning,
        SendableRecordBatchStream, Statistics,
    },
};
use std::sync::RwLock;
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    sync::Arc,
};

pub struct CoalesceBatchesExecWrapper {
    raw: CoalesceBatchesExec,
    /// The input plan
    input: Arc<dyn ProvableExecutionPlan>,
    /// Same but as Arc<dyn ExecutionPlan> because trait upcast is unstable
    input_as_plan: Arc<dyn ExecutionPlan>,
    /// All the provables
    provable_children: Vec<Arc<dyn Provable>>,
    proof: RwLock<Option<Arc<DataFusionProof>>>,
    output: RwLock<Option<RecordBatch>>,
}

impl CoalesceBatchesExecWrapper {
    pub fn raw_spec(&self) -> CoalesceBatchesExec {
        CoalesceBatchesExec::new(self.raw.input().clone(), self.raw.target_batch_size())
    }

    pub fn try_new_from_raw(raw: &CoalesceBatchesExec) -> ProofResult<Self> {
        let raw_input = raw.input();
        let target_batch_size = raw.target_batch_size();
        let (wrapped_input, wrapped_input_as_plan, wrapped_input_as_provable) =
            wrap_exec_plan(raw_input)?;
        Ok(CoalesceBatchesExecWrapper {
            raw: CoalesceBatchesExec::new(raw_input.clone(), target_batch_size),
            input: wrapped_input.clone(),
            input_as_plan: wrapped_input_as_plan.clone(),
            provable_children: vec![wrapped_input_as_provable],
            proof: RwLock::new(None),
            output: RwLock::new(None),
        })
    }

    pub fn try_new_from_children(
        input: Arc<dyn ProvableExecutionPlan>,
        target_batch_size: usize,
    ) -> ProofResult<Self> {
        let raw = CoalesceBatchesExec::new(input.try_raw()?, target_batch_size);
        Self::try_new_from_raw(&raw)
    }

    /// The input plan
    pub fn input(&self) -> &Arc<dyn ProvableExecutionPlan> {
        &self.input
    }

    /// Minimum number of rows for coalesces batches
    pub fn target_batch_size(&self) -> usize {
        self.raw.target_batch_size()
    }
}

#[async_trait]
impl ProvableExecutionPlan for CoalesceBatchesExecWrapper {
    fn try_raw(&self) -> ProofResult<Arc<dyn ExecutionPlan>> {
        Ok(Arc::new(self.raw_spec()))
    }
    // Compute output of an execution plan and store it
    async fn execute_and_collect(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> ProofResult<()> {
        self.input
            .execute_and_collect(partition, context.clone())
            .await?;
        let stream: SendableRecordBatchStream = self
            .execute(partition, context.clone())
            .into_proof_result()?;
        let schema: SchemaRef = stream.schema();
        let output_batches = collect(stream).await.into_proof_result()?;
        let output = RecordBatch::concat(&schema, &output_batches[..]).into_proof_result()?;
        *self.output.write().into_proof_result()? = Some(output);
        Ok(())
    }
    // Return output of an execution plan
    fn output(&self) -> ProofResult<RecordBatch> {
        (*self.output.read().into_proof_result()?)
            .clone()
            .ok_or(ProofError::UnexecutedError)
    }
}

impl Provable for CoalesceBatchesExecWrapper {
    fn children(&self) -> &[Arc<dyn Provable>] {
        &self.provable_children[..]
    }
    fn get_proof(&self) -> ProofResult<Arc<DataFusionProof>> {
        (*self.proof.read().into_proof_result()?)
            .clone()
            .ok_or(ProofError::NoProofError)
    }
    fn set_proof(&self, proof: &Arc<DataFusionProof>) -> ProofResult<()> {
        let typed_proof: &TrivialProof = match &**proof {
            ExecutionPlanProofEnumVariant(TrivialProofEnumVariant(p)) => p,
            _ => return Err(ProofError::TypeError),
        };
        *self.proof.write().into_proof_result()? = Some(Arc::new(ExecutionPlanProofEnumVariant(
            TrivialProofEnumVariant((*typed_proof).clone()),
        )));
        Ok(())
    }
    fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()> {
        let input_table = Table::try_from(&self.input.output()?)?;
        let output_table = Table::try_from(&self.output()?)?;
        let c_in: Vec<Commitment> = input_table.commit();
        let proof = TrivialProof::prove(transcript, (input_table,), output_table, (c_in,));
        *self.proof.write().into_proof_result()? = Some(Arc::new(ExecutionPlanProofEnumVariant(
            TrivialProofEnumVariant(proof),
        )));
        Ok(())
    }
    fn run_verify(&self, transcript: &mut Transcript) -> ProofResult<()> {
        let proof = self.get_proof()?;
        match &*proof {
            ExecutionPlanProofEnumVariant(TrivialProofEnumVariant(p)) => {
                let input_proof: Arc<DataFusionProof> = self.input.get_proof()?;
                let c_in: Vec<Commitment> = match &*input_proof {
                    ExecutionPlanProofEnumVariant(exec_proof) => {
                        exec_proof.get_output_commitments()
                    }
                    _ => Err(ProofError::TypeError),
                }?;
                p.verify(transcript, (c_in,))
            }
            _ => Err(ProofError::TypeError),
        }
    }
}

impl ExecutionPlan for CoalesceBatchesExecWrapper {
    fn as_any(&self) -> &dyn Any {
        self
    }

    /// Get the schema for this execution plan
    fn schema(&self) -> SchemaRef {
        self.raw.schema()
    }

    fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
        vec![self.input_as_plan.clone()]
    }

    /// Get the output partitioning of this plan
    fn output_partitioning(&self) -> Partitioning {
        // The coalesce batches operator does not make any changes to the partitioning of its input
        self.input.output_partitioning()
    }

    fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
        None
    }

    fn relies_on_input_order(&self) -> bool {
        false
    }

    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> datafusion::common::Result<Arc<dyn ExecutionPlan>> {
        let input = children[0].clone();
        let raw_input = unwrap_exec_plan_if_wrapped(&input).into_datafusion_result()?;
        let raw = CoalesceBatchesExec::new(raw_input, self.raw.target_batch_size());
        Ok(Arc::new(
            CoalesceBatchesExecWrapper::try_new_from_raw(&raw).into_datafusion_result()?,
        ))
    }

    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> datafusion::common::Result<SendableRecordBatchStream> {
        self.raw.execute(partition, context)
    }

    fn fmt_as(&self, t: DisplayFormatType, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.raw.fmt_as(t, f)
    }

    fn metrics(&self) -> Option<MetricsSet> {
        self.raw.metrics()
    }

    fn statistics(&self) -> Statistics {
        self.raw.statistics()
    }
}

impl Debug for CoalesceBatchesExecWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoalesceBatchesExecWrapper")
            .field("raw", &self.raw)
            .finish()
    }
}
