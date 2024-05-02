//! Example demonstrating an implementation of a simple csv-backed database with Proof of SQL capabilities.
//!
//! # Install
//! Run `cargo install --example csv_db --path crates/proofs` to install the example.
//! # Quick Start Exmaple
//! Run the following
//! ```bash
//! csv_db create -t sxt.table -c a,b -d BIGINT,VARCHAR
//! csv_db append -t sxt.table -f hello_world.csv
//! csv_db prove -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof
//! csv_db verify -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof
//! ```
mod commit_accessor;
mod csv_accessor;
mod record_batch_accessor;
use arrow::{
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use blitzar::proof::InnerProductProof;
use clap::{arg, Parser, Subcommand, ValueEnum};
use commit_accessor::CommitAccessor;
use csv_accessor::{read_record_batch_from_csv, CsvDataAccessor};
use curve25519_dalek::RistrettoPoint;
use itertools::Itertools;
use proofs::{
    base::{
        commitment::TableCommitment,
        database::{SchemaAccessor, TableRef},
    },
    sql::{parse::QueryExpr, proof::VerifiableQueryResult},
};
use proofs_sql::{Identifier, SelectStatement};
use std::{
    fs,
    io::{prelude::Write, stdout},
    path::PathBuf,
    sync::Arc,
    time::Instant,
};

/// Command line interface demonstrating an implementation of a simple csv-backed database with Proof of SQL capabilities.
#[derive(Parser, Debug)]
#[command()]
struct CliArgs {
    /// Path to the directory where the csv files are stored.
    #[arg(short, long, default_value = ".")]
    path: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum, Debug)]
