use crate::{
    base::{
        datafusion::{
            DataFusionProof::{
                self, ExecutionPlanProof as ExecutionPlanProofEnumVariant,
                PhysicalExprProof as PhysicalExprProofEnumVariant,
            },
            ExecutionPlanProof::TrivialProof as TrivialProofEnumVariant,
            Provable, ProvableExecutionPlan, ProvablePhysicalExpr,
        },
        proof::{
            Commit, Commitment, IntoDataFusionResult, IntoProofResult, PipProve, PipVerify,
            ProofError, ProofResult, Table, Transcript,
        },
    },
    datafusion_integration::wrappers::{
        unwrap_exec_plan_if_wrapped, wrap_exec_plan, wrap_physical_expr,
    },
    pip::execution_plans::TrivialProof,
};
use async_trait::async_trait;
use datafusion::{
    arrow::{array::ArrayRef, datatypes::SchemaRef, record_batch::RecordBatch},
    execution::context::TaskContext,
    physical_plan::{
        common::collect, expressions::PhysicalSortExpr, metrics::MetricsSet,
        projection::ProjectionExec, DisplayFormatType, ExecutionPlan, Partitioning, PhysicalExpr,
        SendableRecordBatchStream, Statistics,
    },
};
use std::sync::RwLock;
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    iter::Extend,
    sync::Arc,
};

type ExprTuple = (Arc<dyn PhysicalExpr>, String);
type ProvableExprTuple = (Arc<dyn ProvablePhysicalExpr>, String);

pub struct ProjectionExecWrapper {
    raw: ProjectionExec,
    /// The projection expressions stored as tuples of (expression, output column name)
    expr: Vec<ProvableExprTuple>,
    /// The input plan
    input: Arc<dyn ProvableExecutionPlan>,
    /// Same but as Arc<dyn ExecutionPlan> because trait upcast is unstable
    input_as_plan: Arc<dyn ExecutionPlan>,
    /// All the provables
    provable_children: Vec<Arc<dyn Provable>>,
    proof: RwLock<Option<Arc<DataFusionProof>>>,
    output: RwLock<Option<RecordBatch>>,
}

impl ProjectionExecWrapper {
    pub fn try_raw_spec(&self) -> ProofResult<ProjectionExec> {
        ProjectionExec::try_new(self.raw.expr().to_vec(), self.raw.input().clone())
            .into_proof_result()
    }

    pub fn try_new_from_raw(raw: &ProjectionExec) -> ProofResult<Self> {
        let raw_expr = raw.expr();
        let raw_input = raw.input();
        let (wrapped_input, wrapped_input_as_plan, wrapped_input_as_provable) =
            wrap_exec_plan(raw_input)?;
        let (wrapped_expr, expr_provable_children): (
            Vec<ProvableExprTuple>,
            Vec<Arc<dyn Provable>>,
        ) = raw_expr
            .iter()
            .map(|field| {
                let (physical_expr, physical_expr_as_provable) = wrap_physical_expr(&field.0)?;
                Ok((
                    (physical_expr.clone(), field.1.clone()),
                    physical_expr_as_provable.clone(),
                ))
            })
            .into_iter()
            .collect::<ProofResult<Vec<(ProvableExprTuple, Arc<dyn Provable>)>>>()?
            .into_iter()
            .unzip();
        let mut provable_children = vec![wrapped_input_as_provable];
        provable_children.extend(expr_provable_children);
        Ok(ProjectionExecWrapper {
            raw: ProjectionExec::try_new(raw_expr.to_vec(), raw_input.clone())
                .into_proof_result()?,
            expr: wrapped_expr.clone(),
            input: wrapped_input.clone(),
            input_as_plan: wrapped_input_as_plan.clone(),
            provable_children: provable_children.clone(),
            proof: RwLock::new(None),
            output: RwLock::new(None),
        })
    }

    pub fn try_new_from_children(
        expr: Vec<ProvableExprTuple>,
        input: Arc<dyn ProvableExecutionPlan>,
    ) -> ProofResult<Self> {
        let raw_expr: Vec<ExprTuple> = expr
            .iter()
            .map(|field| Ok((field.0.try_raw()?, field.1.clone())))
            .into_iter()
            .collect::<ProofResult<Vec<ExprTuple>>>()?;
        let raw = ProjectionExec::try_new(raw_expr, input.try_raw()?)?;
        Self::try_new_from_raw(&raw)
    }

    /// The projection expressions stored as tuples of (expression, output column name)
    pub fn expr(&self) -> &[ProvableExprTuple] {
        &self.expr
    }

    /// The input plan
    pub fn input(&self) -> &Arc<dyn ProvableExecutionPlan> {
        &self.input
    }
}

#[async_trait]
impl ProvableExecutionPlan for ProjectionExecWrapper {
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
        let input = self.input.output()?;
        for field in self.expr.iter() {
            field.0.evaluate(&input)?;
        }

        let stream: SendableRecordBatchStream = self
            .execute(partition, context.clone())
            .into_proof_result()?;
        let schema: SchemaRef = stream.schema();
        let output_batches = collect(stream).await.into_proof_result()?;
        let output = RecordBatch::concat(&schema, &output_batches[..]).into_proof_result()?;

        *self.output.write().into_proof_result()? = Some(output.clone());

        // Give the correct num_rows to the exprs so that they can generate ArrayRefs for the proofs
        // This has to be after execution
        for field in self.expr.iter() {
            field.0.set_num_rows(output.num_rows())?;
        }
        Ok(())
    }
    // Return output of an execution plan
    fn output(&self) -> ProofResult<RecordBatch> {
        (*self.output.read().into_proof_result()?)
            .clone()
            .ok_or(ProofError::UnexecutedError)
    }
}

impl Provable for ProjectionExecWrapper {
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
        let input: Vec<ArrayRef> = self
            .expr
            .iter()
            .map(|field| field.0.array_output())
            .into_iter()
            .collect::<ProofResult<Vec<ArrayRef>>>()?;
        let input_table = Table::try_from(&input)?;
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
                let c_expr: Vec<Commitment> = self
                    .expr
                    .iter()
                    .map(|field| -> ProofResult<Commitment> {
                        let proof: Arc<DataFusionProof> = (*field.0).get_proof()?;
                        match &*proof {
                            PhysicalExprProofEnumVariant(expr_proof) => {
                                expr_proof.get_output_commitments()
                            }
                            _ => Err(ProofError::TypeError),
                        }
                    })
                    .into_iter()
                    .collect::<ProofResult<Vec<Commitment>>>()?;
                p.verify(transcript, (c_expr,))
            }
            _ => Err(ProofError::TypeError),
        }
    }
}

impl ExecutionPlan for ProjectionExecWrapper {
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
        self.input.output_partitioning()
    }

    fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
        self.input.output_ordering()
    }

    fn maintains_input_order(&self) -> bool {
        // tell optimizer this operator doesn't reorder its input
        true
    }

    fn relies_on_input_order(&self) -> bool {
        false
    }

    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> datafusion::common::Result<Arc<dyn ExecutionPlan>> {
        let expr = self.raw.expr();
        let input = children[0].clone();
        let raw_input = unwrap_exec_plan_if_wrapped(&input).into_datafusion_result()?;
        let raw = ProjectionExec::try_new(expr.to_vec(), raw_input)?;
        Ok(Arc::new(
            ProjectionExecWrapper::try_new_from_raw(&raw).into_datafusion_result()?,
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

impl Debug for ProjectionExecWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectionExecWrapper")
            .field("raw", &self.raw)
            .finish()
    }
}
