//! Benchmarking using the `criterion` crate.
//! To run, execute the following command:
//! ```bash
//! cargo bench -p proof-of-sql --bench criterion_benches
//! ```
#![allow(missing_docs)]
use blitzar::{compute::init_backend, proof::InnerProductProof};
use bumpalo::Bump;
use criterion::{criterion_group, criterion_main, AxisScale, Criterion, PlotConfiguration};
use proof_of_sql::{
    base::{commitment::CommitmentEvaluationProof, database::ColumnType},
    sql::proof::VerifiableQueryResult,
};

mod scaffold;
use scaffold::{querys::QUERIES, BenchmarkAccessor, OptionalRandBound};

const SIZES: &[usize] = &[
    1,
    10,
    100,
    1_000,
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

fn criterion_scaffold<CP: CommitmentEvaluationProof>(
    c: &mut Criterion,
    title: &str,
    query: &str,
    columns: &[(&str, ColumnType, OptionalRandBound)],
    sizes: &[usize],
    prover_setup: &CP::ProverPublicSetup<'_>,
    verifier_setup: &CP::VerifierPublicSetup<'_>,
) {
    let mut group = c.benchmark_group(format!("{} - {}", title, query));
    group.sample_size(10);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    init_backend();
    let mut rng = rand::thread_rng();
    let mut accessor = BenchmarkAccessor::default();
    let alloc = Bump::new();
    for &size in sizes {
        group.throughput(criterion::Throughput::Elements(size as u64));
        accessor.insert_table(
            "bench.table".parse().unwrap(),
            &scaffold::generate_random_columns(&alloc, &mut rng, columns, size),
            prover_setup,
        );
        let query = proof_of_sql::sql::parse::QueryExpr::try_new(
            query.parse().unwrap(),
            "bench".parse().unwrap(),
            &accessor,
        )
        .unwrap();
        let result = VerifiableQueryResult::<CP>::new(query.proof_expr(), &accessor, prover_setup);
        group.bench_function("Generate Proof", |b| {
            b.iter(|| VerifiableQueryResult::<CP>::new(query.proof_expr(), &accessor, prover_setup))
        });
        group.bench_function("Verify Proof", |b| {
            b.iter(|| result.verify(query.proof_expr(), &accessor, verifier_setup))
        });
    }
}

fn all_benches(c: &mut Criterion) {
    for (title, query, columns) in QUERIES {
        criterion_scaffold::<InnerProductProof>(c, title, query, columns, SIZES, &(), &());
    }
}

criterion_group!(benches, all_benches);
criterion_main!(benches);
