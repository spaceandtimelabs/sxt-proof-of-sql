use crate::{
    base::{
        datafusion::{
            impl_physical_expr_for_provable,
            DataFusionProof::{self, PhysicalExprProof as PhysicalExprProofEnumVariant},
            PhysicalExprProof, Provable, ProvablePhysicalExpr,
        },
        proof::{
            GeneralColumn, IntoDataFusionResult, IntoProofResult, PipProve, PipVerify, ProofError,
            ProofResult, Transcript,
        },
    },
    datafusion_integration::wrappers::wrap_physical_expr,
    pip::{
        addition::AdditionProof, equality::EqualityProof, inequality::InequalityProof, or::OrProof,
        subtraction::SubtractionProof,
    },
};
use datafusion::{
    arrow::{
        array::ArrayRef,
        datatypes::{DataType, Schema},
        record_batch::RecordBatch,
    },
    logical_expr::Operator,
    physical_expr::{expressions::BinaryExpr, PhysicalExpr},
    physical_plan::ColumnarValue,
};
use std::{
    any::Any,
    fmt::{Debug, Display, Formatter},
    sync::{Arc, RwLock},
};

/// Wrapper around datafusion [BinaryExpr]s that provides proving and verification
pub struct BinaryExprWrapper {
    // storing the args as an array instead of separate fields makes it possible to slice them
    args: [Arc<dyn ProvablePhysicalExpr>; 2],
    args_as_provable: [Arc<dyn Provable>; 2],
    raw: BinaryExpr,
    proof: RwLock<Option<Arc<DataFusionProof>>>,
    output: RwLock<Option<ColumnarValue>>,
    num_rows: RwLock<Option<usize>>,
}

impl BinaryExprWrapper {
    pub fn try_new(raw: &BinaryExpr) -> ProofResult<Self> {
        // wrap left argument for construction
        let raw_left = raw.left();
        let (wrapped_left, _, wrapped_left_as_provable) = wrap_physical_expr(raw_left)?;

        // wrap right argument for construction
        let raw_right = raw.right();
        let (wrapped_right, _, wrapped_right_as_provable) = wrap_physical_expr(raw_right)?;

        Ok(BinaryExprWrapper {
            args: [wrapped_left, wrapped_right],
            args_as_provable: [wrapped_left_as_provable, wrapped_right_as_provable],
            raw: BinaryExpr::new(raw_left.clone(), *raw.op(), raw_right.clone()),
            // These three fields are initially None, but are populated later in the
            // proving/evaluation process.
            proof: RwLock::new(None),
            output: RwLock::new(None),

            num_rows: RwLock::new(None),
        })
    }
}

impl ProvablePhysicalExpr for BinaryExprWrapper {
    fn try_raw(&self) -> ProofResult<Arc<dyn PhysicalExpr>> {
        Ok(Arc::new(BinaryExpr::new(
            self.raw.left().clone(),
            *self.raw.op(),
            self.raw.right().clone(),
        )))
    }

    fn set_num_rows(&self, num_rows: usize) -> ProofResult<()> {
        *self.num_rows.write().into_proof_result()? = Some(num_rows);
        self.args[0].set_num_rows(num_rows)?;
        self.args[1].set_num_rows(num_rows)?;
        Ok(())
    }

    fn array_output(&self) -> ProofResult<ArrayRef> {
        let num_rows =
            (*self.num_rows.read().into_proof_result()?).ok_or(ProofError::UnexecutedError)?;
        (*self.output.read().into_proof_result()?)
            .clone()
            // If the output doesn't exist yet, the expression hasn't been evaluated by datafusion.
            .ok_or(ProofError::UnevaluatedError)
            .map(|c| c.into_array(num_rows))
    }
}

impl Provable for BinaryExprWrapper {
    fn children(&self) -> &[Arc<dyn Provable>] {
        self.args_as_provable.as_slice()
    }

    fn get_proof(&self) -> ProofResult<Arc<DataFusionProof>> {
        (*self.proof.read().into_proof_result()?)
            .clone()
            .ok_or(ProofError::NoProofError)
    }

