use crate::base::proof::{IntoProofResult, ProofResult, Transcript};
use std::sync::Arc;

use datafusion::{
    arrow::{
        array::{ArrayRef, Int64Array, StringArray},
        compute::kernels::{aggregate::min_boolean, comparison::eq_dyn},
        record_batch::RecordBatch,
    },
    execution::context::TaskContext,
    physical_plan::{collect, projection::ProjectionExec, ColumnarValue},
    prelude::*,
};

use super::casting::batch_column_to_columnar_value;
use super::wrappers::wrap_physical_expr;

#[tokio::test]
async fn test_integration() -> ProofResult<()> {
    //Prover side. Produces proof.

    let ctx = SessionContext::new();
    ctx.register_csv(
        "example",
        "test_files/integration_test.csv",
        CsvReadOptions::new(),
    )
    .await
    .into_proof_result()?;
    let plan = ctx
        .sql("SELECT -a FROM example")
        .await
        .into_proof_result()?
        .create_physical_plan()
        .await
        .into_proof_result()?;
    let proj: &ProjectionExec = plan.as_any().downcast_ref::<ProjectionExec>().unwrap();
    let (physical, _) = wrap_physical_expr(&proj.expr()[0].0).unwrap();
    let input = collect(
        proj.input().clone(),
        Arc::new(TaskContext::from(&ctx.state.read().clone())),
    )
    .await
    .into_proof_result()?;
    physical.evaluate(&input[0]).into_proof_result()?;

    let mut transcript = Transcript::new(b"test_integration");
    physical
        .run_create_proof_with_children(&mut transcript)
        .unwrap();
    let proof = physical.get_proof_with_children().unwrap();
    assert_eq!(proof.len(), 2);

    //End prover side. Produces proof.

    //Verifier side. Consumes proof.
    let ctx = SessionContext::new();
    ctx.register_csv(
        "example",
        "test_files/integration_test.csv",
        CsvReadOptions::new(),
    )
    .await
    .into_proof_result()?;
    let plan = ctx
        .sql("SELECT -a FROM example")
        .await
        .into_proof_result()?
        .create_physical_plan()
        .await
        .into_proof_result()?;
    let proj: &ProjectionExec = plan.as_any().downcast_ref::<ProjectionExec>().unwrap();
    let (_, provable) = wrap_physical_expr(&proj.expr()[0].0).unwrap();

    println!("{:?}", provable.set_proof_with_children(&proof));
    let mut transcript = Transcript::new(b"test_integration");
    assert!(provable.run_verify_with_children(&mut transcript).is_ok());

    //End verifier side. Consumes proof.
    Ok(())
}

#[test]
fn test_batch_column_convert() {
    let arrs: [ArrayRef; 2] = [
        Arc::new(Int64Array::from(vec![0, 1, 2, 3, 4])),
        Arc::new(StringArray::from(vec![
            None,
            Some("test"),
            Some("space"),
            None,
            Some("time!"),
        ])),
    ];
    let batch =
        RecordBatch::try_from_iter(vec![("col0", arrs[0].clone()), ("col1", arrs[1].clone())])
            .unwrap();
    for i in 0..2 {
        let actual = batch_column_to_columnar_value(&batch, i);
        let expected = ColumnarValue::Array(arrs[i].clone());
        // Check array equality
        match (actual, expected) {
            (ColumnarValue::Array(actual_array), ColumnarValue::Array(expected_array)) => {
                let min = min_boolean(&eq_dyn(&*actual_array, &*expected_array).unwrap());
                assert_eq!(Some(true), min);
            }
            _ => panic!("Either the expected ColumnarValue or the actual one is a Scalar!"),
        }
    }
}

#[test]
#[should_panic]
fn test_invalid_batch_column_convert_bad_index() {
    let arr: ArrayRef = Arc::new(Int64Array::from(vec![0, 1, 2, 3, 4]));
    let batch = RecordBatch::try_from_iter(vec![("col", arr.clone())]).unwrap();
    batch_column_to_columnar_value(&batch, 1);
}
