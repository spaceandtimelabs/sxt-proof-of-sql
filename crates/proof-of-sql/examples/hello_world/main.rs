#![doc = include_str!("README.md")]

use blitzar::{compute::init_backend, proof::InnerProductProof};
use proof_of_sql::{
    base::database::{owned_table_utility::{bigint, owned_table, varchar}, OwnedTableTestAccessor, TestAccessor},
    sql::{parse::QueryExpr, proof::QueryProof},
};
use std::{
    io::{stdout, Write},
    time::Instant,
};

/// # Panics
///
/// Will panic if flushing the output fails, which can happen due to issues with the underlying output stream.
fn start_timer(message: &str) -> Instant {
    print!("{message}...");
    stdout().flush().unwrap();
    Instant::now()
}
/// # Panics
///
/// This function does not panic under normal circumstances but may panic if the internal printing fails due to issues with the output stream.
fn end_timer(instant: Instant) {
    println!(" {:?}", instant.elapsed());
}

/// # Panics
///
/// - Will panic if the GPU initialization fails during `init_backend`.
/// - Will panic if the table reference cannot be parsed in `add_table`.
/// - Will panic if the offset provided to `add_table` is invalid.
/// - Will panic if the query string cannot be parsed in `QueryExpr::try_new`.
/// - Will panic if the table reference cannot be parsed in `QueryExpr::try_new`.
/// - Will panic if the query expression creation fails.
/// - Will panic if printing fails during error handling.
fn main() {
    let timer = start_timer("Warming up GPU");
    init_backend();
    end_timer(timer);
    let timer = start_timer("Loading data");
    let mut accessor = OwnedTableTestAccessor::<InnerProductProof>::new_empty_with_setup(());
    accessor.add_table(
        "sxt.table".parse().unwrap(),
        owned_table([
            bigint("a", [1, 2, 3, 2]),
            varchar("b", ["hi", "hello", "there", "world"]),
        ]),
        0,
    );
    end_timer(timer);
    let timer = start_timer("Parsing Query");
    let query = QueryExpr::try_new(
        "SELECT b FROM table WHERE a = 2".parse().unwrap(),
        "sxt".parse().unwrap(),
        &accessor,
    )
    .unwrap();
    end_timer(timer);
    let timer = start_timer("Generating Proof");
    let (proof, serialized_result) =
        QueryProof::<InnerProductProof>::new(query.proof_expr(), &accessor, &());
    end_timer(timer);
    let timer = start_timer("Verifying Proof");
    let result = proof.verify(query.proof_expr(), &accessor, &serialized_result, &());
    end_timer(timer);
    match result {
        Ok(result) => {
            println!("Valid proof!");
            println!("Query result: {:?}", result.table);
        }
        Err(e) => {
            println!("Error: {e:?}");
        }
    }
}
