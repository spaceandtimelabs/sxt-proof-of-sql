//! This is an non-interactive example of using Proof of SQL with some space related datasets.
//! To run this, use `cargo run --example space`.

// Note: the space_travellers.csv file was obtained from
// https://www.kaggle.com/datasets/kaushiksinghrawat/humans-to-have-visited-space
// under the Apache 2.0 license.

use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use std::fs::File;

fn main() {
    let filename = "./crates/proof-of-sql/examples/space/space_travellers.csv";
    let space_travellers_batch = ReaderBuilder::new(SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ))
    .with_header(true)
    .build(File::open(filename).unwrap())
    .unwrap()
    .next()
    .unwrap()
    .unwrap();
    println!("{space_travellers_batch:?}");
}
