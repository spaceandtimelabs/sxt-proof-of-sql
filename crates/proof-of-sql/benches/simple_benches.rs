//! Benchmarking using simple timings.
//! To run, execute the following command:
//! ```bash
//! cargo bench -p proof-of-sql --bench simple_benches
//! benches/simple_graph.r
//! ```
#![allow(missing_docs)]
use blitzar::compute::{init_backend_with_config, BackendConfig};
use bumpalo::Bump;
use proof_of_sql::{
    proof_primitive::dory::{
        DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup, ProverSetup,
        PublicParameters, VerifierSetup,
    },
    sql::{postprocessing::apply_postprocessing_steps, proof::VerifiableQueryResult},
};
use std::time::Instant;

mod scaffold;
use scaffold::{querys::QUERIES, BenchmarkAccessor};

const SIZES: &[usize] = &[
    1,
    2,
    5,
    10,
    20,
    50,
    100,
    200,
    500,
    1_000,
    2_000,
    5_000,
    10_000,
    20_000,
    50_000,
    100_000,
    200_000,
    500_000,
    1_000_000,
    2_000_000,
    5_000_000,
    10_000_000,
    20_000_000,
    50_000_000,
    100_000_000,
];
const ITERATIONS: usize = 3;
const MAX_SIZE: usize = 10_000;

fn main() {
    println!("title,query_num,size,operation,time,iteration");
    init_backend_with_config(BackendConfig {
        num_precomputed_generators: 16,
    });
    let mut rng = ark_std::test_rng();
    for &size in SIZES.iter().filter(|s| **s <= MAX_SIZE) {
        let nu = ((size - 1).ilog2() as usize) / 2 + 1;
        let pp = PublicParameters::rand(nu, &mut rng);
        let ps = ProverSetup::from(&pp);
        let prover_setup = DoryProverPublicSetup::new(&ps, nu);
        let vs = VerifierSetup::from(&pp);
        let verifier_setup = DoryVerifierPublicSetup::new(&vs, nu);
        for (query_num, (title, query, columns)) in QUERIES.iter().enumerate() {
            let query_num = query_num + 1;
            let mut accessor = BenchmarkAccessor::default();
            let alloc = Bump::new();
            accessor.insert_table(
                "bench.table".parse().unwrap(),
                &scaffold::generate_random_columns(&alloc, &mut rng, columns, size),
                &prover_setup,
            );
            for it in 0..=ITERATIONS {
                let query_expr = proof_of_sql::sql::parse::QueryExpr::try_new(
                    query.parse().unwrap(),
                    "bench".parse().unwrap(),
                    &accessor,
                )
                .unwrap();
                let now = Instant::now();
                let result = VerifiableQueryResult::<DoryEvaluationProof>::new(
                    query_expr.proof_expr(),
                    &accessor,
                    &prover_setup,
                );
                let prover_time = now.elapsed();
                if it != 0 {
                    println!(
                        "{title},Query #{query_num},{size},Generate Proof,{},{it}",
                        prover_time.as_secs_f64()
                    );
                }
                let now = Instant::now();
                let data = result
                    .verify(query_expr.proof_expr(), &accessor, &verifier_setup)
                    .unwrap();
                let verifier_time = now.elapsed();
                if it != 0 {
                    println!(
                        "{title},Query #{query_num},{size},Verify Proof,{},{it}",
                        verifier_time.as_secs_f64()
                    );
                }
                let _table = apply_postprocessing_steps(data.table, query_expr.postprocessing());
            }
        }
    }
}
