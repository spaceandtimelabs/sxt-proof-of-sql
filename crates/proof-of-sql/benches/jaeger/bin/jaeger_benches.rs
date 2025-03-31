//! Benchmarking/Tracing binary wrapper using Jaeger.
//!
//! To run, execute the following commands:
//! ```bash
//! docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:1.62.0
//! cargo run --release --bin jaeger_benches --features="bench" -- --help
//! ```
//!
//! # Options
//! - `-s` `--scheme` - Commitment scheme (e.g. `hyper-kzg`, `inner-product-proof`, `dynamic-dory`, `dory`)
//! - `-i` `--iterations` - Number of iterations to run (default: `3`)
//! - `-t` `--table_size` - Number of iterations to run (default: `1_000_000`)
//! - `-q` `--query` - Query (e.g. `single-column-filter`)
//! - `-b` `--blitzar_handle_path` - Path to the Blitzar handle used for `DynamicDory` (Optional)
//! - `-d` `--dory_public_params_path` - Path to the public parameters used for `DynamicDory` (Optional)
//! - `-p` `--ppot_file_path` - Path to the Perpetual Powers of Tau file used for `HyperKZG` (Optional)
//!
//! # Environment Variables
//! - `BLITZAR_HANDLE_PATH` - Path to the Blitzar handle used for `DynamicDory`
//! - `DORY_PUBLIC_PARAMS_PATH` - Path to the public parameters used for `DynamicDory`
//! - `PPOT_FILE_PATH` - Path to the Perpetual Powers of Tau file used for `HyperKZG`
//!
//! Then, navigate to <http://localhost:16686> to view the traces.

use ark_serialize::Validate;
use ark_std::{rand, test_rng};
use blitzar::{compute::init_backend, proof::InnerProductProof};
use bumpalo::Bump;
use clap::{Parser, ValueEnum};
use curve25519_dalek::RistrettoPoint;
use halo2curves::bn256::G2Affine;
use nova_snark::{
    provider::{
        bn256_grumpkin::bn256::Affine,
        hyperkzg::{CommitmentEngine, CommitmentKey, EvaluationEngine},
    },
    traits::{commitment::CommitmentEngineTrait, evaluation::EvaluationEngineTrait},
};
use proof_of_sql::{
    proof_primitive::{
        dory::{
            DoryCommitment, DoryEvaluationProof, DoryProverPublicSetup, DoryVerifierPublicSetup,
            DynamicDoryCommitment, DynamicDoryEvaluationProof, ProverSetup, PublicParameters,
            VerifierSetup,
        },
        hyperkzg::{
            deserialize_flat_compressed_hyperkzg_public_setup_from_reader,
            nova_commitment_key_to_hyperkzg_public_setup, HyperKZGCommitment,
            HyperKZGCommitmentEvaluationProof, HyperKZGEngine,
        },
    },
    sql::{parse::QueryExpr, proof::VerifiableQueryResult},
};
use std::path::PathBuf;
mod utils;
use utils::{
    benchmark_accessor::BenchmarkAccessor,
    jaeger_setup::{setup_jaeger_tracing, stop_jaeger_tracing},
    queries::{get_query, QueryEntry, QUERIES},
    random_util::generate_random_columns,
};

#[derive(ValueEnum, Clone, Debug)]
/// Supported commitment schemes.
enum CommitmentScheme {
    /// `All` runs all commitment schemes
    All,
    /// `InnerProductProof` commitment scheme
    InnerProductProof,
    /// `Dory` commitment scheme
    Dory,
    /// `DynamicDory` commitment scheme
    DynamicDory,
    /// `HyperKZG` commitment scheme
    HyperKZG,
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
    /// Commitment scheme (e.g. `hyper-kzg`, `ipa`, `dynamic-dory`, `dory`)
    #[arg(short, long, value_enum, env, default_value = "all")]
    scheme: CommitmentScheme,

    /// Number of iterations to run (default: `3`)
    #[arg(short, long, env, default_value_t = 3)]
    iterations: usize,

    ///  Size of the table to query against (default: `1_000_000`)
    #[arg(short, long, env, default_value_t = 1_000_000)]
    table_size: usize,

    /// Query (e.g. `single-column`)
    #[arg(short, long, value_enum, env, default_value = "all")]
    query: Query,

    /// Path to the Blitzar handle used for `DynamicDory`.
    #[arg(short, long, env)]
    blitzar_handle_path: Option<PathBuf>,

    /// Path to the public parameters used for `DynamicDory`.
    #[arg(short, long, env)]
    dory_public_params_path: Option<PathBuf>,

