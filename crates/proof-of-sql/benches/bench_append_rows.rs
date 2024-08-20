//! # Running the Benchmark
//!
//! To run the benchmark with the necessary feature flags enabled, use the following command:
//!
//! ```bash
//! cargo bench --features "test" --bench bench_append_rows
//! ```
#![allow(missing_docs)]
use criterion::{criterion_group, criterion_main, Criterion};
use proof_of_sql::{
    base::{
        commitment::TableCommitment,
        database::{
            owned_table_utility::{bigint, int, owned_table, scalar, smallint, varchar},
            OwnedTable,
        },
    },
    proof_primitive::dory::{
        test_rng, DoryCommitment, DoryProverPublicSetup, DoryScalar, ProverSetup, PublicParameters,
    },
};
use proof_of_sql_parser::Identifier;
use rand::Rng;

const ROW_SIZE: usize = 1000;

// append 10 rows to 10 cols in 1 table in 11.382 ms
// append 10 rows to 10 cols * 100 tables = 1.1382 seconds
// append 1000 rows to 10 cols in 1 table in 196.97 ms
fn bench_append_rows(c: &mut Criterion) {
    let public_parameters = PublicParameters::rand(10, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let dory_prover_setup = DoryProverPublicSetup::new(&prover_setup, 3);
    let mut rng = rand::thread_rng();
    c.bench_function("append_rows_to_table_commitment", |b| {
        let scalar_id: Identifier = "scalar_column".parse().unwrap();
        let varchar_id: Identifier = "varchar_column".parse().unwrap();
        let bigint_id: Identifier = "bigint_column".parse().unwrap();
        let int_id: Identifier = "int_column".parse().unwrap();
        let smallint_id: Identifier = "smallint_column".parse().unwrap();

        let scalar_data = [rng.gen::<u32>(); ROW_SIZE];
        let varchar_data = gen_rnd_str();
        let bigint_data = [rng.gen::<i64>(); ROW_SIZE];
        let int_data = [rng.gen::<i32>(); ROW_SIZE];
        let smallint_data = [rng.gen::<i16>(); ROW_SIZE];

        let initial_columns: OwnedTable<DoryScalar> = owned_table([
            scalar(scalar_id, scalar_data[..2].to_vec()),
            varchar(varchar_id, varchar_data[..2].to_vec()),
            bigint(bigint_id, bigint_data[..2].to_vec()),
            int(int_id, int_data[..2].to_vec()),
            smallint(smallint_id, smallint_data[..2].to_vec()),
            scalar(scalar_id, scalar_data[..2].to_vec()),
            varchar(varchar_id, varchar_data[..2].to_vec()),
            bigint(bigint_id, bigint_data[..2].to_vec()),
            int(int_id, int_data[..2].to_vec()),
            smallint(smallint_id, smallint_data[..2].to_vec()),
        ]);

        let table_commitment = TableCommitment::<DoryCommitment>::try_from_columns_with_offset(
            initial_columns.inner_table(),
            0,
            &dory_prover_setup,
        )
        .unwrap();

        let append_columns: OwnedTable<DoryScalar> = owned_table([
            scalar(scalar_id, scalar_data[2..].to_vec()),
            varchar(varchar_id, varchar_data[2..].to_vec()),
            bigint(bigint_id, bigint_data[2..].to_vec()),
            int(int_id, int_data[2..].to_vec()),
            smallint(smallint_id, smallint_data[2..].to_vec()),
            scalar(scalar_id, scalar_data[2..].to_vec()),
            varchar(varchar_id, varchar_data[2..].to_vec()),
            bigint(bigint_id, bigint_data[2..].to_vec()),
            int(int_id, int_data[2..].to_vec()),
            smallint(smallint_id, smallint_data[2..].to_vec()),
        ]);

        b.iter(|| {
            let mut local_commitment = table_commitment.clone();
            local_commitment
                .try_append_rows(append_columns.inner_table(), &dory_prover_setup)
                .unwrap();
        });
    });
}

criterion_group!(benches, bench_append_rows);
criterion_main!(benches);

fn gen_rnd_str() -> Vec<String> {
    let mut rng = rand::thread_rng();

    // Create a vector to hold the owned Strings
    let mut string_vec: Vec<String> = Vec::with_capacity(ROW_SIZE);

    for _ in 0..ROW_SIZE {
        // Generate a random string of a fixed length (e.g., 10 characters)
        let random_string: String = (0..10)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect();

        string_vec.push(random_string);
    }

    string_vec
}
