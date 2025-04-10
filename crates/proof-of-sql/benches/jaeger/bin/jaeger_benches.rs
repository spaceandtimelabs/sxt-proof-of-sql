//! Benchmarking/Tracing binary wrapper.
//!
//! To run, execute the following commands:
//! ```bash
//! docker run --rm -d --name jaeger -p 6831:6831/udp -p 16686:16686 jaegertracing/all-in-one:1.62.0
//! cargo run --release --bin jaeger_benches --features="bench" -- --help
//! ```
//! Then, navigate to <http://localhost:16686> to view the traces.
//!
//! # Options
//! - `-s` `--scheme` - Commitment scheme (e.g. `hyper-kzg`, `inner-product-proof`, `dynamic-dory`, `dory`)
//! - `-i` `--iterations` - Number of iterations to run (default: `3`)
//! - `-t` `--table_size` - Number of iterations to run (default: `1_000_000`)
//! - `-q` `--query` - Query (e.g. `single-column-filter`)
//! - `-n` `--nu_sigma` - `max_nu` used in the Dynamic Dory or `sigma` used in the Dory setup (default: `11`)
//! - `-r` `--rand_seed` - Optional random seed for deterministic random number generation
//! - `-b` `--blitzar_handle_path` - Path to the Blitzar handle used for `DynamicDory` (Optional)
//! - `-d` `--dory_public_params_path` - Path to the public parameters used for `DynamicDory` (Optional)
//! - `-p` `--ppot_path` - Path to the Perpetual Powers of Tau file used for `HyperKZG` (Optional)
//!
//! # Optional File Path Environment Variables
//! - `BLITZAR_HANDLE_PATH` - Path to the Blitzar handle used for `Dory` and `DynamicDory` commitment schemes
//! - `DORY_PUBLIC_PARAMS_PATH` - Path to the public parameters used for `Dory` and `DynamicDory` commitment schemes
//! - `PPOT_PATH` - Path to the Perpetual Powers of Tau file used for `HyperKZG` commitment scheme

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
use rand::{rngs::StdRng, SeedableRng};
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
    /// Commitment scheme (e.g. `hyper-kzg`, `inner-product-proof`, `dynamic-dory`, `dory`)
    #[arg(short, long, value_enum, env, default_value = "all")]
    scheme: CommitmentScheme,

    /// Number of iterations to run (default: `3`)
    #[arg(short, long, env, default_value_t = 3)]
    iterations: usize,

    ///  Size of the table to query against (default: `1_000_000`)
    #[arg(short, long, env, default_value_t = 1_000_000)]
    table_size: usize,

    /// Query to run tracing on (default: `all`)
    #[arg(short, long, value_enum, env, default_value = "all")]
    query: Query,

    /// `max_nu` used in the Dynamic Dory or `sigma` used in the Dory setup (default: `11`)
    #[arg(short, long, env, default_value_t = 11)]
    nu_sigma: usize,

    /// Optional random seed for deterministic random number generation
    #[arg(short, long, env)]
    rand_seed: Option<u64>,

    /// Optional path to the Blitzar handle used for the `Dory` and `DynamicDory` commitment schemes
    #[arg(short, long, env)]
    blitzar_handle_path: Option<PathBuf>,

    /// Optional path to the public parameters used for the `Dory` and `DynamicDory` commitment schemes
    #[arg(short, long, env)]
    dory_public_params_path: Option<PathBuf>,

    /// Optional path to the Perpetual Powers of Tau file used for `HyperKZG`
    #[arg(short, long, env)]
    ppot_path: Option<PathBuf>,
}

/// Gets a random number generator based on the CLI arguments.
/// If a seed is provided, uses a seeded RNG, otherwise uses `thread_rng`.
///
/// # Arguments
/// * `cli` - A reference to the command line interface arguments.
fn get_rng(cli: &Cli) -> StdRng {
    if let Some(seed) = cli.rand_seed {
        StdRng::seed_from_u64(seed)
    } else {
        StdRng::from_entropy()
    }
}

/// Benchmarks the `InnerProductProof` scheme.
///
/// # Arguments
/// * `cli` - A reference to the command line interface arguments.
/// * `queries` - A slice of query entries to benchmark.
///
/// # Panics
/// * The table reference cannot be parsed from the string.
/// * The columns generated from `generate_random_columns` lead to a failure in `insert_table`.
/// * The query string cannot be parsed into a `QueryExpr`.
/// * The creation of the `VerifiableQueryResult` fails due to invalid proof expressions.
/// * If the verification of the `VerifiableQueryResult` fails.
fn bench_inner_product_proof(cli: &Cli, queries: &[QueryEntry]) {
    let mut accessor: BenchmarkAccessor<'_, RistrettoPoint> = BenchmarkAccessor::default();
    let mut rng = get_rng(cli);
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
                VerifiableQueryResult::new(query_expr.proof_expr(), &accessor, &(), &[]).unwrap();
            result
                .verify(query_expr.proof_expr(), &accessor, &(), &[])
                .unwrap();
        }
    }
}

/// # Panics
/// * The optional Dory public parameters file is defined but can't be loaded.
fn load_dory_public_parameters(cli: &Cli) -> PublicParameters {
    if let Some(dory_public_params_path) = &cli.dory_public_params_path {
        PublicParameters::load_from_file(std::path::Path::new(&dory_public_params_path))
            .expect("Failed to load Dory public parameters")
    } else {
        PublicParameters::test_rand(cli.nu_sigma, &mut test_rng())
    }
}

