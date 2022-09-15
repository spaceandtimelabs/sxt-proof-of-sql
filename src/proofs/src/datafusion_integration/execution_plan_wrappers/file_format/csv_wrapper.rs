use crate::{
    base::{
        datafusion::{
            impl_debug_for_provable, impl_execution_plan_for_provable, impl_provable,
            DataFusionProof::{self, ExecutionPlanProof as ExecutionPlanProofEnumVariant},
            ExecutionPlanProof::ReaderProof as ReaderProofEnumVariant,
            Provable, ProvableExecutionPlan,
        },
        proof::{IntoProofResult, PipProve, PipVerify, ProofError, ProofResult, Table, Transcript},
    },
    pip::execution_plan::ReaderProof,
};
use async_trait::async_trait;
use datafusion::{
    arrow::{datatypes::SchemaRef, record_batch::RecordBatch},
    execution::context::TaskContext,
    physical_plan::{
        common::collect,
        expressions::PhysicalSortExpr,
        file_format::{CsvExec, FileScanConfig},
        metrics::MetricsSet,
        DisplayFormatType, Distribution, ExecutionPlan, Partitioning, SendableRecordBatchStream,
        Statistics,
    },
};
use std::sync::RwLock;
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    stringify,
    sync::Arc,
};

pub struct CsvExecWrapper {
    raw: CsvExec,
    proof: RwLock<Option<Arc<DataFusionProof>>>,
    output: RwLock<Option<RecordBatch>>,
}

impl CsvExecWrapper {
    pub fn raw_spec(&self) -> CsvExec {
        CsvExec::new(
            self.raw.base_config().clone(),
            self.raw.has_header(),
            self.raw.delimiter(),
        )
    }

    pub fn try_new_from_raw(raw: &CsvExec) -> ProofResult<Self> {
        Ok(CsvExecWrapper {
            raw: CsvExec::new(raw.base_config().clone(), raw.has_header(), raw.delimiter()),
            proof: RwLock::new(None),
            output: RwLock::new(None),
        })
    }

    /// Ref to the base configs
    pub fn base_config(&self) -> &FileScanConfig {
        self.raw.base_config()
    }
    /// true if the first line of each file is a header
    pub fn has_header(&self) -> bool {
        self.raw.has_header()
    }
    /// A column delimiter
    pub fn delimiter(&self) -> u8 {
        self.raw.delimiter()
    }
}

#[async_trait]
impl ProvableExecutionPlan for CsvExecWrapper {
    fn try_raw(&self) -> ProofResult<Arc<dyn ExecutionPlan>> {
        Ok(Arc::new(self.raw_spec()))
    }
    // Compute output of an execution plan and store it
    async fn execute_and_collect(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> ProofResult<()> {
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

impl Provable for CsvExecWrapper {
    impl_provable!(
        ReaderProof,
        ExecutionPlanProofEnumVariant,
        ReaderProofEnumVariant
    );

    fn children(&self) -> &[Arc<dyn Provable>] {
        &[]
    }
    fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()> {
        let output_table = Table::try_from(&self.output()?)?;
        let proof = ReaderProof::prove(transcript, (), output_table, ());
        *self.proof.write().into_proof_result()? = Some(Arc::new(ExecutionPlanProofEnumVariant(
            ReaderProofEnumVariant(proof),
        )));
        Ok(())
    }
    fn run_verify(&self, transcript: &mut Transcript) -> ProofResult<()> {
        let proof = self.get_proof()?;
        match &*proof {
            ExecutionPlanProofEnumVariant(ReaderProofEnumVariant(p)) => p.verify(transcript, ()),
            _ => Err(ProofError::TypeError),
        }
    }
}

impl ExecutionPlan for CsvExecWrapper {
    impl_execution_plan_for_provable!();
    fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
        // this is a leaf node and has no children
        vec![]
    }
    fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
        None
    }
    fn with_new_children(
        self: Arc<Self>,
        _: Vec<Arc<dyn ExecutionPlan>>,
    ) -> datafusion::common::Result<Arc<dyn ExecutionPlan>> {
        Ok(self)
    }
}

impl_debug_for_provable!(CsvExecWrapper);
