//! Utility to deserialize and print a commitment from a file or stdin.
use clap::{Parser, ValueEnum};
use curve25519_dalek::ristretto::RistrettoPoint;
use proof_of_sql::{
    base::commitment::TableCommitment,
    proof_primitive::dory::{DoryCommitment, DynamicDoryCommitment},
};
use snafu::Snafu;
use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};

#[derive(ValueEnum, Clone, Debug)]
/// Supported commitment schemes.
enum CommitmentScheme {
    /// Inner Product Argument (IPA) commitment scheme.
    Ipa,
    /// Dory commitment scheme.
    Dory,
    /// Dynamic Dory commitment scheme.
    DynamicDory,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file (defaults to None which is stdin)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file (defaults to None which is stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Commitment scheme (e.g. `ipa`, `dynamic_dory`, `dory`)
    #[arg(long, value_enum, default_value = "CommitmentScheme::DynamicDory")]
    scheme: CommitmentScheme,
}

#[derive(Debug, Snafu)]
enum CommitUtilityError {
    #[snafu(display("Failed to open input file '{:?}'", filename))]
    OpenInputFile { filename: PathBuf },

    #[snafu(display("Failed to read from input file '{:?}'", filename))]
    ReadInputFile { filename: PathBuf },

    #[snafu(display("Failed to read from stdin"))]
    ReadStdin,

    #[snafu(display("Failed to create output file '{:?}'", filename))]
    CreateOutputFile { filename: PathBuf },

    #[snafu(display("Failed to write to output file '{:?}'", filename))]
    WriteOutputFile { filename: PathBuf },

    #[snafu(display("Failed to write to stdout"))]
    WriteStdout,

    #[snafu(display("Failed to deserialize commitment"))]
    DeserializationError,
}

type CommitUtilityResult<T, E = CommitUtilityError> = std::result::Result<T, E>;

fn main() -> CommitUtilityResult<()> {
    let cli = Cli::parse();

    // Read input data
    let input_data = match &cli.input {
        Some(input_file) => {
            let mut file =
                File::open(input_file).map_err(|_| CommitUtilityError::OpenInputFile {
                    filename: input_file.clone(),
                })?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|_| CommitUtilityError::ReadInputFile {
                    filename: input_file.clone(),
                })?;
            buffer
        }
        None => {
            let mut buffer = Vec::new();
            io::stdin()
                .read_to_end(&mut buffer)
                .map_err(|_| CommitUtilityError::ReadStdin)?;
            buffer
        }
    };

    // Deserialize commitment based on the scheme
    let human_readable = match cli.scheme {
        CommitmentScheme::DynamicDory => {
            let commitment: TableCommitment<DynamicDoryCommitment> =
                postcard::from_bytes(&input_data)
                    .map_err(|_| CommitUtilityError::DeserializationError)?;
            format!("{commitment:#?}")
        }
        CommitmentScheme::Dory => {
            let commitment: TableCommitment<DoryCommitment> = postcard::from_bytes(&input_data)
                .map_err(|_| CommitUtilityError::DeserializationError)?;
            format!("{commitment:#?}")
        }
        CommitmentScheme::Ipa => {
            let commitment: TableCommitment<RistrettoPoint> = postcard::from_bytes(&input_data)
                .map_err(|_| CommitUtilityError::DeserializationError)?;
            format!("{commitment:#?}")
        }
    };

    // Write output data
    match &cli.output {
        Some(output_file) => {
            let mut file =
                File::create(output_file).map_err(|_| CommitUtilityError::CreateOutputFile {
                    filename: output_file.clone(),
                })?;
            file.write_all(human_readable.as_bytes()).map_err(|_| {
                CommitUtilityError::WriteOutputFile {
                    filename: output_file.clone(),
                }
            })?;
        }
        None => {
            io::stdout()
                .write_all(human_readable.as_bytes())
                .map_err(|_| CommitUtilityError::WriteStdout)?;
        }
    }

    Ok(())
}
