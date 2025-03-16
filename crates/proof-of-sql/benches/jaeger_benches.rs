//! Benchmarking/Tracing using Jaeger.
//! To run, execute the following commands:
//! ```bash
//! docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:1.62.0
//! cargo bench -p proof-of-sql --bench jaeger_benches InnerProductProof
//! cargo bench -p proof-of-sql --bench jaeger_benches Dory
//! cargo bench -p proof-of-sql --bench jaeger_benches DynamicDory
//! cargo bench -p proof-of-sql --bench jaeger_benches HyperKZG --features="hyperkzg"
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
mod scaffold;
use crate::scaffold::queries::QUERIES;
use scaffold::jaeger_scaffold;
use std::env;

const SIZE: usize = 1_000_000;

#[expect(clippy::items_after_statements)]
fn main() {
    init_backend();

    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("benches")
        .install_simple()
        .unwrap();

    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("DEBUG"));

    tracing_subscriber::registry()
        .with(opentelemetry)
        .with(filter)
        .try_init()
        .unwrap();

    // Check for command-line arguments to select the benchmark type.
    let args: Vec<String> = env::args().collect();
    let benchmark_type = args
        .get(1)
        .expect("Please specify the benchmark type: InnerProductProof or Dory");

    match benchmark_type.as_str() {
        "InnerProductProof" => {
            // Run 3 times to ensure that warm-up of the GPU has occurred.
            for _ in 0..3 {
                for (title, query, columns) in QUERIES {
                    jaeger_scaffold::<InnerProductProof>(title, query, columns, SIZE, &(), &());
                }
            }
        }
        "Dory" => {
            // Run 3 times to ensure that warm-up of the GPU has occurred.
            let pp = PublicParameters::test_rand(10, &mut test_rng());
            let ps = ProverSetup::from(&pp);
            let prover_setup = DoryProverPublicSetup::new(&ps, 10);
            let vs = VerifierSetup::from(&pp);
            let verifier_setup = DoryVerifierPublicSetup::new(&vs, 10);

            for _ in 0..3 {
                for (title, query, columns) in QUERIES {
                    jaeger_scaffold::<DoryEvaluationProof>(
                        title,
                        query,
                        columns,
                        SIZE,
                        &prover_setup,
                        &verifier_setup,
                    );
                }
            }
        }
        "DynamicDory" => {
            // Run 3 times to ensure that warm-up of the GPU has occurred.
            let public_parameters = PublicParameters::test_rand(11, &mut test_rng());
            let prover_setup = ProverSetup::from(&public_parameters);
            let verifier_setup = VerifierSetup::from(&public_parameters);

            for _ in 0..3 {
                for (title, query, columns) in QUERIES {
                    jaeger_scaffold::<DynamicDoryEvaluationProof>(
                        title,
                        query,
                        columns,
                        SIZE,
                        &&prover_setup,
                        &&verifier_setup,
                    );
                }
            }
        }
        "HyperKZG" => {
            let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"bench", SIZE);
            let (_, vk) = EvaluationEngine::setup(&ck);
            for _ in 0..3 {
                for (title, query, columns) in QUERIES {
                    jaeger_scaffold::<HyperKZGCommitmentEvaluationProof>(
                        title,
                        query,
                        columns,
                        SIZE,
                        &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
                        &&vk,
                    );
                }
            }
        }
        _ => panic!("Invalid benchmark type specified."),
    }

    opentelemetry::global::shutdown_tracer_provider();
}
