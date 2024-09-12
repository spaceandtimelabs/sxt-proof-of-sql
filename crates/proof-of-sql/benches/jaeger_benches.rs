//! Benchmarking/Tracing using Jaeger.
//! To run, execute the following commands:
//! ```bash
//! docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:latest
//! cargo bench -p proof-of-sql --bench jaeger_benches InnerProductProof
//! cargo bench -p proof-of-sql --bench jaeger_benches Dory --features="test"
//! ```
//! Then, navigate to http://localhost:16686 to view the traces.

use crate::scaffold::generate_random_columns;
use blitzar::{compute::init_backend, proof::InnerProductProof};
use bumpalo::Bump;
#[cfg(feature = "test")]
use proof_of_sql::proof_primitive::dory::{
    DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup, ProverSetup,
    PublicParameters, VerifierSetup,
};
use proof_of_sql::{
    base::{commitment::CommitmentEvaluationProof, database::ColumnType},
    sql::{
        parse::QueryExpr, postprocessing::apply_postprocessing_steps, proof::VerifiableQueryResult,
    },
};
use std::env;

mod scaffold;
use scaffold::{querys::QUERIES, BenchmarkAccessor, OptionalRandBound};

const SIZE: usize = 1_000_000;

#[tracing::instrument(
    level = "debug",
    skip(query, columns, size, prover_setup, verifier_setup)
)]
fn jaeger_scaffold<CP: CommitmentEvaluationProof>(
    title: &str,
    query: &str,
    columns: &[(&str, ColumnType, OptionalRandBound)],
    size: usize,
    prover_setup: &CP::ProverPublicSetup<'_>,
    verifier_setup: &CP::VerifierPublicSetup<'_>,
) {
    let mut accessor = BenchmarkAccessor::default();
    let mut rng = rand::thread_rng();
    let alloc = Bump::new();
    accessor.insert_table(
        "bench.table".parse().unwrap(),
        &generate_random_columns(&alloc, &mut rng, columns, size),
        prover_setup,
    );
    let query =
        QueryExpr::try_new(query.parse().unwrap(), "bench".parse().unwrap(), &accessor).unwrap();
    let result = VerifiableQueryResult::<CP>::new(query.proof_expr(), &accessor, prover_setup);
    let data = result
        .verify(query.proof_expr(), &accessor, verifier_setup)
        .unwrap();
    let _table = apply_postprocessing_steps(data.table, query.postprocessing());
}

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
                for (title, query, columns) in QUERIES.iter() {
                    jaeger_scaffold::<InnerProductProof>(title, query, columns, SIZE, &(), &());
                }
            }
        }
        #[cfg(feature = "test")]
        "Dory" => {
            // Run 3 times to ensure that warm-up of the GPU has occurred.
            let pp =
                PublicParameters::rand(10, &mut proof_of_sql::proof_primitive::dory::test_rng());
            let ps = ProverSetup::from(&pp);
            let prover_setup = DoryProverPublicSetup::new(&ps, 10);
            let vs = VerifierSetup::from(&pp);
            let verifier_setup = DoryVerifierPublicSetup::new(&vs, 10);

            for _ in 0..3 {
                for (title, query, columns) in QUERIES.iter() {
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
