use crate::{
    base::{
        datafusion::{
            impl_debug_for_provable, impl_execution_plan_for_provable, impl_provable,
            DataFusionProof::{
                self, ExecutionPlanProof as ExecutionPlanProofEnumVariant,
                PhysicalExprProof as PhysicalExprProofEnumVariant,
            },
            ExecutionPlanProof::TrivialProof as TrivialProofEnumVariant,
            PhysicalExprTuple, Provable, ProvableExecutionPlan, ProvablePhysicalExprTuple,
        },
        proof::{
            Commitment, IntoDataFusionResult, IntoProofResult, PipProve, PipVerify, ProofError,
            ProofResult, Table, Transcript,
        },
    },
    datafusion_integration::wrappers::{
        unwrap_exec_plan_if_wrapped, wrap_exec_plan, wrap_physical_expr,
    },
    pip::execution_plan::TrivialProof,
};
use async_trait::async_trait;
use datafusion::{
    arrow::{array::ArrayRef, datatypes::SchemaRef, record_batch::RecordBatch},
    execution::context::TaskContext,
    physical_plan::{
        common::collect, expressions::PhysicalSortExpr, metrics::MetricsSet,
        projection::ProjectionExec, DisplayFormatType, Distribution, ExecutionPlan, Partitioning,
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

pub struct ProjectionExecWrapper {
    raw: ProjectionExec,
    /// The projection expressions stored as tuples of (expression, output column name)
    expr: Vec<ProvablePhysicalExprTuple>,
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
            Vec<ProvablePhysicalExprTuple>,
            Vec<Arc<dyn Provable>>,
        ) = raw_expr
            .iter()
            .map(|field| {
                let (physical_expr, _, physical_expr_as_provable) = wrap_physical_expr(&field.0)?;
                Ok((
                    (physical_expr.clone(), field.1.clone()),
                    physical_expr_as_provable.clone(),
                ))
            })
            .into_iter()
            .collect::<ProofResult<Vec<(ProvablePhysicalExprTuple, Arc<dyn Provable>)>>>()?
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
        expr: Vec<ProvablePhysicalExprTuple>,
        input: Arc<dyn ProvableExecutionPlan>,
    ) -> ProofResult<Self> {
        let raw_expr: Vec<PhysicalExprTuple> = expr
            .iter()
            .map(|field| Ok((field.0.try_raw()?, field.1.clone())))
            .into_iter()
            .collect::<ProofResult<Vec<PhysicalExprTuple>>>()?;
        let raw = ProjectionExec::try_new(raw_expr, input.try_raw()?)?;
        Self::try_new_from_raw(&raw)
    }

    /// The projection expressions stored as tuples of (expression, output column name)
    pub fn expr(&self) -> &[ProvablePhysicalExprTuple] {
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
            field.0.set_num_rows(output.clone().num_rows())?;
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
    impl_provable!(
        TrivialProof,
        ExecutionPlanProofEnumVariant,
        TrivialProofEnumVariant
    );
    fn children(&self) -> &[Arc<dyn Provable>] {
        &self.provable_children[..]
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

        let c_expr: Vec<Commitment> = self
            .expr
            .iter()
            .map(|field| -> ProofResult<Commitment> {
                let proof: Arc<DataFusionProof> = (*field.0).get_proof()?;
                match &*proof {
                    PhysicalExprProofEnumVariant(expr_proof) => expr_proof.get_output_commitments(),
                    _ => Err(ProofError::TypeError),
                }
            })
            .into_iter()
            .collect::<ProofResult<Vec<Commitment>>>()?;

        let proof = TrivialProof::prove(transcript, (input_table,), output_table, (c_expr,));
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
    impl_execution_plan_for_provable!();
    fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
        vec![self.input_as_plan.clone()]
    }
    fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
        self.input.output_ordering()
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
}

impl_debug_for_provable!(ProjectionExecWrapper);