    fn set_proof(&self, proof: &Arc<DataFusionProof>) -> ProofResult<()> {
        // The proof isn't created by the verifier, so this method sets it on the verifier end
        // We avoid just cloning the proof, since we need to verify that the proof's type is
        // suitable for this wrapper.
        // This match expression performs that verification
        *self.proof.write().into_proof_result()? =
            Some(Arc::new(PhysicalExprProofEnumVariant(match &**proof {
                PhysicalExprProofEnumVariant(PhysicalExprProof::EqualityProof(p)) => {
                    PhysicalExprProof::EqualityProof(p.clone())
                }
                PhysicalExprProofEnumVariant(PhysicalExprProof::InequalityProof(p)) => {
                    PhysicalExprProof::InequalityProof(p.clone())
                }
                PhysicalExprProofEnumVariant(PhysicalExprProof::OrProof(p)) => {
                    PhysicalExprProof::OrProof(p.clone())
                }
                PhysicalExprProofEnumVariant(PhysicalExprProof::AdditionProof(p)) => {
                    PhysicalExprProof::AdditionProof(p.clone())
                }
                PhysicalExprProofEnumVariant(PhysicalExprProof::SubtractionProof(p)) => {
                    PhysicalExprProof::SubtractionProof(p.clone())
                }
                _ => return Err(ProofError::TypeError),
            })));

        Ok(())
    }

    fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()> {
        // Proofs are only meaningful after execution and evaluation because
        // it relies on the returned ArrayRef
        let left_array = self.args[0].array_output()?;
        let left_col = GeneralColumn::try_from(&left_array)?;
        let right_array = self.args[1].array_output()?;
        let right_col = GeneralColumn::try_from(&right_array)?;

        let output_col = self
            .output
            .read()
            .into_proof_result()?
            .clone()
            .ok_or(ProofError::UnevaluatedError)?;
        let output = GeneralColumn::try_from(&output_col)?;

        // The input commitment can be obtained from the output commitments of the child proofs.
        // It's important to get the input commitment this way rather than calculating the
        // commitment from the ArrayRef.
        // The `log_max` values of the commitments should be incremented during arithmetic
        // operations for security purposes, and calculating a new commitment will simply ignore
        // this incrementation.
        let c_in = match (&*self.args[0].get_proof()?, &*self.args[1].get_proof()?) {
            (
                DataFusionProof::PhysicalExprProof(left_p),
                DataFusionProof::PhysicalExprProof(right_p),
            ) => (
                left_p.get_output_commitments()?,
                right_p.get_output_commitments()?,
            ),
            _ => return Err(ProofError::TypeError),
        };

        // Typing violations for mismatched or unsupported `GeneralColumn` variants will be
        // different for each proof.
        // So, these errors are detected within each proof's "general" `PipProve` implementation,
        // and all we have to do here is provide the proofs with general columns.
        let input = (left_col, right_col);
        let proof = match self.raw.op() {
            Operator::Eq => PhysicalExprProof::EqualityProof(EqualityProof::prove(
                transcript, input, output, c_in,
            )),
            Operator::NotEq => PhysicalExprProof::InequalityProof(InequalityProof::prove(
                transcript, input, output, c_in,
            )),
            Operator::Or => {
                PhysicalExprProof::OrProof(OrProof::prove(transcript, input, output, c_in))
            }
            Operator::Plus => PhysicalExprProof::AdditionProof(AdditionProof::prove(
                transcript, input, output, c_in,
            )),
            Operator::Minus => PhysicalExprProof::SubtractionProof(SubtractionProof::prove(
                transcript, input, output, c_in,
            )),
            _ => return Err(ProofError::UnimplementedError),
        };

        *self.proof.write().into_proof_result()? =
            Some(Arc::new(DataFusionProof::PhysicalExprProof(proof)));
        Ok(())
    }

    fn run_verify(&self, transcript: &mut Transcript) -> ProofResult<()> {
        // All proofs that this wrapper covers have the same input commitment type:
        // c_in: (Commitment, Commitment)
        // This allows us to create the input commitment before matching against the proof variants
        let c_in = match (&*self.args[0].get_proof()?, &*self.args[1].get_proof()?) {
            (
                DataFusionProof::PhysicalExprProof(left_p),
                DataFusionProof::PhysicalExprProof(right_p),
            ) => (
                left_p.get_output_commitments()?,
                right_p.get_output_commitments()?,
            ),
            _ => return Err(ProofError::TypeError),
        };

        let proof = self.get_proof()?;

        // Verify on the destructured proof variants
        match &*proof {
            DataFusionProof::PhysicalExprProof(p) => match p {
                PhysicalExprProof::EqualityProof(p) => p.verify(transcript, c_in),
                PhysicalExprProof::InequalityProof(p) => p.verify(transcript, c_in),
                PhysicalExprProof::OrProof(p) => p.verify(transcript, c_in),
                PhysicalExprProof::AdditionProof(p) => p.verify(transcript, c_in),
                PhysicalExprProof::SubtractionProof(p) => p.verify(transcript, c_in),
                _ => Err(ProofError::TypeError),
            },
            _ => Err(ProofError::TypeError),
        }
    }
}

