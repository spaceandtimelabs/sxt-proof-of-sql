//! Create the powers of tau binary file.
//!
//! Power of Tau files are available at
//! `<https://github.com/privacy-scaling-explorations/perpetualpowersoftau>`
//!
//! Usage:
//!
//! cargo run --release <`ptau_file_path`> <`binary_file_path`> <`n`>

use ark_bn254::G1Affine;
use ark_serialize::CanonicalSerialize;
use nova_snark::{
    provider::{
        hyperkzg::{CommitmentEngine, CommitmentKey},
        Bn256EngineKZG,
    },
    traits::commitment::CommitmentEngineTrait,
};
use std::{
    env,
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    path::Path,
};

type E = Bn256EngineKZG;

/// Parse command-line arguments.
///
/// # Errors
///
/// This function returns an error if the arguments are invalid.
fn parse_args(args: &[String]) -> Result<(&str, &str, usize), String> {
    if args.len() < 4 {
        return Err(format!("Usage: {} <ptau_path> <binary_path> <n>", args[0]));
    }

    let ptau_path = &args[1];
    let binary_path = &args[2];
    let n: usize = args[3]
        .parse()
        .map_err(|_| "The third argument must be a valid usize.".to_string())?;

    if !Path::new(ptau_path).exists() {
        return Err(format!("Path '{ptau_path}' does not exist."));
    }

    Ok((ptau_path, binary_path, n))
}

/// Load the setup from a file.
///
/// # Panics
///
/// This function panics if the file cannot be read.
fn load_setup_from_file(ptau_path: &str, n: usize) -> CommitmentKey<E> {
    let file = OpenOptions::new().read(true).open(ptau_path).unwrap();
    let mut reader = BufReader::new(file);
    CommitmentEngine::<E>::load_setup(&mut reader, n).unwrap()
}

/// Write the commitment key to a binary file.
///
/// # Panics
///
/// This function panics if the binary file cannot be written.
fn write_commitment_key_to_binary(binary_path: &str, setup: &CommitmentKey<E>) {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(binary_path)
        .unwrap();
    let mut writer = BufWriter::new(file);

    // Write the commitment key element to the binary file
    let elements: Vec<G1Affine> = setup
        .ck()
        .iter()
        .map(blitzar::compute::convert_to_ark_bn254_g1_affine)
        .collect();
    elements.serialize_compressed(&mut writer).unwrap();
}

fn create_binary_file(args: &[String]) {
    let (ptau_path, binary_path, n) = match parse_args(args) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    };

    // Read the setup using Microsoft's Nova crate
    let setup = load_setup_from_file(ptau_path, n);

    // Create the binary file
    write_commitment_key_to_binary(binary_path, &setup);
}

