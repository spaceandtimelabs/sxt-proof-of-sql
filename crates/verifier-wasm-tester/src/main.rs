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
        DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup,
        ProverSetup, PublicParameters, VerifierSetup,
    },
    sql::{
        parse::QueryExpr,
        proof::QueryProof,
    },
};
use proof_of_sql::sql::proof::ProofExecutionPlan;
use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    let public_parameters = PublicParameters::rand(4, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);
    let sigma = 3;
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, sigma);
    let _dory_verifier_setup = DoryVerifierPublicSetup::new(&verifier_setup, sigma);

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

    // Save parameters to files
    let query = "SELECT * FROM table WHERE b = 1";
    let schema = "sxt";

    {
        let mut file = File::create("param_query.txt")?;
        file.write_all(query.as_bytes())?;
    }
    {
        let mut file = File::create("param_schema.txt")?;
        file.write_all(schema.as_bytes())?;
    }
    {
        let mut file = File::create("param_sigma.txt")?;
        file.write_all(format!("{sigma}").as_bytes())?;
    }

    // Serialize complex parameters
    let query_commitments_enc = bincode::serialize(&query_commitments).unwrap();
    let proof_enc = bincode::serialize(&proof).unwrap();
    let serialized_result_enc = bincode::serialize(&serialized_result).unwrap();
    let verifier_setup_enc = bincode::serialize(&verifier_setup).unwrap();

    {
        let mut file = File::create("param_query_commitments.bin")?;
        file.write_all(&query_commitments_enc)?;
    }
    {
        let mut file = File::create("param_proof.bin")?;
        file.write_all(&proof_enc)?;
    }
    {
        let mut file = File::create("param_serialized_result.bin")?;
        file.write_all(&serialized_result_enc)?;
    }
    {
        let mut file = File::create("param_verifier_setup.bin")?;
        file.write_all(&verifier_setup_enc)?;
    }

    Ok(())
}
