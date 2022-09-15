use crate::{
    base::{
        datafusion::{
            impl_debug_display_for_phys_expr_wrapper, impl_physical_expr_for_provable,
            impl_provable,
            DataFusionProof::{self, PhysicalExprProof as PhysicalExprProofEnumVariant},
            PhysicalExprProof::LiteralProof as LiteralProofEnumVariant,
            Provable, ProvablePhysicalExpr,
        },
        proof::{
            GeneralColumn, IntoProofResult, PipProve, PipVerify, ProofError, ProofResult,
            Transcript,
        },
    },
    pip::physical_expr::LiteralProof,
};
use datafusion::{
    arrow::{
        array::ArrayRef,
        datatypes::{DataType, Schema},
        record_batch::RecordBatch,
    },
    physical_expr::{expressions::Literal, PhysicalExpr},
    physical_plan::ColumnarValue,
    scalar::ScalarValue,
};
use std::sync::RwLock;
use std::{
    any::Any,
    fmt::{Debug, Display, Formatter},
    sync::Arc,
};

// Literal is a constant so there is no need to store a separate ColumnarValue output
pub struct LiteralWrapper {
    raw: Literal,
    proof: RwLock<Option<Arc<DataFusionProof>>>,
    num_rows: RwLock<Option<usize>>,
}

impl LiteralWrapper {
    pub fn try_new(raw: &Literal) -> ProofResult<Self> {
        Ok(LiteralWrapper {
            raw: Literal::new(raw.value().clone()),
            proof: RwLock::new(None),
            num_rows: RwLock::new(None),
        })
    }

    pub fn value(&self) -> &ScalarValue {
        self.raw.value()
    }
}

impl ProvablePhysicalExpr for LiteralWrapper {
    fn try_raw(&self) -> ProofResult<Arc<dyn PhysicalExpr>> {
        Ok(Arc::new(Literal::new(self.raw.value().clone())))
    }
    fn set_num_rows(&self, num_rows: usize) -> ProofResult<()> {
        *self.num_rows.write().into_proof_result()? = Some(num_rows);
        Ok(())
    }
    fn array_output(&self) -> ProofResult<ArrayRef> {
        let num_rows =
            (*self.num_rows.read().into_proof_result()?).ok_or(ProofError::UnexecutedError)?;
        Ok(self.raw.value().to_array_of_size(num_rows))
    }
}

impl Provable for LiteralWrapper {
    impl_provable!(
        LiteralProof,
        PhysicalExprProofEnumVariant,
        LiteralProofEnumVariant
    );

    // Literal does not have children by definition
    fn children(&self) -> &[Arc<dyn Provable>] {
        &[]
    }
    fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()> {
        // Proofs are only meaningful after evaluation because
        // it relies on the returned ColumnarValue
        let output = self.array_output()?;
        let col: GeneralColumn = GeneralColumn::try_from(&output)?;
        let proof = LiteralProof::prove(transcript, (), col, ());
        *self.proof.write().into_proof_result()? = Some(Arc::new(PhysicalExprProofEnumVariant(
            LiteralProofEnumVariant(proof),
        )));
        Ok(())
    }
    fn run_verify(&self, transcript: &mut Transcript) -> ProofResult<()> {
        let proof = self.get_proof()?;
        match &*proof {
            PhysicalExprProofEnumVariant(LiteralProofEnumVariant(p)) => p.verify(transcript, ()),
            _ => Err(ProofError::TypeError),
        }
    }
}

impl PhysicalExpr for LiteralWrapper {
    impl_physical_expr_for_provable!();

    fn evaluate(&self, batch: &RecordBatch) -> datafusion::common::Result<ColumnarValue> {
        self.raw.evaluate(batch)
    }
}

impl_debug_display_for_phys_expr_wrapper!(LiteralWrapper);

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::arrow::{
        array::PrimitiveArray,
        datatypes::{DataType, Field, Int32Type, Int64Type, Schema},
        record_batch::RecordBatch,
    };

    #[test]
    fn test_literal_wrapper() {
        // Setup
        let array0 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values(
            (0..7_i64).map(|x| x + 1),
        ));
        let array1 = Arc::new(PrimitiveArray::<Int64Type>::from_iter_values(
            (0..7_i64).map(|x| x + 2),
        ));
        let schema = Schema::new(vec![
            Field::new("a", DataType::Int64, false),
            Field::new("b", DataType::Int64, false),
        ]);
        let batch = RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![array0.clone(), array1.clone()],
        )
        .unwrap();

        let raw = Literal::new(ScalarValue::Int32(Some(4)));
        let expected = Arc::new(PrimitiveArray::<Int32Type>::from(vec![4; 7]));

        // Prover
        let prover_expr = LiteralWrapper::try_new(&raw).unwrap();

        // Evaluate and check output
        let _res = prover_expr.evaluate(&batch).unwrap();
        prover_expr.set_num_rows(7).unwrap();
        let res_array = prover_expr.array_output().unwrap().clone();
        assert_eq!(*res_array, *expected);

        // Produce the proof
        let mut transcript = Transcript::new(b"test_literal_wrapper");
        prover_expr
            .run_create_proof_with_children(&mut transcript)
            .unwrap();
        let proof = prover_expr.get_proof_with_children().unwrap();
        assert_eq!(proof.len(), 1);

        // Verifier
        let verifier_expr = LiteralWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_literal_wrapper");
        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }
}
