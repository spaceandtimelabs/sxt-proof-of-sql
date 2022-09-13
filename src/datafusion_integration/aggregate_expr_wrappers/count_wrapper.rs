use crate::{
    base::{
        datafusion::{
            impl_aggregate_expr_for_provable, impl_debug_for_provable, impl_provable,
            AggregateExprProof::CountProof as CountProofEnumVariant,
            DataFusionProof::{
                self, AggregateExprProof as AggregateExprProofEnumVariant,
                PhysicalExprProof as PhysicalExprProofEnumVariant,
            },
            Provable, ProvableAggregateExpr, ProvablePhysicalExpr,
        },
        proof::{
            GeneralColumn, IntoProofResult, PipProve, PipVerify, ProofError, ProofResult,
            Transcript,
        },
    },
    datafusion_integration::wrappers::wrap_physical_expr,
    pip::aggregate_expr::CountProof,
};
use datafusion::{
    arrow::{array::ArrayRef, datatypes::Field, record_batch::RecordBatch},
    physical_expr::AggregateExpr,
    physical_plan::{expressions::Count, Accumulator, ColumnarValue, PhysicalExpr, RowAccumulator},
};
use std::sync::RwLock;
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    slice,
    sync::Arc,
};
pub struct CountWrapper {
    raw: Count,
    expr: Arc<dyn ProvablePhysicalExpr>,
    expr_as_physical_expr: Arc<dyn PhysicalExpr>,
    expr_as_provable: Arc<dyn Provable>,
    proof: RwLock<Option<Arc<DataFusionProof>>>,
    output: RwLock<Option<ArrayRef>>,
}

impl CountWrapper {
    pub fn try_raw_spec(&self) -> ProofResult<Count> {
        Ok(Count::new(
            self.raw.expressions()[0].clone(),
            self.raw.name(),
            self.raw.field().into_proof_result()?.data_type().clone(),
        ))
    }
    pub fn try_new(raw: &Count) -> ProofResult<Self> {
        let raw_expr = raw.expressions()[0].clone();
        let name = raw.name();
        let data_type = raw.field().into_proof_result()?.data_type().clone();
        let (wrapped_expr, wrapped_expr_as_physical_expr, wrapped_expr_as_provable) =
            wrap_physical_expr(&raw_expr)?;
        Ok(CountWrapper {
            expr: wrapped_expr.clone(),
            expr_as_physical_expr: wrapped_expr_as_physical_expr.clone(),
            expr_as_provable: wrapped_expr_as_provable.clone(),
            raw: Count::new(raw_expr, name, data_type),
            proof: RwLock::new(None),
            output: RwLock::new(None),
        })
    }
}

impl ProvableAggregateExpr for CountWrapper {
    fn try_raw(&self) -> ProofResult<Arc<dyn AggregateExpr>> {
        Ok(Arc::new(self.try_raw_spec()?))
    }
    fn set_output(&self, output: &ColumnarValue) -> ProofResult<()> {
        *self.output.write().into_proof_result()? = Some(output.clone().into_array(1));
        Ok(())
    }
    fn evaluate_and_set_num_rows_for_physicals(&self, input: &RecordBatch) -> ProofResult<()> {
        self.expr.evaluate(input)?;
        self.expr.set_num_rows(input.num_rows())?;
        Ok(())
    }
    fn array_output(&self) -> ProofResult<ArrayRef> {
        (*self.output.read().into_proof_result()?)
            .clone()
            .ok_or(ProofError::UnexecutedError)
    }
}

impl Provable for CountWrapper {
    impl_provable!(
        CountProof,
        AggregateExprProofEnumVariant,
        CountProofEnumVariant
    );
    fn children(&self) -> &[Arc<dyn Provable>] {
        slice::from_ref(&self.expr_as_provable)
    }
    fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()> {
        // Proofs are only meaningful after execution and evaluation because
        // it relies on the returned ArrayRef
        let input = self.expr.array_output()?;
        let output = self.array_output()?;
        let input_col = GeneralColumn::try_from(&input)?;
        let output_col = GeneralColumn::try_from(&output)?;

        // The input commitment can be obtained from the output commitments of the child proof.
        // It's important to get the input commitment this way rather than calculating the
        // commitment from the ArrayRef.
        // The `log_max` values of the commitments should be incremented during arithmetic
        // operations for security purposes, and calculating a new commitment will simply ignore
        // this incrementation.
        let c_in = match &*self.expr.get_proof()? {
            DataFusionProof::PhysicalExprProof(p) => p.get_output_commitments()?,
            _ => return Err(ProofError::TypeError),
        };

        let proof = CountProof::prove(transcript, (input_col,), output_col, (c_in,));
        *self.proof.write().into_proof_result()? = Some(Arc::new(AggregateExprProofEnumVariant(
            CountProofEnumVariant(proof),
        )));
        Ok(())
    }
    fn run_verify(&self, transcript: &mut Transcript) -> ProofResult<()> {
        let proof = self.get_proof()?;
        match &*proof {
            AggregateExprProofEnumVariant(CountProofEnumVariant(p)) => {
                let expr_proof: &DataFusionProof = &*self.expr.get_proof()?;
                match expr_proof {
                    PhysicalExprProofEnumVariant(expr_p) => {
                        let c_in = expr_p.get_output_commitments()?;
                        p.verify(transcript, (c_in,))
                    }
                    _ => Err(ProofError::TypeError),
                }
            }
            _ => Err(ProofError::TypeError),
        }
    }
}

impl AggregateExpr for CountWrapper {
    impl_aggregate_expr_for_provable!();

    fn expressions(&self) -> Vec<Arc<dyn PhysicalExpr>> {
        vec![self.expr_as_physical_expr.clone()]
    }
}

impl_debug_for_provable!(CountWrapper);
