use crate::{
    base::{
        datafusion::{
            impl_debug_for_provable, impl_execution_plan_for_provable, impl_provable,
            DataFusionProof::{
                self, AggregateExprProof as AggregateExprProofEnumVariant,
                ExecutionPlanProof as ExecutionPlanProofEnumVariant,
            },
            ExecutionPlanProof::TrivialProof as TrivialProofEnumVariant,
            Provable, ProvableAggregateExpr, ProvableExecutionPlan, ProvablePhysicalExpr,
        },
        proof::{
            Commit, Commitment, IntoDataFusionResult, IntoProofResult, PipProve, PipVerify,
            ProofError, ProofResult, Table, Transcript,
        },
    },
    datafusion_integration::{
        wrappers::{unwrap_exec_plan_if_wrapped, wrap_aggregate_expr, wrap_exec_plan},
        ColumnWrapper, ProvablePhysicalGroupBy,
    },
    pip::execution_plan::TrivialProof,
};
use async_trait::async_trait;
use datafusion::{
    arrow::{array::ArrayRef, datatypes::SchemaRef, record_batch::RecordBatch},
    execution::context::TaskContext,
    physical_plan::{
        aggregates::{AggregateExec, AggregateMode},
        common::collect,
        expressions::{Column as ColumnExpr, PhysicalSortExpr},
        metrics::MetricsSet,
        AggregateExpr, ColumnarValue, DisplayFormatType, Distribution, ExecutionPlan, Partitioning,
        SendableRecordBatchStream, Statistics,
    },
};
use std::sync::RwLock;
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    iter::Extend,
    stringify,
    sync::Arc,
};

pub struct AggregateExecWrapper {
    raw: AggregateExec,
    /// Group by expressions
    group_by: ProvablePhysicalGroupBy,
    /// Aggregate expressions
    aggr_expr: Vec<Arc<dyn ProvableAggregateExpr>>,
    /// Input plan, could be a partial aggregate or the input to the aggregate
    input: Arc<dyn ProvableExecutionPlan>,
    input_as_plan: Arc<dyn ExecutionPlan>,
    /// All the provables
    provable_children: Vec<Arc<dyn Provable>>,
    proof: RwLock<Option<Arc<DataFusionProof>>>,
    output: RwLock<Option<RecordBatch>>,
}

impl AggregateExecWrapper {
    pub fn try_raw_spec(&self) -> ProofResult<AggregateExec> {
        AggregateExec::try_new(
            *self.raw.mode(),
            self.raw.group_expr().clone(),
            self.raw.aggr_expr().to_vec(),
            self.raw.input().clone(),
            self.raw.input_schema(),
        )
        .into_proof_result()
    }

    #[allow(clippy::type_complexity)]
    pub fn try_new_from_raw(raw: &AggregateExec) -> ProofResult<Self> {
        let raw_groupby = raw.group_expr();
        // There is no groupby proof yet so it has to be empty.
        if !raw_groupby.is_empty() {
            return Err(ProofError::UnimplementedError);
        }
        let wrapped_groupby = ProvablePhysicalGroupBy::try_new(raw_groupby)?;
        let (wrapped_input, wrapped_input_as_plan, wrapped_input_as_provable) =
            wrap_exec_plan(raw.input())?;
        let (wrapped_aggr_expr, wrapped_aggr_expr_as_provable): (
            Vec<Arc<dyn ProvableAggregateExpr>>,
            Vec<Arc<dyn Provable>>,
        ) = raw
            .aggr_expr()
            .iter()
            .map(|expr| {
                let (wrapped_ind_aggr, _, wrapped_ind_aggr_as_provable) =
                    wrap_aggregate_expr(expr)?;
                Ok((wrapped_ind_aggr, wrapped_ind_aggr_as_provable))
            })
            .into_iter()
            .collect::<ProofResult<Vec<(Arc<dyn ProvableAggregateExpr>, Arc<dyn Provable>)>>>()?
            .iter()
            .cloned()
            .unzip();
        let mut provable_children: Vec<Arc<dyn Provable>> = vec![wrapped_input_as_provable];

        if *raw.mode() == AggregateMode::Partial {
            provable_children.extend(wrapped_aggr_expr_as_provable);
        }

        Ok(AggregateExecWrapper {
            raw: AggregateExec::try_new(
                *raw.mode(),
                raw_groupby.clone(),
                raw.aggr_expr().to_vec(),
                raw.input().clone(),
                raw.input_schema(),
            )?,
            group_by: wrapped_groupby,
            aggr_expr: wrapped_aggr_expr.clone(),
            input: wrapped_input.clone(),
            input_as_plan: wrapped_input_as_plan.clone(),
            provable_children: provable_children.clone(),
            proof: RwLock::new(None),
            output: RwLock::new(None),
        })
    }

