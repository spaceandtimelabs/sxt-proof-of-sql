//! Example to use Proof of SQL with census datasets
//! To run, use `cargo run --example census`.

// Note: the census-income.csv was obtained from
// https://github.com/domoritz/vis-examples/blob/master/data/census-income.csv
use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use std::fs::File;

fn main() {
    let filename = "./crates/proof-of-sql/examples/census/census-income.csv";
    let census_income_batch = ReaderBuilder::new(SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ))
    .with_header(true)
    .build(File::open(filename).unwrap())
    .unwrap()
    .next()
    .unwrap()
    .unwrap();
    println!("{census_income_batch:?}");
}
