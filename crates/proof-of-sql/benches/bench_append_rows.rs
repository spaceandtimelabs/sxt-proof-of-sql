//! # Running the Benchmark
//!
//! To run the benchmark with the necessary feature flags enabled, use the following command:
//!
//! ```bash
//! cargo bench --features "test" --bench bench_append_rows
//! ```
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use proof_of_sql::{
    base::{
        commitment::TableCommitment,
        database::{owned_table_utility::generate_random_owned_table, OwnedTable},
    },
    proof_primitive::dory::{
        test_rng, DoryCommitment, DoryProverPublicSetup, DoryScalar, ProverSetup, PublicParameters,
    },
};

// append 10 rows to 10 cols in 1 table in 11.382 ms
// append 10 rows to 10 cols * 100 tables = 1.1382 seconds
fn bench_append_rows_10x10(c: &mut Criterion) {
    let public_parameters = PublicParameters::rand(10, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    c.bench_function("append_rows_to_table_commitment", |b| {
        let initial_columns: OwnedTable<DoryScalar> = generate_random_owned_table(10, 10);

        let table_commitment = TableCommitment::<DoryCommitment>::try_from_columns_with_offset(
            initial_columns.inner_table(),
            0,
            &dory_prover_setup,
        )
        .unwrap();

        let append_columns: OwnedTable<DoryScalar> = initial_columns;

        b.iter(|| {
            let mut local_commitment = table_commitment.clone();
            local_commitment
                .try_append_rows(
                    black_box(append_columns.inner_table()),
                    &black_box(dory_prover_setup),
                )
                .unwrap();
        });
    });
}

// append 1000 rows to 10 cols in 1 table in 652ms
fn bench_append_rows_10x1000(c: &mut Criterion) {
    let public_parameters = PublicParameters::rand(10, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    c.bench_function("append_rows_to_table_commitment", |b| {
        let initial_columns: OwnedTable<DoryScalar> = generate_random_owned_table(10, 1000);

        let table_commitment = TableCommitment::<DoryCommitment>::try_from_columns_with_offset(
            initial_columns.inner_table(),
            0,
            &dory_prover_setup,
        )
        .unwrap();

        let append_columns: OwnedTable<DoryScalar> = initial_columns;

        b.iter(|| {
            let mut local_commitment = table_commitment.clone();
            local_commitment
                .try_append_rows(
                    black_box(append_columns.inner_table()),
                    &black_box(dory_prover_setup),
                )
                .unwrap();
        });
    });
}

criterion_group!(benches, bench_append_rows_10x10, bench_append_rows_10x1000);
criterion_main!(benches);
