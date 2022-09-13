use crate::{
    base::{
        datafusion::{
            impl_debug_display_for_phys_expr_wrapper, impl_physical_expr_for_provable,
            impl_provable,
            DataFusionProof::{self, PhysicalExprProof as PhysicalExprProofEnumVariant},
            PhysicalExprProof::NegativeProof as NegativeProofEnumVariant,
            Provable, ProvablePhysicalExpr,
        },
        proof::{
            GeneralColumn, IntoDataFusionResult, IntoProofResult, PipProve, PipVerify, ProofError,
            ProofResult, Transcript,
        },
    },
    datafusion_integration::wrappers::wrap_physical_expr,
    pip::physical_expr::NegativeProof,
};
use datafusion::{
    arrow::{
        array::ArrayRef,
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
    num_rows: RwLock<Option<usize>>,
}

impl NegativeExprWrapper {
    pub fn try_new(raw: &NegativeExpr) -> ProofResult<Self> {
        let raw_arg = raw.arg();
        let (wrapped_arg, _, wrapped_arg_as_provable) = wrap_physical_expr(raw_arg)?;
        Ok(NegativeExprWrapper {
            arg: wrapped_arg.clone(),
            arg_as_provable: wrapped_arg_as_provable.clone(),
            raw: NegativeExpr::new(raw_arg.clone()),
            proof: RwLock::new(None),
            output: RwLock::new(None),
            num_rows: RwLock::new(None),
        })
    }

    /// Get the input expression
    pub fn arg(&self) -> &Arc<dyn ProvablePhysicalExpr> {
        &self.arg
    }
}

impl ProvablePhysicalExpr for NegativeExprWrapper {
    fn try_raw(&self) -> ProofResult<Arc<dyn PhysicalExpr>> {
        Ok(Arc::new(NegativeExpr::new(self.raw.arg().clone())))
    }
    fn set_num_rows(&self, num_rows: usize) -> ProofResult<()> {
        *self.num_rows.write().into_proof_result()? = Some(num_rows);
        self.arg.set_num_rows(num_rows)?;
        Ok(())
    }
    fn array_output(&self) -> ProofResult<ArrayRef> {
        let num_rows =
            (*self.num_rows.read().into_proof_result()?).ok_or(ProofError::UnexecutedError)?;
        (*self.output.read().into_proof_result()?)
            .clone()
            .ok_or(ProofError::UnevaluatedError)
            .map(|c| c.into_array(num_rows))
    }
}

impl Provable for NegativeExprWrapper {
    impl_provable!(
        NegativeProof,
        PhysicalExprProofEnumVariant,
        NegativeProofEnumVariant
    );

    fn children(&self) -> &[Arc<dyn Provable>] {
        slice::from_ref(&self.arg_as_provable)
    }
    fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()> {
        // Proofs are only meaningful after execution and evaluation because
        // it relies on the returned ArrayRef
        let input = self.arg.array_output()?;
        let col = GeneralColumn::try_from(&input)?;

        // The input commitment can be obtained from the output commitments of the child proof.
        // It's important to get the input commitment this way rather than calculating the
        // commitment from the ArrayRef.
        // The `log_max` values of the commitments should be incremented during arithmetic
        // operations for security purposes, and calculating a new commitment will simply ignore
        // this incrementation.
        let c_in = match &*self.arg.get_proof()? {
            DataFusionProof::PhysicalExprProof(p) => p.get_output_commitments()?,
            _ => return Err(ProofError::TypeError),
        };

        let proof = NegativeProof::prove(transcript, (col.clone(),), col, (c_in,));
        *self.proof.write().into_proof_result()? = Some(Arc::new(PhysicalExprProofEnumVariant(
            NegativeProofEnumVariant(proof),
        )));
        Ok(())
    }
    fn run_verify(&self, transcript: &mut Transcript) -> ProofResult<()> {
        let proof = self.get_proof()?;
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
    impl_physical_expr_for_provable!();

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

impl_debug_display_for_phys_expr_wrapper!(NegativeExprWrapper);

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::{
        arrow::{
            array::PrimitiveArray,
            datatypes::{DataType, Field, Int64Type, Schema},
            record_batch::RecordBatch,
        },
        logical_plan::Operator,
        physical_expr::expressions::{BinaryExpr, Column as ColumnExpr},
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
        prover_expr.set_num_rows(7).unwrap();
        let res_array = prover_expr.array_output().unwrap().clone();
        assert_eq!(*res_array, *expected);

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

    #[test]
    fn test_negative_wrapper_log_max_persists() {
        // Setup
        let array = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, -1, 5, -5, 0, 10,
        ]));
        let expected = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, -2, 2, -10, 10, 0, -20,
        ]));
        let schema = Schema::new(vec![Field::new("a", DataType::Int64, false)]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![array]).unwrap();

        let col = (ColumnExpr::new_with_schema("a", &schema)).unwrap();

        let raw = NegativeExpr::new(Arc::new(BinaryExpr::new(
            Arc::new(col.clone()),
            Operator::Plus,
            Arc::new(col.clone()),
        )));

        // Prover
        let prover_expr = NegativeExprWrapper::try_new(&raw).unwrap();

        // Evaluate and check output
        let _res = prover_expr.evaluate(&batch).unwrap();
        prover_expr.set_num_rows(7).unwrap();
        let res_array = prover_expr.array_output().unwrap().clone();
        assert_eq!(*res_array, *expected);

        // Produce the proof
        let mut transcript = Transcript::new(b"test_binary_wrapper_eq");
        prover_expr
            .run_create_proof_with_children(&mut transcript)
            .unwrap();
        let proof = prover_expr.get_proof_with_children().unwrap();
        assert_eq!(proof.len(), 4);

        // Check that the output log_max starts from the initial log_max of i64s (63)
        // and is incremented by the addition proof once (64)
        // and that this change persists through the negative proof
        match proof.first().unwrap().as_ref() {
            DataFusionProof::PhysicalExprProof(p) => {
                assert_eq!(p.get_output_commitments().unwrap().log_max, Some(63));
            }
            _ => panic!(),
        }
        match proof.last().unwrap().as_ref() {
            DataFusionProof::PhysicalExprProof(p) => {
                assert_eq!(p.get_output_commitments().unwrap().log_max, Some(64));
            }
            _ => panic!(),
        }

        // Verifier
        let verifier_expr = NegativeExprWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_binary_wrapper_eq");

        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }
}
