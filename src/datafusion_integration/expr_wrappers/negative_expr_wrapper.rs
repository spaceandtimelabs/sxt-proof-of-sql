use crate::{
    base::{
        datafusion::{
            DataFusionProof::{self, PhysicalExprProof as PhysicalExprProofEnumVariant},
            PhysicalExprProof::NegativeProof as NegativeProofEnumVariant,
            Provable, ProvablePhysicalExpr,
        },
        proof::{
            Column, Commit, Commitment, IntoDataFusionResult, IntoProofResult, PipProve, PipVerify,
            ProofError, ProofResult, Transcript,
        },
    },
    datafusion_integration::wrappers::wrap_physical_expr,
    pip::expressions::NegativeProof,
};
use datafusion::{
    arrow::{
        datatypes::{DataType, Schema},
        record_batch::RecordBatch,
    },
    physical_expr::{expressions::NegativeExpr, PhysicalExpr},
    physical_plan::ColumnarValue,
};
use std::{
    any::Any,
    fmt::{Debug, Display, Formatter},
    slice,
    sync::{Arc, RwLock},
};

pub struct NegativeExprWrapper {
    arg: Arc<dyn ProvablePhysicalExpr>,
    arg_as_provable: Arc<dyn Provable>,
    raw: NegativeExpr,
    proof: RwLock<Option<Arc<DataFusionProof>>>,
    output: RwLock<Option<ColumnarValue>>,
}

impl NegativeExprWrapper {
    pub fn try_new(raw: &NegativeExpr) -> ProofResult<Self> {
        let raw_arg = raw.arg();
        let (wrapped_arg, wrapped_arg_as_provable) = wrap_physical_expr(raw_arg)?;
        Ok(NegativeExprWrapper {
            arg: wrapped_arg.clone(),
            arg_as_provable: wrapped_arg_as_provable.clone(),
            raw: NegativeExpr::new(raw_arg.clone()),
            proof: RwLock::new(None),
            output: RwLock::new(None),
        })
    }

    /// Get the input expression
    pub fn arg(&self) -> &Arc<dyn ProvablePhysicalExpr> {
        &self.arg
    }
}

impl ProvablePhysicalExpr for NegativeExprWrapper {
    fn output(&self) -> ProofResult<ColumnarValue> {
        (*self.output.read().into_proof_result()?)
            .clone()
            .ok_or(ProofError::UnevaluatedError)
    }
}

impl Provable for NegativeExprWrapper {
    // Column does not have children by definition
    fn children(&self) -> &[Arc<dyn Provable>] {
        slice::from_ref(&self.arg_as_provable)
    }
    fn get_proof(&self) -> ProofResult<Arc<DataFusionProof>> {
        (*self.proof.read().into_proof_result()?)
            .clone()
            .ok_or(ProofError::NoProofError)
    }
    fn set_proof(&self, proof: &Arc<DataFusionProof>) -> ProofResult<()> {
        let typed_proof: &NegativeProof = match &**proof {
            PhysicalExprProofEnumVariant(NegativeProofEnumVariant(p)) => p,
            _ => return Err(ProofError::TypeError),
        };
        *self.proof.write().into_proof_result()? = Some(Arc::new(PhysicalExprProofEnumVariant(
            NegativeProofEnumVariant(typed_proof.clone()),
        )));
        Ok(())
    }
    fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()> {
        // Proofs are only meaningful after evaluation because
        // it relies on the returned ColumnarValue
        let input = self.arg.output()?;
        match input.data_type() {
            DataType::Int64 => {
                let col: Column<i64> = Column::try_from(&input)?;
                let c_in: Commitment = col.commit();
                let proof = NegativeProof::prove(transcript, (col.clone(),), col, (c_in,));
                *self.proof.write().into_proof_result()? = Some(Arc::new(
                    PhysicalExprProofEnumVariant(NegativeProofEnumVariant(proof)),
                ));
                Ok(())
            }
            _ => Err(ProofError::TypeError),
        }
    }
    fn run_verify(&self, transcript: &mut Transcript) -> ProofResult<()> {
        let proof = (*self.proof.read().into_proof_result()?)
            .clone()
            .ok_or(ProofError::NoProofError)?;
        match &*proof {
            PhysicalExprProofEnumVariant(NegativeProofEnumVariant(p)) => {
                let arg_proof: &DataFusionProof = &*self.arg.get_proof()?;
                match arg_proof {
                    PhysicalExprProofEnumVariant(arg_p) => {
                        let c_in = arg_p.get_output_commitments()?;
                        p.verify(transcript, (c_in,))
                    }
                    _ => Err(ProofError::TypeError),
                }
            }
            _ => Err(ProofError::TypeError),
        }
    }
}

impl PhysicalExpr for NegativeExprWrapper {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn data_type(&self, input_schema: &Schema) -> datafusion::common::Result<DataType> {
        self.raw.data_type(input_schema)
    }
    fn nullable(&self, input_schema: &Schema) -> datafusion::common::Result<bool> {
        self.raw.nullable(input_schema)
    }
    fn evaluate(&self, batch: &RecordBatch) -> datafusion::common::Result<ColumnarValue> {
        // TODO: This essentially evaluates the arg twice. Is there any way to change datafusion so that
        // we only do it once?
        self.arg.evaluate(batch)?;
        let result = self.raw.evaluate(batch);
        match result {
            Ok(r) => {
                *self.output.write().into_datafusion_result()? = Some(r.clone());
                Ok(r)
            }
            Err(e) => {
                *self.output.write().into_datafusion_result()? = None;
                Err(e)
            }
        }
    }
}

impl Display for NegativeExprWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.raw, f)
    }
}

impl Debug for NegativeExprWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NegativeExprWrapper")
            .field("arg", &self.arg)
            .field("raw", &self.raw)
            .field(
                "output",
                &(*self.output.read().map_err(|_| std::fmt::Error)?)
                    .clone()
                    .map(|cv| cv.into_array(1)),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        arrow::{
            array::PrimitiveArray,
            datatypes::{DataType, Field, Int64Type, Schema},
            record_batch::RecordBatch,
        },
        physical_expr::expressions::Column as ColumnExpr,
    };

    #[test]
    fn test_negative_wrapper() {
        // Setup
        let array0 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values(
            (0..7_i64).map(|x| x - 1),
        ));
        let array1 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values(
            (0..7_i64).map(|x| x - 2),
        ));
        let expected = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values(
            (0..7_i64).map(|x| -x + 1),
        ));
        let schema = Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Int64, false),
        ]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![array0, array1]).unwrap();

        let col = (ColumnExpr::new_with_schema("a", &schema)).unwrap();
        let raw = NegativeExpr::new(Arc::new(col));

        // Prover
        let prover_expr = NegativeExprWrapper::try_new(&raw).unwrap();

        // Evaluate and check output
        let _res = prover_expr.evaluate(&batch).unwrap();
        match prover_expr.output().unwrap().clone() {
            ColumnarValue::Array(a) => assert_eq!(*a, *expected),
            _ => panic!("Output is unexpectedly a ScalarValue!"),
        }

        // Produce the proof
        let mut transcript = Transcript::new(b"test_negative_wrapper");
        prover_expr
            .run_create_proof_with_children(&mut transcript)
            .unwrap();
        let proof = prover_expr.get_proof_with_children().unwrap();
        assert_eq!(proof.len(), 2);

        // Verifier
        let verifier_expr = NegativeExprWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_negative_wrapper");
        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }
}
