use clap::Parser;
use pedersen::compute::{init_backend_with_config, BackendConfig};
use proofs::base::database::{
    make_random_test_accessor, RandomTestAccessorDescriptor, TestAccessor,
};
use proofs::sql::ast::FilterExpr;
use proofs::sql::parse::Converter;
use proofs::sql::proof::VerifiableQueryResult;
use proofs_sql::sql::SelectStatementParser;
use rand::rngs::StdRng;
use rand::SeedableRng;
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

fn parse_query(query: String, accessor: &TestAccessor) -> FilterExpr {
    let intermediate_ast = SelectStatementParser::new().parse(&query).unwrap();

    Converter::default()
        .visit_intermediate_ast(&intermediate_ast, accessor)
        .unwrap()
}

fn generate_accessor(
    table_length: usize,
    num_columns: usize,
    min_value: i64,
    max_value: i64,
) -> (String, TestAccessor) {
    assert!(num_columns < 26);

    let mut rng = StdRng::from_seed([0u8; 32]);
    let cols: Vec<_> = (0..num_columns)
        .map(|val| ((b'A' + (val as u8)) as char).to_string())
        .collect();
    let ref_cols: Vec<&str> = cols.iter().map(|val| val.as_str()).collect();

    let descriptor = RandomTestAccessorDescriptor {
        min_rows: table_length,
        max_rows: table_length,
        min_value,
        max_value,
    };

    let table_name = "T".to_string();
    let accessor = make_random_test_accessor(&mut rng, &table_name, &ref_cols[..], &descriptor);

    (table_name, accessor)
}

fn generate_input_data(args: &Args) -> (FilterExpr, TestAccessor) {
    init_backend_with_config(BackendConfig {
        num_precomputed_generators: args.table_length as u64,
    });

    let (table_name, accessor) = generate_accessor(
        args.table_length,
        args.num_columns,
        args.min_value,
        args.max_value,
    );

    let query = "select ".to_owned()
        + args.result_columns.as_str()
        + " from "
        + table_name.as_str()
        + " where "
        + args.where_expr.as_str();

    let provable_ast = parse_query(query, &accessor);

    (provable_ast, accessor)
}

fn main() {
    let args = Args::parse();

    let (provable_ast, accessor) = generate_input_data(&args);

    let mut mean_time: f64 = 0.0;

    toggle_collect();
    for _ in 0..args.num_samples {
        let before = Instant::now();

        // generate and verify proof
        let verifiable_result = VerifiableQueryResult::new(&provable_ast, &accessor);

        verifiable_result
            .verify(&provable_ast, &accessor)
            .unwrap()
            .unwrap();

        mean_time += before.elapsed().as_secs_f64();
    }
    toggle_collect();

    // convert from seconds to milliseconds
    mean_time = (mean_time / (args.num_samples as f64)) * 1e3;

    println!("{:.4?}", mean_time);
}
