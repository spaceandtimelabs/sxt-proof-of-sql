//! Benchmarking/Tracing using Jaeger.
//! To run, execute the following commands:
//! ```bash
//! docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one
//! cargo bench -p proof-of-sql --bench jaeger_benches InnerProductProof
//! cargo bench -p proof-of-sql --bench jaeger_benches Dory
//! cargo bench -p proof-of-sql --bench jaeger_benches DynamicDory
//! cargo bench -p proof-of-sql --bench jaeger_benches HyperKZG --features="hyperkzg_proof"
//! ```
//! Then, navigate to <http://localhost:16686> to view the traces.

use ark_std::test_rng;
use blitzar::{compute::init_backend, proof::InnerProductProof};
use nova_snark::{
    provider::hyperkzg::{CommitmentEngine, CommitmentKey, EvaluationEngine},
    traits::{commitment::CommitmentEngineTrait, evaluation::EvaluationEngineTrait},
};
use proof_of_sql::proof_primitive::{
    dory::{
        DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup,
        DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
    },
    hyperkzg::{
        nova_commitment_key_to_hyperkzg_public_setup, HyperKZGCommitmentEvaluationProof,
        HyperKZGEngine,
    },
};
mod jaeger_setup;
use jaeger_setup::{setup_jaeger_tracing, stop_jaeger_tracing};
mod scaffold;
use crate::scaffold::queries::{get_query, QueryEntry, QUERIES};
use scaffold::jaeger_scaffold;
use std::env;

fn bench_inner_product_proof(iterations: usize, queries: &[QueryEntry], table_size: usize) {
    for (title, query, columns) in queries {
        for _ in 0..iterations {
            jaeger_scaffold::<InnerProductProof>(title, query, columns, table_size, &(), &());
        }
    }
}

fn bench_dory(iterations: usize, queries: &[QueryEntry], table_size: usize) {
    let pp = PublicParameters::test_rand(10, &mut test_rng());
    let ps = ProverSetup::from(&pp);
    let prover_setup = DoryProverPublicSetup::new(&ps, 10);
    let vs = VerifierSetup::from(&pp);
    let verifier_setup = DoryVerifierPublicSetup::new(&vs, 10);

    for (title, query, columns) in queries {
        for _ in 0..iterations {
            jaeger_scaffold::<DoryEvaluationProof>(
                title,
                query,
                columns,
                table_size,
                &prover_setup,
                &verifier_setup,
            );
        }
    }
}

fn bench_dynamic_dory(iterations: usize, queries: &[QueryEntry], table_size: usize) {
    let public_parameters = PublicParameters::test_rand(11, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let verifier_setup = VerifierSetup::from(&public_parameters);

    for (title, query, columns) in queries {
        for _ in 0..iterations {
            jaeger_scaffold::<DynamicDoryEvaluationProof>(
                title,
                query,
                columns,
                table_size,
                &&prover_setup,
                &&verifier_setup,
            );
        }
    }
}

fn bench_hyperkzg(iterations: usize, queries: &[QueryEntry], table_size: usize) {
    let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"bench", table_size);
    let (_, vk) = EvaluationEngine::setup(&ck);
    for (title, query, columns) in queries {
        for _ in 0..iterations {
            jaeger_scaffold::<HyperKZGCommitmentEvaluationProof>(
                title,
                query,
                columns,
                table_size,
                &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
                &&vk,
            );
        }
    }
}

fn main() {
    init_backend();

    setup_jaeger_tracing().expect("Failed to setup Jaeger tracing.");

    // Check for command-line arguments to select the benchmark type.
    let args: Vec<String> = env::args().collect();

    let benchmark_type = args.get(1).expect("Please specify the benchmark type");

    let num_iterations: usize = args
        .get(2)
        .expect("Please specify the number of iterations")
        .parse()
        .expect("Failed to parse the number of iterations as a number");

    let table_size: usize = args
        .get(3)
        .expect("Please specify the table size")
        .parse()
        .expect("Failed to parse the table size as a number");

    let query = args.get(4).expect("Please specify the query type");

    let queries = if query == "all" {
        QUERIES
    } else {
        let query = get_query(query).expect("Invalid query type specified.");
        &[query]
    };

    match benchmark_type.as_str() {
        "InnerProductProof" => {
            bench_inner_product_proof(num_iterations, queries, table_size);
        }
        "Dory" => {
            bench_dory(num_iterations, queries, table_size);
        }
        "DynamicDory" => {
            bench_dynamic_dory(num_iterations, queries, table_size);
        }
        "HyperKZG" => {
            bench_hyperkzg(num_iterations, queries, table_size);
        }
        _ => panic!("Invalid benchmark type specified."),
    }

    stop_jaeger_tracing();
}
