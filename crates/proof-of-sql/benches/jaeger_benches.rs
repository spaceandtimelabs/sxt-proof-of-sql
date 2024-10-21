//! Benchmarking/Tracing using Jaeger.
//! To run, execute the following commands:
//! ```bash
//! docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:latest
//! cargo bench -p proof-of-sql --bench jaeger_benches InnerProductProof
//! cargo bench -p proof-of-sql --bench jaeger_benches Dory
//! ```
//! Then, navigate to <http://localhost:16686> to view the traces.

use ark_std::test_rng;
use blitzar::{compute::init_backend, proof::InnerProductProof};
use proof_of_sql::proof_primitive::dory::{
    DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup, ProverSetup,
    PublicParameters, VerifierSetup,
};
mod scaffold;
use crate::scaffold::querys::QUERIES;
use scaffold::jaeger_scaffold;
use std::env;

const SIZE: usize = 1_000_000;

#[allow(clippy::items_after_statements)]
fn main() {
    init_backend();
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("benches")
        .install_simple()
        .unwrap();
    let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(opentelemetry)
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
        _ => panic!("Invalid benchmark type specified."),
    }

    opentelemetry::global::shutdown_tracer_provider();
}
