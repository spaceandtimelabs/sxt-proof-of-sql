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
pub mod queries;
mod random_util;
use random_util::{OptionalRandBound, generate_random_columns};
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
    accessor.insert_table(
        "bench.table".parse().unwrap(),
        &generate_random_columns(alloc, rng, columns, size),
        prover_setup,
    );
    let query = QueryExpr::try_new(query.parse().unwrap(), "bench".into(), accessor).unwrap();
    let result = VerifiableQueryResult::new(query.proof_expr(), accessor, prover_setup);
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
    let (query, result) = scaffold::<CP>(
        query,
        columns,
        size,
        prover_setup,
        &alloc,
        &mut accessor,
        &mut rng,
    );
    result
        .verify(query.proof_expr(), &accessor, verifier_setup)
        .unwrap();
}

#[allow(dead_code, clippy::module_name_repetitions)]
pub fn criterion_scaffold<CP: CommitmentEvaluationProof + Clone>(
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
            b.iter(|| {
                result
                    .clone()
                    .verify(query.proof_expr(), &accessor, verifier_setup)
            });
        });
    }
}