/// # Panics
///
/// This function panics if the binary file cannot be written.
fn main() {
    let args: Vec<String> = env::args().collect();
    create_binary_file(args.as_slice());
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_serialize::CanonicalDeserialize;
    use std::fs;

    /// # Panics
    ///
    /// This function panics if the file cannot be generated.
    #[test]
    fn we_can_create_a_binary_file() {
        let n = 4;
        let ptau_path = "/tmp/create_binary_file_test.ptau";
        let binary_path = "/tmp/create_binary_file_test.bin";
        let ck: CommitmentKey<E> = CommitmentEngine::setup(b"test", n);

        // Generate the ptau file
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(ptau_path)
            .unwrap();
        let mut writer = BufWriter::new(file);
        ck.save_to(&mut writer).unwrap();

        // Create the binary file
        create_binary_file(&[
            "program".to_string(),
            ptau_path.to_string(),
            binary_path.to_string(),
            n.to_string(),
        ]);

        // Verify the binary file
        let file = OpenOptions::new().read(true).open(binary_path).unwrap();
        let mut reader = BufReader::new(file);
        let elements: Vec<G1Affine> = Vec::deserialize_compressed(&mut reader).unwrap();

        assert_eq!(ck.ck().len(), elements.len());
        for (ck_elem, setup_elem) in ck.ck().iter().zip(elements.iter()) {
            assert_eq!(
                *ck_elem,
                blitzar::compute::convert_to_halo2_bn256_g1_affine(setup_elem)
            );
        }

        // Clean up
        fs::remove_file(ptau_path).unwrap();
        fs::remove_file(binary_path).unwrap();
    }

    /// # Panics
    ///
    /// This function panics if the file cannot be generated.
    #[test]
    fn we_can_parse_args() {
        let n = 4;
        let file_name = "/tmp/args_test.ptau";
        let ck: CommitmentKey<E> = CommitmentEngine::setup(b"test", n);

        // Generate the file
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(file_name)
            .unwrap();
        let mut writer = BufWriter::new(file);

        // Use Nova's save_to implementation to write the file
        ck.save_to(&mut writer).unwrap();

        let args = vec![
            "program".to_string(),
            "/tmp/args_test.ptau".to_string(),
            "/tmp/args_test.bin".to_string(),
            "4".to_string(),
        ];
        let (ptau_path, binary_path, n) = parse_args(&args).unwrap();
        assert_eq!(ptau_path, "/tmp/args_test.ptau");
        assert_eq!(binary_path, "/tmp/args_test.bin");
        assert_eq!(n, 4);

        // Clean up
        fs::remove_file(file_name).unwrap();
    }

    /// # Panics
    ///
    /// This function panics if the file cannot be generated.
    #[test]
    fn we_can_parse_args_with_missing_args() {
        let n = 4;
        let file_name = "/tmp/missing_args_test.ptau";
        let ck: CommitmentKey<E> = CommitmentEngine::setup(b"test", n);

        // Generate the file
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(file_name)
            .unwrap();
        let mut writer = BufWriter::new(file);

        // Use Nova's save_to implementation to write the file
        ck.save_to(&mut writer).unwrap();

        let args = vec!["program".to_string()];
        let result = parse_args(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Usage: program <ptau_path> <binary_path> <n>"
        );

        // Clean up
        fs::remove_file(file_name).unwrap();
    }

    /// # Panics
    ///
    /// This function panics if the file cannot be generated.
    #[test]
    fn we_can_parse_args_with_invalid_n() {
        let n = 4;
        let file_name = "/tmp/invalid_n_args_test.ptau";
        let ck: CommitmentKey<E> = CommitmentEngine::setup(b"test", n);

        // Generate the file
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(file_name)
            .unwrap();
        let mut writer = BufWriter::new(file);

        // Use Nova's save_to implementation to write the file
        ck.save_to(&mut writer).unwrap();

        let args = vec![
            "program".to_string(),
            "/tmp/invalid_n_args_test.ptau".to_string(),
            "/tmp/invalid_n_args_test.bin".to_string(),
            "invalid".to_string(),
        ];
        let result = parse_args(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "The third argument must be a valid usize."
        );

        // Clean up
        fs::remove_file(file_name).unwrap();
    }

    /// # Panics
    ///
    /// This function panics if the result is not an error.
    #[test]
    fn we_can_parse_args_with_an_invalid_path() {
        let args = vec![
            "program".to_string(),
            "/tmp/invalid.ptau".to_string(),
            "/tmp/test.bin".to_string(),
            "4".to_string(),
        ];
        let result = parse_args(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Path '/tmp/invalid.ptau' does not exist."
        );
    }

    /// # Panics
    ///
    /// This function panics if the file cannot be read.
    #[test]
    fn we_can_load_ptau_file() {
        let n = 4;
        let file_name = "/tmp/load_test.ptau";
        let ck: CommitmentKey<E> = CommitmentEngine::setup(b"test", n);

        // Generate the file
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(file_name)
            .unwrap();
        let mut writer = BufWriter::new(file);

        // Use Nova's save_to implementation to write the file
        ck.save_to(&mut writer).unwrap();

        // Load the powers of tau from the file
        let setup = load_setup_from_file(file_name, n);

        assert_eq!(ck.ck().len(), setup.ck().len());
        assert_eq!(ck.h(), setup.h());
        assert_eq!(ck.tau_H(), setup.tau_H());
        for (ck_elem, setup_elem) in ck.ck().iter().zip(setup.ck().iter()) {
            assert_eq!(ck_elem, setup_elem);
        }

        // Clean up
        fs::remove_file(file_name).unwrap();
    }

    /// # Panics
    ///
    /// Panics if the file cannot be read.
    #[test]
    fn we_can_write_commitment_key_to_binary() {
        let n = 4;
        let ptau_path = "/tmp/write_binary_test.ptau";
        let binary_path = "/tmp/write_binary_test.bin";
        let ck: CommitmentKey<E> = CommitmentEngine::setup(b"test", n);

        // Generate the ptau file
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(ptau_path)
            .unwrap();
        let mut writer = BufWriter::new(file);
        ck.save_to(&mut writer).unwrap();

        // Load the setup from the ptau file
        let setup = load_setup_from_file(ptau_path, n);

        // Write the commitment key to the binary file
        write_commitment_key_to_binary(binary_path, &setup);

        // Verify the binary file
        let file = OpenOptions::new().read(true).open(binary_path).unwrap();
        let mut reader = BufReader::new(file);
        let elements: Vec<G1Affine> = Vec::deserialize_compressed(&mut reader).unwrap();

        assert_eq!(ck.ck().len(), elements.len());
        for (ck_elem, setup_elem) in ck.ck().iter().zip(elements.iter()) {
            assert_eq!(
                *ck_elem,
                blitzar::compute::convert_to_halo2_bn256_g1_affine(setup_elem)
            );
        }

        // Clean up
        fs::remove_file(ptau_path).unwrap();
        fs::remove_file(binary_path).unwrap();
    }
}