    /// Path to the Perpetual Powers of Tau file used for `HyperKZG`.
    #[arg(short, long, env)]
    ppot_file_path: Option<PathBuf>,
}

/// # Panics
///
/// Will panic if:
/// - The table reference cannot be parsed from the string.
/// - The columns generated from `generate_random_columns` lead to a failure in `insert_table`.
/// - The query string cannot be parsed into a `QueryExpr`.
/// - The creation of the `VerifiableQueryResult` fails due to invalid proof expressions.
fn bench_inner_product_proof(cli: &Cli, queries: &[QueryEntry]) {
    let mut accessor: BenchmarkAccessor<'_, RistrettoPoint> = BenchmarkAccessor::default();
    let mut rng = rand::thread_rng();
    let alloc = Bump::new();

    for (_title, query, columns) in queries {
        accessor.insert_table(
            "bench.table".parse().unwrap(),
            &generate_random_columns(&alloc, &mut rng, columns, cli.table_size),
            &(),
        );
        let query_expr =
            QueryExpr::try_new(query.parse().unwrap(), "bench".into(), &accessor).unwrap();

        for _ in 0..cli.iterations {
            let result: VerifiableQueryResult<InnerProductProof> =
                VerifiableQueryResult::new(query_expr.proof_expr(), &accessor, &(), &[]);
            result
                .verify(query_expr.proof_expr(), &accessor, &(), &[])
                .unwrap();
        }
    }
}

/// # Panics
///
/// Will panic if:
/// - The table reference cannot be parsed from the string.
/// - The columns generated from `generate_random_columns` lead to a failure in `insert_table`.
/// - The query string cannot be parsed into a `QueryExpr`.
/// - The creation of the `VerifiableQueryResult` fails due to invalid proof expressions.
fn bench_dory(cli: &Cli, queries: &[QueryEntry]) {
    let pp = PublicParameters::test_rand(10, &mut test_rng());
    let ps = ProverSetup::from(&pp);
    let prover_setup = DoryProverPublicSetup::new(&ps, 10);
    let vs = VerifierSetup::from(&pp);
    let verifier_setup = DoryVerifierPublicSetup::new(&vs, 10);

    let mut accessor: BenchmarkAccessor<'_, DoryCommitment> = BenchmarkAccessor::default();
    let mut rng = rand::thread_rng();
    let alloc = Bump::new();

    for (_title, query, columns) in queries {
        accessor.insert_table(
            "bench.table".parse().unwrap(),
            &generate_random_columns(&alloc, &mut rng, columns, cli.table_size),
            &prover_setup,
        );
        let query_expr =
            QueryExpr::try_new(query.parse().unwrap(), "bench".into(), &accessor).unwrap();

        for _ in 0..cli.iterations {
            let result: VerifiableQueryResult<DoryEvaluationProof> =
                VerifiableQueryResult::new(query_expr.proof_expr(), &accessor, &prover_setup, &[]);
            result
                .verify(query_expr.proof_expr(), &accessor, &verifier_setup, &[])
                .unwrap();
        }
    }
}

/// # Panics
///
/// Will panic if:
/// - The table reference cannot be parsed from the string.
/// - The columns generated from `generate_random_columns` lead to a failure in `insert_table`.
/// - The query string cannot be parsed into a `QueryExpr`.
/// - The creation of the `VerifiableQueryResult` fails due to invalid proof expressions.
/// - If the public parameters file or the Blitzar handle file path is not valid.
fn bench_dynamic_dory(cli: &Cli, queries: &[QueryEntry]) {
    let public_parameters;
    let (prover_setup, verifier_setup) =
        if let (Some(blitzar_handle_path), Some(dory_public_params_path)) =
            (&cli.blitzar_handle_path, &cli.dory_public_params_path)
        {
            let handle =
                blitzar::compute::MsmHandle::new_from_file(blitzar_handle_path.to_str().unwrap());
            public_parameters =
                PublicParameters::load_from_file(std::path::Path::new(&dory_public_params_path))
                    .expect("Failed to load Dory public parameters");
            let prover_setup =
                ProverSetup::from_public_parameters_and_blitzar_handle(&public_parameters, handle);
            let verifier_setup = VerifierSetup::from(&public_parameters);

            (prover_setup, verifier_setup)
        } else {
            public_parameters = PublicParameters::test_rand(11, &mut test_rng());
            let prover = ProverSetup::from(&public_parameters);
            let verifier = VerifierSetup::from(&public_parameters);
            (prover, verifier)
        };

    let mut accessor: BenchmarkAccessor<'_, DynamicDoryCommitment> = BenchmarkAccessor::default();
    let mut rng = rand::thread_rng();
    let alloc = Bump::new();

    for (_title, query, columns) in queries {
        accessor.insert_table(
            "bench.table".parse().unwrap(),
            &generate_random_columns(&alloc, &mut rng, columns, cli.table_size),
            &&prover_setup,
        );
        let query_expr =
            QueryExpr::try_new(query.parse().unwrap(), "bench".into(), &accessor).unwrap();

        for _ in 0..cli.iterations {
            let result: VerifiableQueryResult<DynamicDoryEvaluationProof> =
                VerifiableQueryResult::new(query_expr.proof_expr(), &accessor, &&prover_setup, &[]);
            result
                .verify(query_expr.proof_expr(), &accessor, &&verifier_setup, &[])
                .unwrap();
        }
    }
}

