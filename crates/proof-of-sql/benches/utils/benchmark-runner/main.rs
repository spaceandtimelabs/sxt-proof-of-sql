//! Utility to run Jaeger benchmarks.
use clap::{Parser, ValueEnum};
use std::process::Command;

#[derive(ValueEnum, Clone, Debug)]
/// Supported commitment schemes.
enum CommitmentScheme {
    /// `InnerProductProof` commitment scheme.
    InnerProductProof,
    /// `Dory` commitment scheme.
    Dory,
    /// `DynamicDory` commitment scheme.
    DynamicDory,
    /// `HyperKZG`,
    HyperKZG,
}

impl CommitmentScheme {
    /// Converts the `CommitmentScheme` enum to a string representation.
    pub fn to_string(&self) -> &'static str {
        match self {
            CommitmentScheme::InnerProductProof => "InnerProductProof",
            CommitmentScheme::Dory => "Dory",
            CommitmentScheme::DynamicDory => "DynamicDory",
            CommitmentScheme::HyperKZG => "HyperKZG",
        }
    }
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
/// Supported queries.
enum Query {
    /// All queries
    All,
    /// Single column filter query
    SingleColumnFilter,
    /// Multi column filter query
    MultiColumnFilter,
    /// Arithmetic query
    Arithmetic,
    /// Group by query
    GroupBy,
    /// Aggregate query
    Aggregate,
    /// Boolean filter query
    BooleanFilter,
    /// Large column set query
    LargeColumnSet,
    /// Complex condition query
    ComplexCondition,
}

impl Query {
    /// Converts the `Query` enum to a string representation.
    pub fn to_string(&self) -> &'static str {
        match self {
            Query::All => "All",
            Query::SingleColumnFilter => "Single Column Filter",
            Query::MultiColumnFilter => "Multi Column Filter",
            Query::Arithmetic => "Arithmetic",
            Query::GroupBy => "Group By",
            Query::Aggregate => "Aggregate",
            Query::BooleanFilter => "Boolean Filter",
            Query::LargeColumnSet => "Large Column Set",
            Query::ComplexCondition => "Complex Condition",
        }
    }
}

#[derive(Parser)]
#[command(about, long_about = None)]
struct Cli {
    /// Commitment scheme (e.g. `hyper-kzg`, `ipa`, `dynamic_dory`, `dory`)
    #[arg(long, value_enum, default_value = "hyper-kzg")]
    scheme: CommitmentScheme,

    /// Number of iterations to run (default: `3`)
    #[arg(long, default_value_t = 3)]
    iterations: usize,

    /// Number of iterations to run (default: `1_000_000`)
    #[arg(long, default_value_t = 1_000_000)]
    table_size: usize,

    /// Query (e.g. `single-column`)
    #[arg(long, value_enum, default_value = "all")]
    query: Query,
}

/// # Panics
/// Panics if the `cargo bench` command fails to execute.
fn main() {
    let cli = Cli::parse();

    println!(
        "Running the {:?} query with commitment scheme {:?} on a table of size {} over {} iterations",
        cli.query, cli.scheme, cli.table_size, cli.iterations
    );

    if cli.query == Query::All {
        for query in &[
            Query::SingleColumnFilter,
            Query::MultiColumnFilter,
            Query::Arithmetic,
            Query::GroupBy,
            Query::Aggregate,
            Query::BooleanFilter,
            Query::LargeColumnSet,
            Query::ComplexCondition,
        ] {
            Command::new("cargo")
                .arg("bench")
                .arg("-p")
                .arg("proof-of-sql")
                .arg("--bench")
                .arg("jaeger_benches")
                .arg(cli.scheme.to_string())
                .arg("--features=blitzar, hyperkzg_proof")
                .arg("--")
                .arg(cli.iterations.to_string())
                .arg(cli.table_size.to_string())
                .arg(query.to_string())
                .status()
                .expect("Failed to execute `cargo bench`");
        }
    } else {
        Command::new("cargo")
            .arg("bench")
            .arg("-p")
            .arg("proof-of-sql")
            .arg("--bench")
            .arg("jaeger_benches")
            .arg(cli.scheme.to_string())
            .arg("--features=blitzar, hyperkzg_proof")
            .arg("--")
            .arg(cli.iterations.to_string())
            .arg(cli.table_size.to_string())
            .arg(cli.query.to_string())
            .status()
            .expect("Failed to execute `cargo bench`");
    }
}