/// Loads the Dory setup for the given public parameters.
///
/// # Arguments
/// * `public_parameters` - A reference to the public parameters.
/// * `cli` - A reference to the command line interface arguments.
///
/// # Panics
/// * The Blitzar handle path cannot be parsed from the string.
fn load_dory_setup<'a>(
    public_parameters: &'a PublicParameters,
    cli: &'a Cli,
) -> (ProverSetup<'a>, VerifierSetup) {
    let (prover_setup, verifier_setup) = if let Some(blitzar_handle_path) = &cli.blitzar_handle_path
    {
        let handle =
            blitzar::compute::MsmHandle::new_from_file(blitzar_handle_path.to_str().unwrap());
        let prover_setup =
            ProverSetup::from_public_parameters_and_blitzar_handle(public_parameters, handle);
        let verifier_setup = VerifierSetup::from(public_parameters);

        (prover_setup, verifier_setup)
    } else {
        let prover_setup = ProverSetup::from(public_parameters);
        let verifier_setup = VerifierSetup::from(public_parameters);
        (prover_setup, verifier_setup)
    };

    (prover_setup, verifier_setup)
}

/// Benchmarks the `Dory` scheme.
///
/// # Arguments
/// * `cli` - A reference to the command line interface arguments.
/// * `queries` - A slice of query entries to benchmark.
///
/// # Panics
/// * The table reference cannot be parsed from the string.
/// * The columns generated from `generate_random_columns` lead to a failure in `insert_table`.
/// * The query string cannot be parsed into a `QueryExpr`.
/// * The creation of the `VerifiableQueryResult` fails due to invalid proof expressions.
/// * If the verification of the `VerifiableQueryResult` fails.
fn bench_dory(cli: &Cli, queries: &[QueryEntry]) {
    let public_parameters = load_dory_public_parameters(cli);
    let (prover_setup, verifier_setup) = load_dory_setup(&public_parameters, cli);

    let prover_public_setup = DoryProverPublicSetup::new(&prover_setup, cli.nu_sigma);
    let verifier_public_setup = DoryVerifierPublicSetup::new(&verifier_setup, cli.nu_sigma);

    let mut accessor: BenchmarkAccessor<'_, DoryCommitment> = BenchmarkAccessor::default();
    let mut rng = get_rng(cli);
    let alloc = Bump::new();

    for (_title, query, columns) in queries {
        accessor.insert_table(
            "bench.table".parse().unwrap(),
            &generate_random_columns(&alloc, &mut rng, columns, cli.table_size),
            &prover_public_setup,
        );
        let query_expr =
            QueryExpr::try_new(query.parse().unwrap(), "bench".into(), &accessor).unwrap();

        for _ in 0..cli.iterations {
            let result: VerifiableQueryResult<DoryEvaluationProof> = VerifiableQueryResult::new(
                query_expr.proof_expr(),
                &accessor,
                &prover_public_setup,
                &[],
            )
            .unwrap();
            result
                .verify(
                    query_expr.proof_expr(),
                    &accessor,
                    &verifier_public_setup,
                    &[],
                )
                .unwrap();
        }
    }
}

/// # Panics
/// * The table reference cannot be parsed from the string.
/// * The columns generated from `generate_random_columns` lead to a failure in `insert_table`.
/// * The query string cannot be parsed into a `QueryExpr`.
/// * The creation of the `VerifiableQueryResult` fails due to invalid proof expressions.
/// * If the verification of the `VerifiableQueryResult` fails.
fn bench_dynamic_dory(cli: &Cli, queries: &[QueryEntry]) {
    let public_parameters = load_dory_public_parameters(cli);
    let (prover_setup, verifier_setup) = load_dory_setup(&public_parameters, cli);

    let mut accessor: BenchmarkAccessor<'_, DynamicDoryCommitment> = BenchmarkAccessor::default();
    let mut rng = get_rng(cli);
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
                VerifiableQueryResult::new(query_expr.proof_expr(), &accessor, &&prover_setup, &[])
                    .unwrap();
            result
                .verify(query_expr.proof_expr(), &accessor, &&verifier_setup, &[])
                .unwrap();
        }
    }
}

/// # Panics
/// * The table reference cannot be parsed from the string.
/// * The columns generated from `generate_random_columns` lead to a failure in `insert_table`.
/// * The query string cannot be parsed into a `QueryExpr`.
/// * The creation of the `VerifiableQueryResult` fails due to invalid proof expressions.
/// * If the verification of the `VerifiableQueryResult` fails.
fn bench_hyperkzg(cli: &Cli, queries: &[QueryEntry]) {
    // Load the prover setup and verification key
    let (prover_setup, vk) = if let Some(ppot_file_path) = &cli.ppot_path {
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
    let mut rng = get_rng(cli);
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
                )
                .unwrap();
            result
                .verify(query_expr.proof_expr(), &accessor, &&vk, &[])
                .unwrap();
        }
    }
}

/// # Panics
/// * If Jaeger tracing fails to setup.
/// * If the query type specified is invalid.
/// * If the commitment computation fails.
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
