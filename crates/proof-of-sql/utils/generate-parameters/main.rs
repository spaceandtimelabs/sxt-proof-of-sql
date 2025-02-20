//! A CLI utility to generate the public parameters for the prover and verifier

#[cfg(test)]
mod round_trip_test;

use ark_std::rand::SeedableRng;
use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use proof_of_sql::proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup};
use rand_chacha::ChaCha20Rng;
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

/// Transparent public randomness
const SEED: &str = "SpaceAndTime";

const BLITZAR_PARTITION_WINDOW_WIDTH: &str = "BLITZAR_PARTITION_WINDOW_WIDTH";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The value for `nu` (number of public parameters)
    #[arg(short, long, default_value_t = 8)]
    nu: usize,

    /// Mode for generating parameters: "p" for Prover, "v" for Verifier, pv for both
    #[arg(short, long, default_value = "all")]
    mode: Mode,

    /// The initial randomness for the transparent setup
    #[arg(short, long, default_value = SEED)]
    seed: String,

    /// The directory to store generated files and archives
    #[arg(short, long, default_value = "./output")]
    target: String,
}

// An enum representing possible modes of operation,
// abbreviated to keep clap commands concise, instead
// of requiring the user to type "ProverAndVerifier",
// they type "pv"
#[derive(Debug, Clone, ValueEnum)]
enum Mode {
    Prover,   //Prover
    Verifier, //verifier
    All,      //Both
}

fn main() {
    // Set the BLITZAR_PARTITION_WINDOW_WIDTH environment variable
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { env::set_var(BLITZAR_PARTITION_WINDOW_WIDTH, "14") };

    // Confirm that it was set by reading it back
    match env::var(BLITZAR_PARTITION_WINDOW_WIDTH) {
        Ok(value) => {
            println!("Environment variable {BLITZAR_PARTITION_WINDOW_WIDTH} set to {value}");
        }
        Err(e) => {
            eprintln!("Failed to set {BLITZAR_PARTITION_WINDOW_WIDTH}: {e}");
        }
    }

    // Parse command-line arguments
    let args = Args::parse();

    // Ensure the target directory exists
    if let Ok(()) = fs::create_dir_all(&args.target) {
        generate_parameters(&args);
    } else {
        eprintln!(
            "Skipping generation, failed to write or create target directory: {}. Check path and try again.",
            args.target,
        );
        std::process::exit(-1)
    };
}

fn generate_parameters(args: &Args) {
    // Clear out the digests.txt file if it already exists
    let digests_path = format!("{}/digests_nu_{}.txt", args.target, args.nu);
    if Path::new(&digests_path).exists() {
        match fs::write(&digests_path, "") {
            Ok(()) => {}
            Err(e) => {
                eprintln!("Failed to clear digests.txt file: {e}");
                std::process::exit(-1)
            }
        }
    }

    let mut rng = rng_from_seed(args);

    let spinner = spinner(format!(
        "Generating a random public setup with seed {SEED:?} please wait..."
    ));

    // Obtain public parameter from nu
    let public_parameters = PublicParameters::rand(args.nu, &mut rng);
    spinner.finish_with_message("Public parameter setup complete");

    match args.mode {
        Mode::All => {
            println!("Generating parameters for Prover...");
            generate_prover_setup(&public_parameters, args.nu, &args.target);
            println!("Generating parameters for Verifier...");
            generate_verifier_setup(&public_parameters, args.nu, &args.target);
        }
        Mode::Prover => {
            println!("Generating parameters for Prover...");
            generate_prover_setup(&public_parameters, args.nu, &args.target);
        }
        Mode::Verifier => {
            println!("Generating parameters for Verifier...");
            generate_verifier_setup(&public_parameters, args.nu, &args.target);
        }
    }
}

/// # Panics
/// expects that a [u8; 32] always contains 32 elements, guaranteed not to panic
fn rng_from_seed(args: &Args) -> ChaCha20Rng {
    // Convert the seed string to bytes and create a seeded RNG
    let seed_bytes = args
        .seed
        .bytes()
        .chain(std::iter::repeat(0u8))
        .take(32)
        .collect::<Vec<_>>()
        .try_into()
        .expect("collection is guaranteed to contain 32 elements");
    ChaCha20Rng::from_seed(seed_bytes)
}

