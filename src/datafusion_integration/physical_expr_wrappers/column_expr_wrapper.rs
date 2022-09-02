use crate::{
    base::{
        datafusion::{
            impl_debug_display_for_phys_expr_wrapper, impl_physical_expr_for_provable,
            impl_provable,
            DataFusionProof::{self, PhysicalExprProof as PhysicalExprProofEnumVariant},
            PhysicalExprProof::ColumnProof as ColumnProofEnumVariant,
            Provable, ProvablePhysicalExpr,
        },
        proof::{
            GeneralColumn, IntoDataFusionResult, IntoProofResult, PipProve, PipVerify, ProofError,
            ProofResult, Transcript,
        },
    },
    pip::physical_expr::ColumnProof,
};
use datafusion::{
    arrow::{
        array::ArrayRef,
        datatypes::{DataType, Schema},
        record_batch::RecordBatch,
    },
    physical_expr::{expressions::Column as ColumnExpr, PhysicalExpr},
    physical_plan::ColumnarValue,
};
use std::sync::RwLock;
use std::{
    any::Any,
    fmt::{Debug, Display, Formatter},
    sync::Arc,
};

pub struct ColumnWrapper {
    raw: ColumnExpr,
    proof: RwLock<Option<Arc<DataFusionProof>>>,
    output: RwLock<Option<ColumnarValue>>,
}

impl ColumnWrapper {
    pub fn try_new(raw: &ColumnExpr) -> ProofResult<Self> {
        Ok(ColumnWrapper {
            raw: ColumnExpr::new(raw.name(), raw.index()),
            proof: RwLock::new(None),
            output: RwLock::new(None),
        })
    }
}

impl ProvablePhysicalExpr for ColumnWrapper {
    fn try_raw(&self) -> ProofResult<Arc<dyn PhysicalExpr>> {
        Ok(Arc::new(ColumnExpr::new(self.raw.name(), self.raw.index())))
    }
    fn set_num_rows(&self, _: usize) -> ProofResult<()> {
        // num_rows do not need to be set for ColumnExpr for it always returns
        // an ArrayRef when evaluated
        Ok(())
    }
    fn array_output(&self) -> ProofResult<ArrayRef> {
        // We use 1 here because it doesn't matter
        self.output
            .read()
            .into_proof_result()?
            .clone()
            .ok_or(ProofError::UnevaluatedError)
            .map(|c| c.into_array(1))
    }
}

impl Provable for ColumnWrapper {
    impl_provable!(
        ColumnProof,
        PhysicalExprProofEnumVariant,
        ColumnProofEnumVariant
    );

    // Column does not have children by definition
    fn children(&self) -> &[Arc<dyn Provable>] {
        &[]
    }
    fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()> {
        // Proofs are only meaningful after evaluation because
        // it relies on the returned ColumnarValue
        let output = self.array_output()?;
        let col: GeneralColumn = GeneralColumn::try_from(&output)?;
        let proof = ColumnProof::prove(transcript, (), col, ());
        *self.proof.write().into_proof_result()? = Some(Arc::new(PhysicalExprProofEnumVariant(
            ColumnProofEnumVariant(proof),
        )));
        Ok(())
    }
    fn run_verify(&self, transcript: &mut Transcript) -> ProofResult<()> {
        let proof = self.get_proof()?;
        match &*proof {
            PhysicalExprProofEnumVariant(ColumnProofEnumVariant(p)) => p.verify(transcript, ()),
            _ => Err(ProofError::TypeError),
        }
    }
}

impl PhysicalExpr for ColumnWrapper {
    impl_physical_expr_for_provable!();

    fn evaluate(&self, batch: &RecordBatch) -> datafusion::common::Result<ColumnarValue> {
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

impl_debug_display_for_phys_expr_wrapper!(ColumnWrapper);

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::arrow::{
        array::PrimitiveArray,
        datatypes::{DataType, Field, Int64Type, Schema},
        record_batch::RecordBatch,
    };

    #[test]
    fn test_column_wrapper() {
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

        let raw = (ColumnExpr::new_with_schema("b", &schema)).unwrap();

        // Prover
        let prover_expr = ColumnWrapper::try_new(&raw).unwrap();

        // Evaluate and check output
        let _res = prover_expr.evaluate(&batch).unwrap();
        let res_array = prover_expr.array_output().unwrap().clone();
        assert_eq!(*res_array, *array1);

        // Produce the proof
        let mut transcript = Transcript::new(b"test_column_wrapper");
        prover_expr
            .run_create_proof_with_children(&mut transcript)
            .unwrap();
        let proof = prover_expr.get_proof_with_children().unwrap();
        assert_eq!(proof.len(), 1);

        // Verifier
        let verifier_expr = ColumnWrapper::try_new(&raw).unwrap();

        // Verify the proof
        println!("{:?}", verifier_expr.set_proof_with_children(&proof));
        let mut transcript = Transcript::new(b"test_column_wrapper");
        assert!(verifier_expr
            .run_verify_with_children(&mut transcript)
            .is_ok());
    }
}