    pub fn try_new_from_children(
        mode: AggregateMode,
        group_by: ProvablePhysicalGroupBy,
        aggr_expr: Vec<Arc<dyn ProvableAggregateExpr>>,
        input: Arc<dyn ProvableExecutionPlan>,
        input_schema: SchemaRef,
    ) -> ProofResult<Self> {
        let raw_groupby = group_by.raw();
        let raw_aggr_expr: Vec<Arc<dyn AggregateExpr>> = aggr_expr
            .iter()
            .map(|expr| expr.try_raw())
            .into_iter()
            .collect::<ProofResult<Vec<Arc<dyn AggregateExpr>>>>()?;
        let raw_input = (&*input).try_raw()?;
        let raw =
            AggregateExec::try_new(mode, raw_groupby, raw_aggr_expr, raw_input, input_schema)?;
        Self::try_new_from_raw(&raw)
    }

    pub fn mode(&self) -> &AggregateMode {
        self.raw.mode()
    }

    /// Grouping expressions
    pub fn group_expr(&self) -> &ProvablePhysicalGroupBy {
        &self.group_by
    }

    /// Grouping expressions as they occur in the output schema
    pub fn output_group_expr(&self) -> Vec<Arc<dyn ProvablePhysicalExpr>> {
        // ColumnWrapper always returns Ok so it is safe to unwrap
        self.group_by
            .expr()
            .iter()
            .enumerate()
            .map(|(index, (_col, name))| {
                Arc::new(ColumnWrapper::try_new(&ColumnExpr::new(name, index)).unwrap())
                    as Arc<dyn ProvablePhysicalExpr>
            })
            .collect()
    }

    pub fn aggr_expr(&self) -> &[Arc<dyn ProvableAggregateExpr>] {
        &self.aggr_expr
    }

    pub fn input(&self) -> &Arc<dyn ProvableExecutionPlan> {
        &self.input
    }

    pub fn input_schema(&self) -> SchemaRef {
        self.raw.input_schema()
    }
}

#[async_trait]
impl ProvableExecutionPlan for AggregateExecWrapper {
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

        // Once we have groupby we use groupby results instead
        let input = self.input.output()?;

        let stream: SendableRecordBatchStream = self
            .execute(partition, context.clone())
            .into_proof_result()?;
        let schema: SchemaRef = stream.schema();
        let output_batches = collect(stream).await.into_proof_result()?;
        let output = RecordBatch::concat(&schema, &output_batches[..]).into_proof_result()?;

        *self.output.write().into_proof_result()? = Some(output.clone());
        let num_aggr_exprs = self.aggr_expr.len();

        // Pass results back to aggr_exprs
        for i in 0..num_aggr_exprs {
            let array = output.column(i);
            self.aggr_expr[i].set_output(&ColumnarValue::Array(array.clone()))?;
            self.aggr_expr[i].evaluate_and_set_num_rows_for_physicals(&input)?;
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

impl Provable for AggregateExecWrapper {
    impl_provable!(
        TrivialProof,
        ExecutionPlanProofEnumVariant,
        TrivialProofEnumVariant
    );
    fn children(&self) -> &[Arc<dyn Provable>] {
        &self.provable_children[..]
    }
    fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()> {
        // Input and output are the same
        let input: Vec<ArrayRef> = self
            .aggr_expr
            .iter()
            .map(|expr| expr.array_output())
            .into_iter()
            .collect::<ProofResult<Vec<ArrayRef>>>()?;
        let input_table = Table::try_from(&input)?;
        let output_table = Table::try_from(&self.output()?)?;

        let c_in: Vec<Commitment> = match self.mode() {
            AggregateMode::Partial => self
                .aggr_expr
                .iter()
                .map(|field| -> ProofResult<Commitment> {
                    let proof: Arc<DataFusionProof> = (*field).get_proof()?;
                    match &*proof {
                        AggregateExprProofEnumVariant(expr_proof) => {
                            expr_proof.get_output_commitments()
                        }
                        _ => Err(ProofError::TypeError),
                    }
                })
                .into_iter()
                .collect::<ProofResult<Vec<Commitment>>>()?,
            _ => input_table.commit(),
        };

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
                if *self.mode() == AggregateMode::Partial {
                    let c_expr: Vec<Commitment> = self
                        .aggr_expr
                        .iter()
                        .map(|field| -> ProofResult<Commitment> {
                            let proof: Arc<DataFusionProof> = (*field).get_proof()?;
                            match &*proof {
                                AggregateExprProofEnumVariant(expr_proof) => {
                                    expr_proof.get_output_commitments()
                                }
                                _ => Err(ProofError::TypeError),
                            }
                        })
                        .into_iter()
                        .collect::<ProofResult<Vec<Commitment>>>()?;
                    p.verify(transcript, (c_expr,))
                } else {
                    Ok(())
                }
            }
            _ => Err(ProofError::TypeError),
        }
    }
}

impl ExecutionPlan for AggregateExecWrapper {
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
        let input = children[0].clone();
        let raw_input = unwrap_exec_plan_if_wrapped(&input).into_datafusion_result()?;
        let raw = AggregateExec::try_new(
            *self.mode(),
            self.raw.group_expr().clone(),
            self.raw.aggr_expr().to_vec(),
            raw_input,
            self.raw.input_schema(),
        )?;
        Ok(Arc::new(
            AggregateExecWrapper::try_new_from_raw(&raw).into_datafusion_result()?,
        ))
    }
}

impl_debug_for_provable!(AggregateExecWrapper);
