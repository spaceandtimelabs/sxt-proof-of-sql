use std::io::Write;
use std::sync::Arc;

use proofs::{
    base::{
        datafusion::collect,
        proof::{IntoProofResult, ProofResult, Transcript},
    },
    datafusion_integration::wrappers::wrap_exec_plan,
};

use datafusion::{execution::context::TaskContext, physical_plan::ExecutionPlan, prelude::*};

fn prompt(name: &str) -> String {
    let mut line = String::new();
    print!("{}", name);
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    return line.trim().to_string();
}

#[allow(clippy::await_holding_lock)]
async fn test_query(query: &str) -> ProofResult<()> {
    let table_name = "tab";
    let file_name = "two_columns";
    //Prover side. Produces proof.
    println!("Proving...");
    let config = SessionConfig::new().with_target_partitions(1);
    let ctx = SessionContext::with_config(config);
    let path = format!("../../test_files/csv/{}.csv", file_name);
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
    println!("Verifying...");
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

    provable.set_proof_with_children(&proof)?;
    let mut transcript = Transcript::new(b"test_integration");
    //provable.run_verify_with_children(&mut transcript)
    assert!(provable.run_verify_with_children(&mut transcript).is_ok());
    //End verifier side. Consumes proof.
    Ok(())
}

#[tokio::main]
async fn main() -> ProofResult<()> {
    println!("Proofs-CLI");
    println!("Please enter your query:");
    loop {
        let input = prompt("> ");
        if input == "exit" {
            break;
        } else {
            let result = test_query(input.as_str()).await;
            match result {
                Ok(()) => println!("Success!"),
                Err(err) => println!("Error found: {:?}", err),
            }
        };
    }
    Ok(())
}
