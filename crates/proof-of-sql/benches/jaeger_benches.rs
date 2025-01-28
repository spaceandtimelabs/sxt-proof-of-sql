//! Benchmarking/Tracing using Jaeger.
//! To run, execute the following commands:
//! ```bash
//! docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:1.62.0
//! cargo bench -p proof-of-sql --bench jaeger_benches InnerProductProof
//! cargo bench -p proof-of-sql --bench jaeger_benches Dory
//! cargo bench -p proof-of-sql --bench jaeger_benches DynamicDory
//! cargo bench -p proof-of-sql --bench jaeger_benches HyperKZG
//! ```
//! Then, navigate to <http://localhost:16686> to view the traces.

use ark_std::test_rng;
use blitzar::{compute::init_backend, proof::InnerProductProof};
use nova_snark::{
    provider::hyperkzg::{CommitmentEngine, CommitmentKey, EvaluationEngine},
    traits::{commitment::CommitmentEngineTrait, evaluation::EvaluationEngineTrait},
};
use num_bigint::BigUint;
use proof_of_sql::{
    base::database::{
        owned_table_utility::{owned_table, scalar},
        ColumnRef, ColumnType, OwnedTable, OwnedTableTestAccessor, TestAccessor,
    },
    proof_primitive::{
        dory::{
            DoryEvaluationProof, DoryProverPublicSetup, DoryScalar, DoryVerifierPublicSetup,
            DynamicDoryEvaluationProof, ProverSetup, PublicParameters, VerifierSetup,
        },
        hyperkzg::{HyperKZGCommitmentEvaluationProof, HyperKZGEngine},
    },
    sql::{proof::VerifiableQueryResult, proof_gadgets::range_check_test::RangeCheckTestPlan},
};
mod scaffold;
use crate::scaffold::querys::QUERIES;
use scaffold::jaeger_scaffold;
use std::{env, path::Path};

const SIZE: usize = 1_000_000;

#[allow(clippy::items_after_statements)]
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
                        title, query, columns, SIZE, &&ck, &&vk,
                    );
                }
            }
        }
        "DynDoryRangeCheck" => {
            let blitzar_handle_path = std::env::var("BLITZAR_HANDLE_PATH")
                .expect("Environment variable BLITZAR_HANDLE_PATH not set");
            let public_parameters_path = std::env::var("PUBLIC_PARAMETERS_PATH")
                .expect("Environment variable PUBLIC_PARAMETERS_PATH not set");
            let verifier_setup_path = std::env::var("VERIFIER_SETUP_PATH")
                .expect("Environment variable VERIFIER_SETUP_PATH not set");

            let handle = blitzar::compute::MsmHandle::new_from_file(&blitzar_handle_path);
            let public_parameters =
                PublicParameters::load_from_file(Path::new(&public_parameters_path)).unwrap();

            let prover_setup =
                ProverSetup::from_public_parameters_and_blitzar_handle(&public_parameters, handle);
            let verifier_setup = VerifierSetup::load_from_file(Path::new(&verifier_setup_path))
                .expect("Failed to load VerifierSetup");

            // 2^248 - 1
            let big_uint = BigUint::from(2u8).pow(248) - BigUint::from(1u8);
            let limbs_vec: Vec<u64> = big_uint.to_u64_digits();

            // Convert Vec<u64> to [u64; 4]
            let limbs: [u64; 4] = limbs_vec[..4].try_into().unwrap();

            let upper_bound = DoryScalar::from_bigint(limbs);

            // Generate the test data
            let data: OwnedTable<DoryScalar> = owned_table([scalar(
                "a",
                (0..2u32.pow(20))
                    .map(|i| upper_bound - DoryScalar::from(u64::from(i))) // Count backward from 2^248
                    .collect::<Vec<_>>(),
            )]);

            let t = "sxt.t".parse().unwrap();
            let mut accessor =
                OwnedTableTestAccessor::<DynamicDoryEvaluationProof>::new_empty_with_setup(
                    &prover_setup,
                );

            accessor.add_table("sxt.t".parse().unwrap(), data, 0);

            let ast = RangeCheckTestPlan {
                column: ColumnRef::new(t, "a".into(), ColumnType::Scalar),
            };

            let verifiable_res = VerifiableQueryResult::<DynamicDoryEvaluationProof>::new(
                &ast,
                &accessor,
                &&prover_setup,
            );

            let res = verifiable_res.verify(&ast, &accessor, &&verifier_setup);

            if let Err(e) = res {
                panic!("Verification failed: {e}");
            }
            assert!(res.is_ok());
        }
        _ => panic!("Invalid benchmark type specified."),
    }

    opentelemetry::global::shutdown_tracer_provider();
}
