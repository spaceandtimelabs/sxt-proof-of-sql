use crate::base::{
    datafusion::collect,
    proof::{IntoProofResult, ProofResult, Transcript},
};
use std::sync::Arc;

use datafusion::{execution::context::TaskContext, physical_plan::ExecutionPlan, prelude::*};

use super::wrappers::wrap_exec_plan;

/// General csv-sql test function
async fn test_read_csv_and_query(
    table_name: &str,
    file_name: &str,
    query: &str,
) -> ProofResult<()> {
    let path = format!("test_files/csv/{}.csv", file_name);
    //Prover side. Produces proof.
    let ctx = SessionContext::new();
    ctx.register_csv(table_name, &path[..], CsvReadOptions::new())
        .await
        .into_proof_result()?;
    let plan: Arc<dyn ExecutionPlan> = ctx
        .sql(query)
        .await
        .into_proof_result()?
        .create_physical_plan()
        .await
        .into_proof_result()?;
    let (physical, _, _) = wrap_exec_plan(&plan).unwrap();
    let _query_result = collect(
        &physical,
        Arc::new(TaskContext::from(&ctx.state.read().clone())),
    )
    .await?;

    let mut transcript = Transcript::new(b"test_integration");
    physical
        .run_create_proof_with_children(&mut transcript)
        .unwrap();
    let proof = physical.get_proof_with_children().unwrap();

    //End prover side. Produces proof.

    //Verifier side. Consumes proof.
    let ctx = SessionContext::new();
    ctx.register_csv(table_name, &path[..], CsvReadOptions::new())
        .await
        .into_proof_result()?;
    let plan = ctx
        .sql(query)
        .await
        .into_proof_result()?
        .create_physical_plan()
        .await
        .into_proof_result()?;
    let (_, _, provable) = wrap_exec_plan(&plan).unwrap();

    println!("{:?}", provable.set_proof_with_children(&proof));
    let mut transcript = Transcript::new(b"test_integration");
    assert!(provable.run_verify_with_children(&mut transcript).is_ok());

    //End verifier side. Consumes proof.
    Ok(())
}

macro_rules! test_read_csv_and_query_macro {
    ($table_name:expr, $file_name:expr, $query:expr, $test_func_name:ident) => {
        #[tokio::test]
        async fn $test_func_name() -> ProofResult<()> {
            test_read_csv_and_query($table_name, $file_name, $query).await
        }
    };
}

// Put your tests here
test_read_csv_and_query_macro! {"tab1", "one_column", "select a from tab1", test_trivial0}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a from tab2", test_trivial1}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select -b from tab2", test_neg0}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select -a, b from tab2", test_neg1}

// binary expressions
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a = b from tab2", test_eq}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a != b from tab2", test_neq}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a + b from tab2", test_add}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a - b from tab2", test_sub}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select (a = b) or (a = -b) from tab2", test_or}
