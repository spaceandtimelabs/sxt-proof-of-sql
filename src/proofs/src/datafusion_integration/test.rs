use crate::base::{
    datafusion::collect,
    proof::{IntoProofResult, ProofResult, Transcript},
};
use std::sync::Arc;

use datafusion::{
    execution::context::TaskContext,
    physical_plan::{displayable, ExecutionPlan},
    prelude::*,
};

use super::wrappers::wrap_exec_plan;

/// Helpful in checking for unimplemented wrappers
#[allow(dead_code)]
fn display_exec_plan(plan: Arc<dyn ExecutionPlan>) {
    let displayable_plan = displayable(plan.as_ref());
    let plan_string = format!("{}", displayable_plan.indent());
    println!("{}", plan_string);
}

/// General csv-sql test function
async fn test_read_csv_and_query(
    table_name: &str,
    file_name: &str,
    query: &str,
) -> ProofResult<()> {
    let path = format!("test_files/csv/{}.csv", file_name);
    //Prover side. Produces proof.
    let config = SessionConfig::new().with_target_partitions(1);
    let ctx = SessionContext::with_config(config);
    ctx.register_csv(table_name, &path[..], CsvReadOptions::new())
        .await
        .into_proof_result()?;

    // Print Exec Plan here
    // display_exec_plan(plan);
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
    let config = SessionConfig::new().with_target_partitions(1);
    let ctx = SessionContext::with_config(config);
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
    //provable.run_verify_with_children(&mut transcript)
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

// Basic projection
test_read_csv_and_query_macro! {"tab1", "one_column", "select a from tab1", test_trivial0}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a from tab2", test_trivial1}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select -b from tab2", test_neg0}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select -a, b from tab2", test_neg1}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select 2 as c from tab2", test_const}

// Binary expressions
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a = b from tab2", test_eq}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a != b from tab2", test_neq}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a + b from tab2", test_add}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a - b from tab2", test_sub}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select a * b from tab2", test_mul}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select (a = b) or (a = -b) from tab2", test_or}

// Aggregation
test_read_csv_and_query_macro! {"tab2", "two_columns", "select count(1) as c from tab2", test_count0}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select count(a) as c from tab2", test_count1}
test_read_csv_and_query_macro! {"tab2", "two_columns", "select count(1) from tab2", test_count2}

// Filter
//test_read_csv_and_query_macro! {"tab1", "one_column", "select a from tab1 where a > 1", test_filter0}
//test_read_csv_and_query_macro! {"tab2", "two_columns", "select a from tab2 where a > 1", test_filter1}
//test_read_csv_and_query_macro! {"tab2", "two_columns", "select a, b from tab2 where a > 1 and b > 1", test_filter2}

// Mixed
// test_read_csv_and_query_macro! {"tab2", "two_columns", "select sum(a) as sa, count(1) as c from tab2 where a > 4", test_count_filter}
