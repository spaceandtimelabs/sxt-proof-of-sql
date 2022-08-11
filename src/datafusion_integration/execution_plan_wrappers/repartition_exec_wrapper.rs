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
        common::collect, expressions::PhysicalSortExpr, metrics::MetricsSet,
        repartition::RepartitionExec, DisplayFormatType, ExecutionPlan, Partitioning,
        SendableRecordBatchStream, Statistics,
    },
};
use std::sync::RwLock;
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    sync::Arc,
};

pub struct RepartitionExecWrapper {
    raw: RepartitionExec,
    /// The input plan
    input: Arc<dyn ProvableExecutionPlan>,
    /// Same but as Arc<dyn ExecutionPlan> because trait upcast is unstable
    input_as_plan: Arc<dyn ExecutionPlan>,
    /// All the provables
    provable_children: Vec<Arc<dyn Provable>>,
    proof: RwLock<Option<Arc<DataFusionProof>>>,
    output: RwLock<Option<RecordBatch>>,
}

impl RepartitionExecWrapper {
    pub fn try_raw_spec(&self) -> ProofResult<RepartitionExec> {
        RepartitionExec::try_new(self.raw.input().clone(), self.raw.partitioning().clone())
            .into_proof_result()
    }

    pub fn try_new_from_raw(raw: &RepartitionExec) -> ProofResult<Self> {
        let raw_input = raw.input();
        let partitioning = raw.partitioning();
        let (wrapped_input, wrapped_input_as_plan, wrapped_input_as_provable) =
            wrap_exec_plan(raw_input)?;
        Ok(RepartitionExecWrapper {
            raw: RepartitionExec::try_new(raw_input.clone(), partitioning.clone())?,
            input: wrapped_input.clone(),
            input_as_plan: wrapped_input_as_plan.clone(),
            provable_children: vec![wrapped_input_as_provable],
            proof: RwLock::new(None),
            output: RwLock::new(None),
        })
    }

    pub fn try_new_from_children(
        input: Arc<dyn ProvableExecutionPlan>,
        partitioning: Partitioning,
    ) -> ProofResult<Self> {
        let raw = RepartitionExec::try_new(input.try_raw()?, partitioning)?;
        Self::try_new_from_raw(&raw)
    }

    /// The input plan
    pub fn input(&self) -> &Arc<dyn ProvableExecutionPlan> {
        &self.input
    }

    /// Partitioning scheme to use
    pub fn partitioning(&self) -> &Partitioning {
        self.raw.partitioning()
    }
}

#[async_trait]
impl ProvableExecutionPlan for RepartitionExecWrapper {
    fn try_raw(&self) -> ProofResult<Arc<dyn ExecutionPlan>> {
        Ok(Arc::new(self.try_raw_spec()?))
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

impl Provable for RepartitionExecWrapper {
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

impl ExecutionPlan for RepartitionExecWrapper {
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

    fn relies_on_input_order(&self) -> bool {
        false
    }

    /// Get the output partitioning of this plan
    fn output_partitioning(&self) -> Partitioning {
        self.raw.partitioning().clone()
    }

    fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
        None
    }

    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> datafusion::common::Result<Arc<dyn ExecutionPlan>> {
        let input = children[0].clone();
        let raw_input = unwrap_exec_plan_if_wrapped(&input).into_datafusion_result()?;
        let raw = RepartitionExec::try_new(raw_input, self.raw.partitioning().clone())?;
        Ok(Arc::new(
            RepartitionExecWrapper::try_new_from_raw(&raw).into_datafusion_result()?,
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

impl Debug for RepartitionExecWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RepartitionExecWrapper")
            .field("raw", &self.raw)
            .finish()
    }
}
