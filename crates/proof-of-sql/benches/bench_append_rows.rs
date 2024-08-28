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
        database::{
            owned_table_utility::{
                bigint, boolean, decimal75, int, int128, owned_table, scalar, smallint,
                timestamptz, varchar,
            },
            OwnedTable,
        },
        scalar::Scalar,
    },
    proof_primitive::dory::{
        test_rng, DoryCommitment, DoryProverPublicSetup, DoryScalar, ProverSetup, PublicParameters,
    },
};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
use rand::Rng;
use std::ops::Deref;

/// Bench dory performance when appending rows to a table. This includes the computation of
/// commitments. Chose the number of columns to randomly generate across supported PoSQL
/// data types, and choose the number of rows to append at a time.
///
/// ```text
/// Most recent benches on 13th Gen Intel® Core™ i9-13900H × 20 with 32gb of RAM:
/// append 10 rows to 10 cols in 1 table = 11.382 ms
/// append 10 rows to 10 cols in 100 tables = 1.1382 seconds
/// append 1000 rows to 10 cols in 1 table = 652ms
/// ```
fn bench_append_rows(c: &mut Criterion, cols: usize, rows: usize) {
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

/// Generates a random OwnedTable with a specified number of columns
pub fn generate_random_owned_table<S: Scalar>(
    num_columns: usize,
    num_rows: usize,
) -> OwnedTable<S> {
    let mut rng = rand::thread_rng();
    let column_types = [
        "bigint",
        "boolean",
        "int128",
        "scalar",
        "varchar",
        "decimal75",
        "smallint",
        "int",
        "timestamptz",
    ];

    let mut columns = Vec::new();

    for _ in 0..num_columns {
        let column_type = column_types[rng.gen_range(0..column_types.len())];
        let identifier = format!("column_{}", rng.gen::<u32>());

        match column_type {
            "bigint" => columns.push(bigint(identifier.deref(), vec![rng.gen::<i64>(); num_rows])),
            "boolean" => columns.push(boolean(
                identifier.deref(),
                generate_random_boolean_vector(num_rows),
            )),
            "int128" => columns.push(int128(
                identifier.deref(),
                vec![rng.gen::<i128>(); num_rows],
            )),
            "scalar" => columns.push(scalar(
                identifier.deref(),
                vec![generate_random_u64_array(); num_rows],
            )),
            "varchar" => columns.push(varchar(identifier.deref(), gen_rnd_str(num_rows))),
            "decimal75" => columns.push(decimal75(
                identifier.deref(),
                12,
                2,
                vec![generate_random_u64_array(); num_rows],
            )),
            "smallint" => columns.push(smallint(
                identifier.deref(),
                vec![rng.gen::<i16>(); num_rows],
            )),
            "int" => columns.push(int(identifier.deref(), vec![rng.gen::<i32>(); num_rows])),
            "timestamptz" => columns.push(timestamptz(
                identifier.deref(),
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::Utc,
                vec![rng.gen::<i64>(); num_rows],
            )),
            _ => unreachable!(),
        }
    }

    owned_table(columns)
}

/// Generates a random vec of varchar
fn gen_rnd_str(array_size: usize) -> Vec<String> {
    let mut rng = rand::thread_rng();

    // Create a vector to hold the owned Strings
    let mut string_vec: Vec<String> = Vec::with_capacity(array_size);

    for _ in 0..array_size {
        // Generate a random string of a fixed length (e.g., 10 characters)
        let random_string: String = (0..10)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect();

        string_vec.push(random_string);
    }

    string_vec
}

/// Generates a random [u64; 4]
fn generate_random_u64_array() -> [u64; 4] {
    let mut rng = rand::thread_rng();
    [rng.gen(), rng.gen(), rng.gen(), rng.gen()]
}

/// Generates a random vec of true/false
fn generate_random_boolean_vector(size: usize) -> Vec<bool> {
    let mut rng = rand::thread_rng();
    (0..size).map(|_| rng.gen()).collect()
}

fn bench_append_rows_10x10(c: &mut Criterion) {
    bench_append_rows(c, 10, 10);
}

fn bench_append_rows_10x1000(c: &mut Criterion) {
    bench_append_rows(c, 10, 1000);
}
criterion_group!(benches, bench_append_rows_10x10, bench_append_rows_10x1000);
criterion_main!(benches);
