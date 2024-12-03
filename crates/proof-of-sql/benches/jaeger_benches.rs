//! Benchmarking/Tracing using Jaeger.
//! To run, execute the following commands:
//! ```bash
//! docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:latest
//! cargo bench -p proof-of-sql --bench jaeger_benches InnerProductProof
//! cargo bench -p proof-of-sql --bench jaeger_benches Dory
//! cargo bench -p proof-of-sql --bench jaeger_benches DynamicDory
//! ```
//! Then, navigate to <http://localhost:16686> to view the traces.

use ark_std::test_rng;
use blitzar::{compute::init_backend, proof::InnerProductProof};
use proof_of_sql::proof_primitive::dory::{
    DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup,
    DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
};
mod scaffold;
use crate::scaffold::querys::QUERIES;
use scaffold::jaeger_scaffold;
use std::{env, path::Path};

const SIZE: usize = 1_000_000;

#[allow(clippy::items_after_statements)]
fn main() {
    //init_backend();
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
        // "InnerProductProof" => {
        //     // Run 3 times to ensure that warm-up of the GPU has occurred.
        //     for _ in 0..3 {
        //         for (title, query, columns) in QUERIES {
        //             jaeger_scaffold::<InnerProductProof>(title, query, columns, SIZE, &(), &());
        //         }
        //     }
        // }
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
            /*
            // Run 3 times to ensure that warm-up of the GPU has occurred.
            let public_parameters = PublicParameters::test_rand(11, &mut test_rng());
            let prover_setup = ProverSetup::from(&public_parameters);
            let verifier_setup = VerifierSetup::from(&public_parameters);
            */

            /*
            // Load with blitzar
            let blitzar_handle_path = "/home/jacob.trombetta/sxt-proof-of-sql/data/blitzar_handle.bin";
            let public_parameters_path = "/home/jacob.trombetta/sxt-proof-of-sql/data/public_parameters_nu_15.bin";
            let verifier_setup_path = "/home/jacob.trombetta/sxt-proof-of-sql/data/verifier_setup_nu_15.bin";

            let handle = blitzar::compute::MsmHandle::new_from_file(&blitzar_handle_path);
            let params =
                PublicParameters::load_from_file(Path::new(&public_parameters_path)).unwrap();

            let prover_setup =
                ProverSetup::from_public_parameters_and_blitzar_handle(&params, handle);
            let verifier_setup = VerifierSetup::load_from_file(Path::new(&verifier_setup_path))
                .expect("Failed to load VerifierSetup");            
            */

            // Load without blitzar
            let public_parameters_path = "/home/jacob.trombetta/sxt-proof-of-sql/data/public_parameters_nu_15.bin";
            let verifier_setup_path = "/home/jacob.trombetta/sxt-proof-of-sql/data/verifier_setup_nu_15.bin";

            let params =
                PublicParameters::load_from_file(Path::new(&public_parameters_path)).unwrap();

            let prover_setup = ProverSetup::from(&params);
            let verifier_setup = VerifierSetup::load_from_file(Path::new(&verifier_setup_path))
                .expect("Failed to load VerifierSetup");

            
            let sizes = [1<<24];

            for size in sizes {
            //for _ in 0..3 {
                dbg!("Starting {} data size on CPU", size);
                for (title, query, columns) in QUERIES {
                    jaeger_scaffold::<DynamicDoryEvaluationProof>(
                        title,
                        query,
                        columns,
                        size,
                        &&prover_setup,
                        &&verifier_setup,
                    );
                }
                dbg!("Successfully ran benchmarks on {} data points", size);
            }
        }
        _ => panic!("Invalid benchmark type specified."),
    }

    opentelemetry::global::shutdown_tracer_provider();
}
