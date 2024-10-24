//! Example to use Proof of SQL with datasets
//! To run, use `cargo run --example avocado-prices`.
use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use std::fs::File;

fn main() {
    let filename = "./crates/proof-of-sql/examples/avocado-prices/avocado-prices.csv";
    let data_batch = ReaderBuilder::new(SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ))
    .with_header(true)
    .build(File::open(filename).unwrap())
    .unwrap()
    .next()
    .unwrap()
    .unwrap();
    println!("{data_batch:?}");
}
