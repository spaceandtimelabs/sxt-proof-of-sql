//! This is an non-interactive example of using Proof of SQL with some sushi related datasets.
//! To run this, use `cargo run --example sushi`.

//! NOTE: If this doesn't work because you do not have the appropriate GPU drivers installed,
//! you can run `cargo run --release --example space --no-default-features --features="arrow cpu-perf"` instead. It will be slower for proof generation.
use arrow::datatypes::SchemaRef;
use arrow_csv::{infer_schema_from_files, ReaderBuilder};
use std::fs::File;

fn main() {
	let filename = "./crates/proof-of-sql/examples/sushi/fish.csv";
    let fish_batch = ReaderBuilder::new(SchemaRef::new(
        infer_schema_from_files(&[filename.to_string()], b',', None, true).unwrap(),
    ))
		.with_header(true)
		.build(File::open(filename).unwrap())
		.unwrap()
		.next()
		.unwrap()
		.unwrap();
    println!("{fish_batch:?}");
}