impl PhysicalExpr for BinaryExprWrapper {
    impl_physical_expr_for_provable!();

    fn evaluate(&self, batch: &RecordBatch) -> datafusion::error::Result<ColumnarValue> {
        // TODO: This essentially evaluates the args twice. Is there any way to change datafusion so that
        // we only do it once?
        self.args[0].evaluate(batch)?;
        self.args[1].evaluate(batch)?;

        // Evaluate the expression and mutate self.output
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

impl Display for BinaryExprWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.raw, f)
    }
}

impl Debug for BinaryExprWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BinaryExprWrapper")
            .field("args", &self.args)
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
            array::{BooleanArray, PrimitiveArray},
            datatypes::{DataType, Field, Int64Type, Schema},
            record_batch::RecordBatch,
        },
        physical_expr::expressions::Column as ColumnExpr,
    };

    #[test]
    fn test_binary_wrapper_eq() {
        // Setup
        let array0 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, -1, 5, -5, 0, 10,
        ]));
        let array1 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, 2, 3, -5, -7, 10,
        ]));
        let expected = Arc::new(BooleanArray::from(vec![
            true, true, false, false, true, false, true,
        ]));
        let schema = Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Int64, false),
        ]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![array0, array1]).unwrap();

        let left_col = (ColumnExpr::new_with_schema("a", &schema)).unwrap();
        let right_col = (ColumnExpr::new_with_schema("b", &schema)).unwrap();
        let raw = BinaryExpr::new(Arc::new(left_col), Operator::Eq, Arc::new(right_col));

        // Prover
        let prover_expr = BinaryExprWrapper::try_new(&raw).unwrap();

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
        assert_eq!(proof.len(), 3);

        // Verifier
        let verifier_expr = BinaryExprWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_binary_wrapper_eq");

        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }

    #[test]
    fn test_binary_wrapper_not_eq() {
        // Setup
        let array0 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, -1, 5, -5, 0, 10,
        ]));
        let array1 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, 2, 3, -5, -7, 10,
        ]));
        let expected = Arc::new(BooleanArray::from(vec![
            false, false, true, true, false, true, false,
        ]));
        let schema = Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Int64, false),
        ]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![array0, array1]).unwrap();

        let left_col = (ColumnExpr::new_with_schema("a", &schema)).unwrap();
        let right_col = (ColumnExpr::new_with_schema("b", &schema)).unwrap();
        let raw = BinaryExpr::new(Arc::new(left_col), Operator::NotEq, Arc::new(right_col));

        // Prover
        let prover_expr = BinaryExprWrapper::try_new(&raw).unwrap();

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
        assert_eq!(proof.len(), 3);

        // Verifier
        let verifier_expr = BinaryExprWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_binary_wrapper_eq");

        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }

    #[test]
    fn test_binary_wrapper_or() {
        // Setup
        let array0 = Arc::new(BooleanArray::from(vec![
            true, true, false, false, true, true, false,
        ]));
        let array1 = Arc::new(BooleanArray::from(vec![
            true, false, true, false, true, false, true,
        ]));
        let expected = Arc::new(BooleanArray::from(vec![
            true, true, true, false, true, true, true,
        ]));
        let schema = Schema::new(vec![
            Field::new("a", DataType::Boolean, false),
            Field::new("b", DataType::Boolean, false),
        ]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![array0, array1]).unwrap();

        let left_col = (ColumnExpr::new_with_schema("a", &schema)).unwrap();
        let right_col = (ColumnExpr::new_with_schema("b", &schema)).unwrap();
        let raw = BinaryExpr::new(Arc::new(left_col), Operator::Or, Arc::new(right_col));

        // Prover
        let prover_expr = BinaryExprWrapper::try_new(&raw).unwrap();

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
        assert_eq!(proof.len(), 3);

        // Verifier
        let verifier_expr = BinaryExprWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_binary_wrapper_eq");

        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }

    #[test]
    fn test_binary_wrapper_add() {
        // Setup
        let array0 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, -1, 5, -5, 0, 10,
        ]));
        let array1 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, 2, 3, -5, -7, 10,
        ]));
        let expected = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 2, 1, 8, -10, -7, 20,
        ]));
        let schema = Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Int64, false),
        ]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![array0, array1]).unwrap();

        let left_col = (ColumnExpr::new_with_schema("a", &schema)).unwrap();
        let right_col = (ColumnExpr::new_with_schema("b", &schema)).unwrap();
        let raw = BinaryExpr::new(Arc::new(left_col), Operator::Plus, Arc::new(right_col));

        // Prover
        let prover_expr = BinaryExprWrapper::try_new(&raw).unwrap();

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
        assert_eq!(proof.len(), 3);

        // Verifier
        let verifier_expr = BinaryExprWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_binary_wrapper_eq");

        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }

    #[test]
    fn test_binary_wrapper_sub() {
        // Setup
        let array0 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, -1, 5, -5, 0, 10,
        ]));
        let array1 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, 2, 3, -5, -7, 10,
        ]));
        let expected = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 0, -3, 2, 0, 7, 0,
        ]));
        let schema = Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Int64, false),
        ]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![array0, array1]).unwrap();

        let left_col = (ColumnExpr::new_with_schema("a", &schema)).unwrap();
        let right_col = (ColumnExpr::new_with_schema("b", &schema)).unwrap();
        let raw = BinaryExpr::new(Arc::new(left_col), Operator::Minus, Arc::new(right_col));

        // Prover
        let prover_expr = BinaryExprWrapper::try_new(&raw).unwrap();

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
        assert_eq!(proof.len(), 3);

        // Verifier
        let verifier_expr = BinaryExprWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_binary_wrapper_eq");

        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }

    #[test]
    fn test_binary_wrapper_log_max_persists() {
        // Setup
        let array = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, -1, 5, -5, 0, 10,
        ]));
        let expected = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 66, -66, 330, -330, 0, 660,
        ]));
        let schema = Schema::new(vec![Field::new("a", DataType::Int64, false)]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![array]).unwrap();

        let col = (ColumnExpr::new_with_schema("a", &schema)).unwrap();

        let mut raw = BinaryExpr::new(Arc::new(col.clone()), Operator::Plus, Arc::new(col.clone()));
        for _ in 0..64 {
            raw = BinaryExpr::new(Arc::new(col.clone()), Operator::Plus, Arc::new(raw));
        }

        // Prover
        let prover_expr = BinaryExprWrapper::try_new(&raw).unwrap();

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
        assert_eq!(proof.len(), 131);

        // Check that the output log_max starts from the initial log_max of i64s (63)
        // and is incremented by the proofs 65 times (128)
        match proof.first().unwrap().as_ref() {
            DataFusionProof::PhysicalExprProof(p) => {
                assert_eq!(p.get_output_commitments().unwrap().log_max, Some(63));
            }
            _ => panic!(),
        }
        match proof.last().unwrap().as_ref() {
            DataFusionProof::PhysicalExprProof(p) => {
                assert_eq!(p.get_output_commitments().unwrap().log_max, Some(128));
            }
            _ => panic!(),
        }

        // Verifier
        let verifier_expr = BinaryExprWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_binary_wrapper_eq");

        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }

    #[test]
    fn test_binary_wrapper_log_max_reduction() {
        // Setup
        let array = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, 1, -1, 5, -5, 0, 10,
        ]));
        let expected = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values([
            0, -65, 65, -325, 325, 0, -650,
        ]));
        let schema = Schema::new(vec![Field::new("a", DataType::Int64, false)]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![array]).unwrap();

        let col = (ColumnExpr::new_with_schema("a", &schema)).unwrap();

        let mut raw = BinaryExpr::new(
            Arc::new(col.clone()),
            Operator::Minus,
            Arc::new(col.clone()),
        );
        for _ in 0..65 {
            raw = BinaryExpr::new(Arc::new(raw), Operator::Minus, Arc::new(col.clone()));
        }

        // Prover
        let prover_expr = BinaryExprWrapper::try_new(&raw).unwrap();

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
        assert_eq!(proof.len(), 133);

        // Check that the output log_max starts from the initial log_max of i64s (63)
        // and is incremented by the proofs 66 times and then reduced (129 -> 128)
        match proof.first().unwrap().as_ref() {
            DataFusionProof::PhysicalExprProof(p) => {
                assert_eq!(p.get_output_commitments().unwrap().log_max, Some(63));
            }
            _ => panic!(),
        }
        match proof.last().unwrap().as_ref() {
            DataFusionProof::PhysicalExprProof(p) => {
                assert_eq!(p.get_output_commitments().unwrap().log_max, Some(128));
            }
            _ => panic!(),
        }

        // Check that the final proof performed log max reduction
        match proof.last().unwrap().as_ref() {
            DataFusionProof::PhysicalExprProof(PhysicalExprProof::SubtractionProof(p)) => {
                assert!(p.log_max_reduction_proof.is_some())
            }
            _ => panic!(),
        }

        // Verifier
        let verifier_expr = BinaryExprWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_binary_wrapper_eq");

        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }
}
