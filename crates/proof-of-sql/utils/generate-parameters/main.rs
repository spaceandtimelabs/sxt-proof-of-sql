//! doc comment

#[cfg(test)]
mod round_trip_test;

use ark_std::rand::SeedableRng;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use proof_of_sql::proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup};
use rand_chacha::ChaCha20Rng;
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File, OpenOptions},
    io,
    io::Write,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

/// Transparent public randomness
const SEED: &str = "SpaceAndTime";

// Command-line argument parser structure
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The value for `nu` (number of public parameters)
    #[arg(short, long, default_value_t = 8)]
    nu: usize,

    /// Mode for generating parameters: "p" for Prover, "v" for Verifier
    #[arg(short, long, default_value = "pv")]
    mode: String,

    /// The initial randomness for the transparent setup
    #[arg(short, long, default_value = SEED)]
    seed: String,

    /// The directory to store generated files and archives
    #[arg(short, long, default_value = "./output")]
    target: String,
}

fn main() {
    // Set the BLITZAR_PARTITION_WINDOW_WIDTH environment variable
    env::set_var("BLITZAR_PARTITION_WINDOW_WIDTH", "14");

    // Confirm that it was set by reading it back
    match env::var("BLITZAR_PARTITION_WINDOW_WIDTH") {
        Ok(value) => println!(
            "Environment variable BLITZAR_PARTITION_WINDOW_WIDTH set to {value}"
        ),
        Err(e) => println!("Failed to set environment variable: {e}"),
    }

    // Parse command-line arguments
    let args = Args::parse();

    // Ensure the target directory exists
    fs::create_dir_all(&args.target).expect("Failed to create target directory");

    // Clear out the digests.txt file if it already exists
    let digests_path = format!("{}/digests_nu_{}.txt", args.target, args.nu);
    if Path::new(&digests_path).exists() {
        fs::write(&digests_path, "").expect("Failed to clear digests.txt file");
    }

    // Convert the seed string to bytes and create a seeded RNG
    let seed_bytes = args
        .seed
        .bytes()
        .chain(std::iter::repeat(0u8))
        .take(32)
        .collect::<Vec<_>>()
        .try_into()
        .expect("collection is guaranteed to contain 32 elements");
    let mut rng = ChaCha20Rng::from_seed(seed_bytes);

    let spinner = spinner(format!(
        "Generating a random public setup with seed {SEED:?} please wait..."
    ));

    // Use the `nu` value from the command-line argument
    let public_parameters = PublicParameters::rand(args.nu, &mut rng);

    spinner.finish_with_message("Public parameter setup complete");

    match args.mode.as_str() {
        "pv" => {
            println!("Generating parameters for Prover...");
            generate_prover_setup(&public_parameters, args.nu, &args.target);
            println!("Generating parameters for Verifier...");
            generate_verifier_setup(&public_parameters, args.nu, &args.target);
        }
        "p" => {
            println!("Generating parameters for Prover...");
            generate_prover_setup(&public_parameters, args.nu, &args.target);
        }
        "v" => {
            println!("Generating parameters for Verifier...");
            generate_verifier_setup(&public_parameters, args.nu, &args.target);
        }
        _ => {
            println!("Invalid mode! Please choose either 'p' for Prover or 'v' for Verifier.");
        }
    }
}

// Generates and writes the ProverSetup from initial public parameters
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
    let result = public_parameters.save_to_file(Path::new(&public_parameters_path));
    let file_path = format!("{target}/blitzar_handle_nu_{nu}.bin");

    match result {
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
        Err(_) => println!("Failed to save parameters, aborting."),
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
    let result = write_verifier_setup(setup, &file_path);

    match result {
        Ok(()) => {
            println!("Verifier setup and parameters saved successfully.");

            // Compute and save SHA-256
            let mut digests = Vec::new();
            if let Some(digest) = compute_sha256(&file_path) {
                digests.push((file_path.clone(), digest));
            }
            save_digests(&digests, target, nu); // Save digests to digests.txt
        }
        Err(_) => println!("Failed to save setup, aborting."),
    }
}

// Function to compute SHA-256 hash of a file
fn compute_sha256(file_path: &str) -> Option<String> {
    let mut file = File::open(file_path).ok()?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).ok()?;
    Some(format!("{:x}", hasher.finalize()))
}

// Function to save digests to digests.txt
fn save_digests(digests: &[(String, String)], target: &str, nu: usize) {
    let digests_path = format!("{target}/digests_nu_{nu}.txt");

    // Open file in append mode, creating it if it doesn't exist
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&digests_path)
        .expect("Unable to open or create digests.txt");

    for (file_path, digest) in digests {
        writeln!(file, "{digest}  {file_path}").expect("Unable to write digest to file");
    }
    println!("Digests saved to {digests_path}");
}

fn write_prover_blitzar_handle(setup: ProverSetup<'_>, file_path: &str) {
    let blitzar_handle = setup.blitzar_handle();
    blitzar_handle.write(file_path);

    // Check the file size to see if it exceeds 2 GB
    let metadata = fs::metadata(file_path).expect("Unable to read file metadata");
    let file_size = metadata.len();

    if file_size > 2 * 1024 * 1024 * 1024 {
        // 2 GB in bytes
        println!("Handle size exceeds 2 GB, splitting into parts...");

        // Run `split` command to divide the file into 1.8 GB parts
        let split_output = Command::new("split")
            .arg("-b")
            .arg("1800M")
            .arg(file_path)
            .arg(format!("{file_path}.part."))
            .output()
            .expect("Failed to execute split command");

        if split_output.status.success() {
            println!("File successfully split into parts.");

            // Remove the original file after splitting
            fs::remove_file(file_path).expect("Failed to remove original blitzar handle file");
        } else {
            eprintln!(
                "Error during file splitting: {}",
                String::from_utf8_lossy(&split_output.stderr)
            );
        }
    }
}

fn write_verifier_setup(setup: VerifierSetup, file_path: &str) -> std::io::Result<()> {
    setup.save_to_file(Path::new(file_path))
}

// Get a spinner so we have haptic feedback during param generation
fn spinner(message: String) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message(message);
    spinner
}