/// Generates and writes the ```ProverSetup``` from initial public parameters
fn generate_prover_setup(public_parameters: &PublicParameters, nu: usize, target: &str) {
    let spinner = spinner(
        "Generating parameters for the SxT network. This may take a long time, please wait..."
            .into(),
    );

    let start_time = Instant::now();

    // Heavy operation
    let setup = ProverSetup::from(public_parameters);

    spinner.finish_with_message("Prover setup complete.");
    let duration = start_time.elapsed();
    println!("Generated prover setup in {duration:.2?}");

    let public_parameters_path = format!("{target}/public_parameters_nu_{nu}.bin");
    let param_save_result = public_parameters.save_to_file(Path::new(&public_parameters_path));
    let file_path = format!("{target}/blitzar_handle_nu_{nu}.bin");

    match param_save_result {
        Ok(()) => {
            write_prover_blitzar_handle(setup, &file_path);

            // Compute and save SHA-256
            let mut digests = Vec::new();
            let public_parameters_digest = compute_sha256(&public_parameters_path);
            let blitzar_handle_digest = compute_sha256(&file_path);
            if let Some(digest) = public_parameters_digest {
                digests.push((public_parameters_path.clone(), digest));
            }
            if let Some(digest) = blitzar_handle_digest {
                digests.push((file_path.clone(), digest));
            }
            save_digests(&digests, target, nu); // Save digests to digests.txt
        }
        Err(e) => {
            eprintln!("Failed to save prover setup: {e}.");
            std::process::exit(-1)
        }
    }
}

// Generates and writes the VerifierSetup from initial public parameters
fn generate_verifier_setup(public_parameters: &PublicParameters, nu: usize, target: &str) {
    let spinner = spinner(
        "Generating parameters for the SxT network. This may take a long time, please wait..."
            .into(),
    );

    let start_time = Instant::now();

    // Heavy operation
    let setup = VerifierSetup::from(public_parameters);

    spinner.finish_with_message("Verifier setup complete.");
    let duration = start_time.elapsed();
    println!("Generated verifier setup in {duration:.2?}");

    let file_path = format!("{target}/verifier_setup_nu_{nu}.bin");
    let result = write_verifier_setup(&setup, &file_path);

    match result {
        Ok(()) => {
            println!("Verifier setup saved successfully.");

            // Compute and save SHA-256
            let mut digests = Vec::new();
            if let Some(digest) = compute_sha256(&file_path) {
                digests.push((file_path.clone(), digest));
            }
            save_digests(&digests, target, nu); // Save digests to digests.txt
        }
        Err(e) => {
            eprintln!("Failed to save verifier setup: {e}.");
            std::process::exit(-1)
        }
    }
}

// Function to compute SHA-256 hash of a file
fn compute_sha256(file_path: &str) -> Option<String> {
    let mut file = File::open(file_path).ok()?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).ok()?;
    Some(format!("{:x}", hasher.finalize()))
}

/// Function to save digests to a file, or print to console if file saving fails
fn save_digests(digests: &[(String, String)], target: &str, nu: usize) {
    let digests_path = format!("{target}/digests_nu_{nu}.txt");

    // Attempt to open file in append mode, creating it if it doesn't exist
    let mut file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&digests_path)
    {
        Ok(f) => Some(f),
        _ => {
            println!(
                "Failed to open or create file at {digests_path}. Printing digests to console."
            );
            None
        }
    };

    for (file_path, digest) in digests {
        if let Some(f) = &mut file {
            // Attempt to write to file, fall back to printing if it fails
            if writeln!(f, "{digest}  {file_path}").is_err() {
                println!(
                    "Failed to write to {digests_path}. Printing remaining digests to console."
                );
                file = None; // Stop trying to write to the file
            }
        }

        if file.is_none() {
            println!("{digest}  {file_path}");
        }
    }

    if file.is_some() {
        println!("Digests saved to {digests_path}");
    }
}

fn write_prover_blitzar_handle(setup: ProverSetup<'_>, file_path: &str) {
    let blitzar_handle = setup.blitzar_handle();
    blitzar_handle.write(file_path);

    // Check the file size to see if it exceeds 2 GB
    let metadata_res = fs::metadata(file_path);
    match metadata_res {
        Ok(m) => {
            let file_size = m.len();

            if file_size > 2 * 1024 * 1024 * 1024 {
                // 2 GB in bytes
                println!("Handle size exceeds 2 GB, splitting into parts...");

                // Run `split` command to divide the file into 2.0 GB parts
                let split_output = Command::new("split")
                    .arg("-b")
                    .arg("2000M")
                    .arg(file_path)
                    .arg(format!("{file_path}.part."))
                    .output();

                match split_output {
                    Ok(_) => {
                        println!("File successfully split into parts.");
                        fs::remove_file(file_path).unwrap_or_else(|e| {
                            eprintln!("Error clearing large file during split: {e}");
                            std::process::exit(-1)
                        });
                    }
                    Err(e) => {
                        eprintln!("Error during file splitting: {e}");
                        std::process::exit(-1)
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to write blitzar_handle to file: {e}");
            std::process::exit(-1)
        }
    }
}

fn write_verifier_setup(setup: &VerifierSetup, file_path: &str) -> std::io::Result<()> {
    setup.save_to_file(Path::new(file_path))
}

// Get a spinner so we have haptic feedback during param generation
fn spinner(message: String) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message(message);
    spinner
}
