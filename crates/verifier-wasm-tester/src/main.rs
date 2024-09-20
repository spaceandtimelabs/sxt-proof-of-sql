use ark_std::test_rng;
use proof_of_sql::{
    base::{
        commitment::{QueryCommitments, QueryCommitmentsExt},
        database::{
            owned_table_utility::{bigint, owned_table},
            OwnedTableTestAccessor, TestAccessor,
        },
    },
    proof_primitive::dory::{
        DoryCommitment, DoryEvaluationProof, DoryProverPublicSetup, ProverSetup, PublicParameters,
        VerifierSetup,
    },
    sql::{parse::QueryExpr, proof::ProofPlan, proof::ProvableQueryResult, proof::QueryProof},
};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::ExitCode;

struct VerifierInputs {
    query: String,
    schema: String,
    query_commitments: QueryCommitments<DoryCommitment>,
    proof: QueryProof<DoryEvaluationProof>,
    serialized_result: ProvableQueryResult,
    verifier_setup: VerifierSetup,
    sigma: usize,
}

fn generate_verifier_inputs() -> VerifierInputs {
    // Generate verifier parameters with hardcoded values
    let public_parameters = PublicParameters::rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let sigma = 3;
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, sigma);

    let mut accessor =
        OwnedTableTestAccessor::<DoryEvaluationProof>::new_empty_with_setup(dory_prover_setup);
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table([bigint("a", [1, 2, 3]), bigint("b", [1, 0, 1])]),
        0,
    );
    let query = QueryExpr::try_new(
        "SELECT * FROM table WHERE b = 1".parse().unwrap(),
        "sxt".parse().unwrap(),
        &accessor,
    )
    .unwrap();
    let (proof, serialized_result) =
        QueryProof::<DoryEvaluationProof>::new(query.proof_expr(), &accessor, &dory_prover_setup);
    let query_commitments = QueryCommitments::from_accessor_with_max_bounds(
        query.proof_expr().get_column_references(),
        &accessor,
    );

    VerifierInputs {
        query: "SELECT * FROM table WHERE b = 1".into(),
        schema: "sxt".into(),
        query_commitments,
        proof,
        serialized_result,
        verifier_setup,
        sigma,
    }
}

fn create_folder(path: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

fn save_inputs(inputs: VerifierInputs, out_dir: &str) -> std::io::Result<()> {
    // Serialize complex parameters
    let query_commitments_enc = bincode::serialize(&inputs.query_commitments).unwrap();
    let proof_enc = bincode::serialize(&inputs.proof).unwrap();
    let serialized_result_enc = bincode::serialize(&inputs.serialized_result).unwrap();
    let verifier_setup_enc = bincode::serialize(&inputs.verifier_setup).unwrap();

    // Save text parameters to text files
    {
        let mut file = File::create(PathBuf::from(out_dir).join("param_query.txt"))?;
        file.write_all(inputs.query.as_bytes())?;
    }
    {
        let mut file = File::create(PathBuf::from(out_dir).join("param_schema.txt"))?;
        file.write_all(inputs.schema.as_bytes())?;
    }
    {
        let mut file = File::create(PathBuf::from(out_dir).join("param_sigma.txt"))?;
        file.write_all(format!("{}", inputs.sigma).as_bytes())?;
    }

    // Save serialized parameters to binary files
    {
        let mut file = File::create(PathBuf::from(out_dir).join("param_query_commitments.bin"))?;
        file.write_all(&query_commitments_enc)?;
    }
    {
        let mut file = File::create(PathBuf::from(out_dir).join("param_proof.bin"))?;
        file.write_all(&proof_enc)?;
    }
    {
        let mut file = File::create(PathBuf::from(out_dir).join("param_serialized_result.bin"))?;
        file.write_all(&serialized_result_enc)?;
    }
    {
        let mut file = File::create(PathBuf::from(out_dir).join("param_verifier_setup.bin"))?;
        file.write_all(&verifier_setup_enc)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("One argument expected: output folder name");
        return ExitCode::from(1);
    }
    let output_folder = &args[1];

    // Create a folder where the result will be saved
    if let Err(e) = create_folder(output_folder) {
        eprintln!("Error creating the '{}' folder: {}", output_folder, e);
        return ExitCode::from(2);
    }

    // Generate the inputs for the verifier function
    let verifier_inputs = generate_verifier_inputs();

    // Save the inputs to files
    if let Err(e) = save_inputs(verifier_inputs, output_folder) {
        eprintln!("Error writing files: {}", e);
        return ExitCode::from(2);
    }

    ExitCode::from(0)
}