#[value(rename_all = "UPPER")]
enum CsvDataType {
    BigInt,
    VarChar,
    Decimal,
}
impl From<&CsvDataType> for DataType {
    fn from(t: &CsvDataType) -> Self {
        match t {
            CsvDataType::BigInt => DataType::Int64,
            CsvDataType::VarChar => DataType::Utf8,
            CsvDataType::Decimal => DataType::Decimal256(75, 30),
        }
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Creates a new csv for an empty table and initializes the commitment of that table.
    ///
    /// Example: `csv_db create -t sxt.table -c a,b -d BIGINT,VARCHAR`
    Create {
        /// The table to create. The table name should be in the format `schema.table`.
        #[arg(short, long)]
        table: TableRef,
        /// The comma delimited column names of the table.
        #[arg(short, long, value_parser, num_args = 0.., value_delimiter = ',')]
        columns: Vec<Identifier>,
        /// The comma delimited data types of the columns.
        #[arg(short, long, value_parser, num_args = 0.., value_delimiter = ',')]
        data_types: Vec<CsvDataType>,
    },
    /// Appends a csv file to an existing table and updates the commitment of that table.
    ///
    /// Example: `csv_db append -t sxt.table -f hello_world.csv`
    Append {
        /// The table to append to. The table name should be in the format `schema.table`.
        #[arg(short, long)]
        table: TableRef,
        /// The file name of the csv file to append.
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Proves a query and writes the proof to a file.
    ///
    /// Example: `csv_db prove -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof`
    Prove {
        /// The query to prove. Note: the default schema is `example`.
        #[arg(short, long)]
        query: SelectStatement,
        /// The file name of the file to write the proof to.
        #[arg(short, long)]
        file: PathBuf,
    },
    /// Verifies a proof of a query and prints the result.
    ///
    /// Example: `csv_db verify -q "SELECT b FROM sxt.table WHERE a = 2" -f hello.proof`
    Verify {
        /// The query to verify. Note: the default schema is `example`.
        #[arg(short, long)]
        query: SelectStatement,
        /// The file name of the file to read the proof from.
        #[arg(short, long)]
        file: PathBuf,
    },
}

fn start_timer(message: &str) -> Instant {
    print!("{}...", message);
    stdout().flush().unwrap();
    Instant::now()
}
fn end_timer(instant: Instant) {
    println!(" {:?}", instant.elapsed());
}

fn main() {
    let args = CliArgs::parse();
    println!("Warming up GPU...");
    blitzar::compute::init_backend();
    println!("Done.");
    match args.command {
        Commands::Create {
            table,
            columns,
            data_types,
        } => {
            let commit_accessor =
                CommitAccessor::<RistrettoPoint>::new(PathBuf::from(args.path.clone()));
            let csv_accessor = CsvDataAccessor::new(PathBuf::from(args.path));
            let schema = Schema::new(
                columns
                    .iter()
                    .zip_eq(data_types.iter())
                    .map(|(name, data_type)| Field::new(name.as_str(), data_type.into(), false))
                    .collect::<Vec<_>>(),
            );
            let batch = RecordBatch::new_empty(Arc::new(schema));
            let table_commitment = TableCommitment::try_from_record_batch(&batch, &())
                .expect("Failed to create table commitment.");
            commit_accessor
                .write_commit(&table, &table_commitment)
                .expect("Failed to write commit");
            csv_accessor
                .write_table(&table, &batch)
                .expect("Failed to write table");
        }
        Commands::Append {
            table: table_name,
            file: file_path,
        } => {
            let mut commit_accessor =
                CommitAccessor::<RistrettoPoint>::new(PathBuf::from(args.path.clone()));
            let csv_accessor = CsvDataAccessor::new(PathBuf::from(args.path));
            commit_accessor
                .load_commit(table_name)
                .expect("Failed to load commit");
            let mut table_commitment = commit_accessor.get_commit(&table_name).unwrap().clone();
            let schema = Schema::new(
                commit_accessor
                    .lookup_schema(table_name)
                    .iter()
                    .map(|(i, t)| Field::new(i.as_str(), t.into(), false))
                    .collect::<Vec<_>>(),
            );
            let append_batch =
                read_record_batch_from_csv(schema, &file_path).expect("Failed to read csv file.");
            csv_accessor
                .append_batch(&table_name, &append_batch)
                .expect("Failed to write batch");
            let timer = start_timer("Updating Commitment");
            table_commitment
                .try_append_record_batch(&append_batch, &())
                .expect("Failed to append batch");
            end_timer(timer);
            commit_accessor
                .write_commit(&table_name, &table_commitment)
                .expect("Failed to write commit");
        }
        Commands::Prove { query, file } => {
            let mut commit_accessor =
                CommitAccessor::<RistrettoPoint>::new(PathBuf::from(args.path.clone()));
            let mut csv_accessor = CsvDataAccessor::new(PathBuf::from(args.path.clone()));
            let tables = query.get_table_references("example".parse().unwrap());
            for table in tables.into_iter().map(TableRef::new) {
                commit_accessor
                    .load_commit(table)
                    .expect("Failed to load commit");
                let schema = Schema::new(
                    commit_accessor
                        .lookup_schema(table)
                        .iter()
                        .map(|(i, t)| Field::new(i.as_str(), t.into(), false))
                        .collect::<Vec<_>>(),
                );
                csv_accessor
                    .load_table(table, schema)
                    .expect("Failed to load table");
            }
            let query =
                QueryExpr::try_new(query, "example".parse().unwrap(), &commit_accessor).unwrap();
            let timer = start_timer("Generating Proof");
            let proof = VerifiableQueryResult::<InnerProductProof>::new(
                query.proof_expr(),
                &csv_accessor,
                &(),
            );
            end_timer(timer);
            fs::write(
                file,
                postcard::to_allocvec(&proof).expect("Failed to serialize proof"),
            )
            .expect("Failed to write proof");
        }
        Commands::Verify { query, file } => {
            let mut commit_accessor =
                CommitAccessor::<RistrettoPoint>::new(PathBuf::from(args.path.clone()));
            let table_refs = query.get_table_references("example".parse().unwrap());
            for table_ref in table_refs {
                let table_name = TableRef::new(table_ref);
                commit_accessor
                    .load_commit(table_name)
                    .expect("Failed to load commit");
            }
            let query =
                QueryExpr::try_new(query, "example".parse().unwrap(), &commit_accessor).unwrap();
            let result: VerifiableQueryResult<InnerProductProof> =
                postcard::from_bytes(&fs::read(file).expect("Failed to read proof"))
                    .expect("Failed to deserialize proof");
            let timer = start_timer("Verifying Proof");
            let query_result = result
                .verify(query.proof_expr(), &commit_accessor, &())
                .expect("Failed to verify proof");
            end_timer(timer);
            println!(
                "Verified Result: {:?}",
                RecordBatch::try_from(query_result).unwrap()
            );
        }
    }
}
