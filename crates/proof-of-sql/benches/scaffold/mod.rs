use blitzar::compute::init_backend;
use bumpalo::Bump;
use criterion::{AxisScale, Criterion, PlotConfiguration};
use proof_of_sql::{
    base::{commitment::CommitmentEvaluationProof, database::ColumnType},
    sql::{parse::QueryExpr, proof::VerifiableQueryResult},
};
use rand::prelude::Rng;
mod benchmark_accessor;
use benchmark_accessor::BenchmarkAccessor;
pub mod querys;
mod random_util;
use random_util::{generate_random_columns, OptionalRandBound};
use sysinfo::{System, SystemExt};

/// # Panics
///
/// Will panic if:
/// - The table reference cannot be parsed from the string.
/// - The columns generated from `generate_random_columns` lead to a failure in `insert_table`.
/// - The query string cannot be parsed into a `QueryExpr`.
/// - The creation of the `VerifiableQueryResult` fails due to invalid proof expressions.
fn scaffold<'a, CP: CommitmentEvaluationProof>(
    query: &str,
    columns: &[(&str, ColumnType, OptionalRandBound)],
    size: usize,
    prover_setup: &CP::ProverPublicSetup<'_>,
    alloc: &'a Bump,
    accessor: &mut BenchmarkAccessor<'a, CP::Commitment>,
    rng: &mut impl Rng,
) -> (QueryExpr, VerifiableQueryResult<CP>) {
    let mut system = System::new_all();
    system.refresh_all();
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    dbg!("Begin creating scaffold");
    dbg!(total_memory);
    dbg!(used_memory);
    let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
    dbg!(memory_used);

    accessor.insert_table(
        "bench.table".parse().unwrap(),
        &generate_random_columns(alloc, rng, columns, size),
        prover_setup,
    );
    
    system.refresh_all();
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    dbg!("End insert_table");
    dbg!(total_memory);
    dbg!(used_memory);
    let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
    dbg!(memory_used);

    let query =
        QueryExpr::try_new(query.parse().unwrap(), "bench".parse().unwrap(), accessor).unwrap();

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("End QueryExpr::try_new");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);

    let result = VerifiableQueryResult::new(query.proof_expr(), accessor, prover_setup);

    system.refresh_all();
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    dbg!("End VerifiableQueryResult::new");
    dbg!(total_memory);
    dbg!(used_memory);
    let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
    dbg!(memory_used);

    (query, result)
}

#[tracing::instrument(
    level = "debug",
    skip(query, columns, size, prover_setup, verifier_setup)
)]
/// # Panics
///
/// Will panic if:
/// - The call to `scaffold` results in a panic due to invalid inputs.
/// - The `verify` method of `VerifiableQueryResult` fails, indicating an invalid proof.
pub fn jaeger_scaffold<CP: CommitmentEvaluationProof>(
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

    // Print the current CPU memory usage
    let mut system = System::new_all();
    system.refresh_all();
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    dbg!("Begin creating jaeger_scaffold");
    dbg!(total_memory);
    dbg!(used_memory);
    let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
    dbg!(memory_used);


    let (query, result) = scaffold::<CP>(
        query,
        columns,
        size,
        prover_setup,
        &alloc,
        &mut accessor,
        &mut rng,
    );

    system.refresh_all();
    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    dbg!("jaeger_scaffold created");
    dbg!(total_memory);
    dbg!(used_memory);
    let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
    dbg!(memory_used);

    result
        .verify(query.proof_expr(), &accessor, verifier_setup)
        .unwrap();
}

#[allow(dead_code, clippy::module_name_repetitions)]
pub fn criterion_scaffold<CP: CommitmentEvaluationProof>(
    c: &mut Criterion,
    title: &str,
    query: &str,
    columns: &[(&str, ColumnType, OptionalRandBound)],
    sizes: &[usize],
    prover_setup: &CP::ProverPublicSetup<'_>,
    verifier_setup: &CP::VerifierPublicSetup<'_>,
) {
    let mut group = c.benchmark_group(format!("{title} - {query}"));
    group.sample_size(10);
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
    init_backend();
    let mut accessor = BenchmarkAccessor::default();
    let mut rng = rand::thread_rng();
    let alloc = Bump::new();
    for &size in sizes {
        group.throughput(criterion::Throughput::Elements(size as u64));
        let (query, result) = scaffold::<CP>(
            query,
            columns,
            size,
            prover_setup,
            &alloc,
            &mut accessor,
            &mut rng,
        );
        group.bench_function("Generate Proof", |b| {
            b.iter(|| {
                VerifiableQueryResult::<CP>::new(query.proof_expr(), &accessor, prover_setup)
            });
        });
        group.bench_function("Verify Proof", |b| {
            b.iter(|| result.verify(query.proof_expr(), &accessor, verifier_setup));
        });
    }
}