/// # Panics
///
/// Will panic if:
/// - The table reference cannot be parsed from the string.
/// - The columns generated from `generate_random_columns` lead to a failure in `insert_table`.
/// - The query string cannot be parsed into a `QueryExpr`.
/// - The creation of the `VerifiableQueryResult` fails due to invalid proof expressions.
/// - If the public parameters file path is not valid.
fn bench_hyperkzg(cli: &Cli, queries: &[QueryEntry]) {
    let (prover_setup, vk) = if let Some(ppot_file_path) = &cli.ppot_file_path {
        let file = std::fs::File::open(ppot_file_path).unwrap();
        let prover_setup =
            deserialize_flat_compressed_hyperkzg_public_setup_from_reader(&file, Validate::Yes)
                .unwrap();

        let ck: CommitmentKey<HyperKZGEngine> = CommitmentKey::new(
            prover_setup
                .iter()
                .map(blitzar::compute::convert_to_halo2_bn256_g1_affine)
                .collect(),
            Affine::default(),
            G2Affine::default(),
        );
        let (_, vk) = EvaluationEngine::setup(&ck);

        (prover_setup, vk)
    } else {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"bench", cli.table_size);
        let (_, vk) = EvaluationEngine::setup(&ck);
        let prover_setup = nova_commitment_key_to_hyperkzg_public_setup(&ck);
        (prover_setup, vk)
    };

    let mut accessor: BenchmarkAccessor<'_, HyperKZGCommitment> = BenchmarkAccessor::default();
    let mut rng = rand::thread_rng();
    let alloc = Bump::new();

    for (_title, query, columns) in queries {
        accessor.insert_table(
            "bench.table".parse().unwrap(),
            &generate_random_columns(&alloc, &mut rng, columns, cli.table_size),
            &prover_setup.as_slice(),
        );
        let query_expr =
            QueryExpr::try_new(query.parse().unwrap(), "bench".into(), &accessor).unwrap();

        for _ in 0..cli.iterations {
            let result: VerifiableQueryResult<HyperKZGCommitmentEvaluationProof> =
                VerifiableQueryResult::new(
                    query_expr.proof_expr(),
                    &accessor,
                    &prover_setup.as_slice(),
                    &[],
                );
            result
                .verify(query_expr.proof_expr(), &accessor, &&vk, &[])
                .unwrap();
        }
    }
}

/// # Panics
///
/// Will panic if:
/// - If jaeger tracing fails to setup.
/// - If the query type specified is invalid.
/// - If the commitment computation fails.
/// - If the length of the columns does not match after insertion.
/// - If the column reference does not exist in the accessor.
/// - If the creation of the `VerifiableQueryResult` fails due to invalid proof expressions.
/// - If the verification of the `VerifiableQueryResult` fails.
/// - If optional environment variable file paths are not valid.
fn main() {
    #[cfg(debug_assertions)]
    {
        eprintln!("Warning: You are running in debug mode. For accurate benchmarking, run with `cargo run --release`.");
    }

    init_backend();

    setup_jaeger_tracing().expect("Failed to setup Jaeger tracing.");

    let cli = Cli::parse();

    let queries = if cli.query == Query::All {
        QUERIES
    } else {
        let query = get_query(cli.query.to_string()).expect("Invalid query type specified.");
        &[query]
    };

    match cli.scheme {
        CommitmentScheme::All => {
            bench_inner_product_proof(&cli, queries);
            bench_dory(&cli, queries);
            bench_dynamic_dory(&cli, queries);
            bench_hyperkzg(&cli, queries);
        }
        CommitmentScheme::InnerProductProof => {
            bench_inner_product_proof(&cli, queries);
        }
        CommitmentScheme::Dory => {
            bench_dory(&cli, queries);
        }
        CommitmentScheme::DynamicDory => {
            bench_dynamic_dory(&cli, queries);
        }
        CommitmentScheme::HyperKZG => {
            bench_hyperkzg(&cli, queries);
        }
    }

    stop_jaeger_tracing();
}
