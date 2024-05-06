//! Benchmarking using the `criterion` crate.
//! To run, execute the following command:
//! ```bash
//! cargo bench -p proofs --bench criterion_benches
//! ```
#![allow(missing_docs)]
use blitzar::proof::InnerProductProof;
use criterion::{criterion_group, criterion_main, Criterion};

mod scaffold;
use scaffold::{criterion_scaffold, querys::QUERIES};

const SIZES: &[usize] = &[
    1,
    10,
    100,
    1_000,
    10_000,
    20_000,
    50_000,
    100_000,
    200_000,
    500_000,
    1_000_000,
    2_000_000,
    5_000_000,
    10_000_000,
    20_000_000,
    50_000_000,
    100_000_000,
];

fn all_benches(c: &mut Criterion) {
    for (title, query, columns) in QUERIES {
        criterion_scaffold::<InnerProductProof>(c, title, query, columns, SIZES, &(), &());
    }
}

criterion_group!(benches, all_benches);
criterion_main!(benches);
