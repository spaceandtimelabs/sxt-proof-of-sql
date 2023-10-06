use blitzar::compute::{init_backend_with_config, BackendConfig};
use clap::Parser;
use proofs::{
    base::{
        database::{
            make_random_test_accessor_data, ColumnType, RandomTestAccessorDescriptor, TestAccessor,
        },
        proof::ProofError,
    },
    sql::{
        parse::QueryExpr,
        proof::{QueryResult, VerifiableQueryResult},
    },
};
use rand::{rngs::StdRng, SeedableRng};
use std::time::Instant;

#[cfg(feature = "valgrind")]
extern "C" {
    pub fn toggle_collect_c();
}

pub fn toggle_collect() {
    #[cfg(feature = "valgrind")]
    unsafe {
        toggle_collect_c();
    }
}

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, allow_negative_numbers = true)]
    pub min_value: i64,
    #[clap(long)]
    pub max_value: i64,
    #[clap(long)]
    pub num_samples: usize,
    #[clap(long)]
    pub num_columns: usize,
    #[clap(long)]
    pub table_length: usize,
    #[clap(long)]
    pub where_expr: String,
    #[clap(long)]
    pub result_columns: String,
}

fn generate_accessor(
    table_length: usize,
    num_columns: usize,
    min_value: i64,
    max_value: i64,
    offset_generators: usize,
) -> (String, TestAccessor) {
    assert!(num_columns < 26);

    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols: Vec<_> = (0..num_columns)
        .map(|val| ((b'a' + (val as u8)) as char).to_string())
        .collect();
    let ref_cols: Vec<_> = cols
        .iter()
        .map(|val| (val.as_str(), ColumnType::BigInt))
        .collect();

    let descriptor = RandomTestAccessorDescriptor {
        min_rows: table_length,
        max_rows: table_length,
        min_value,
        max_value,
    };

    let table_ref = "sxt.t".parse().unwrap();
    let data = make_random_test_accessor_data(&mut rng, &ref_cols[..], &descriptor);
    let mut accessor = TestAccessor::new();
    accessor.add_table(table_ref, data, offset_generators);

    (table_ref.table_id().name().to_owned(), accessor)
}

fn generate_input_data(args: &Args, offset_generators: usize) -> (QueryExpr, TestAccessor, String) {
    init_backend_with_config(BackendConfig {
        num_precomputed_generators: args.table_length as u64,
    });

    let (table_name, accessor) = generate_accessor(
        args.table_length,
        args.num_columns,
        args.min_value,
        args.max_value,
        offset_generators,
    );

    let query = "select ".to_owned()
        + args.result_columns.as_str()
        + " from "
        + table_name.as_str()
        + " where "
        + args.where_expr.as_str();
    let ast = query.parse().unwrap();
    let default_schema = "sxt".parse().unwrap();

    let provable_ast = QueryExpr::try_new(ast, default_schema, &accessor).unwrap();

    (provable_ast, accessor, query)
}

#[tracing::instrument(skip(provable_ast, accessor))]
fn process_query(
    provable_ast: &QueryExpr,
    accessor: &TestAccessor,
    _args: &Args,
    query: &str,
    sample_iter: usize,
) -> Result<QueryResult, ProofError> {
    // generate and verify proof
    let verifiable_result = VerifiableQueryResult::new(provable_ast, accessor);

    verifiable_result.verify(provable_ast, accessor)
}

fn main() {
    let args = Args::parse();
    let offset_generators = 0_usize;

    let (provable_ast, accessor, query) = generate_input_data(&args, offset_generators);

    let mut mean_time: f64 = 0.0;

    toggle_collect();
    for iter in 0..args.num_samples {
        let before = Instant::now();
        let _res = process_query(&provable_ast, &accessor, &args, &query, iter);
        mean_time += before.elapsed().as_secs_f64();
    }
    toggle_collect();

    // convert from seconds to milliseconds
    mean_time = (mean_time / (args.num_samples as f64)) * 1e3;

    println!("{:.4?}seconds", mean_time);
}
