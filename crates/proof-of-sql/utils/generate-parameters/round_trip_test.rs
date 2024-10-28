use proof_of_sql::proof_primitive::dory::{ProverSetup, PublicParameters, VerifierSetup};
use std::{path::Path, process::Command};
use tempfile::tempdir;

#[test]
fn we_can_generate_save_and_load_public_setups() {
    // Create a temporary directory for the test
    let temp_dir = tempdir().expect("Failed to create a temporary directory");
    let temp_path = temp_dir.path().to_str().unwrap();

    // Run the binary with nu = 4, mode = "pv", and target as the temp directory
    let output = Command::new("cargo")
        .arg("run")
        .arg("--release")
        .arg("--")
        .arg("--nu")
        .arg("4")
        .arg("--mode")
        .arg("pv")
        .arg("--target")
        .arg(temp_path)
        .output()
        .expect("Failed to execute command");

    // Check the output to make sure the process ran successfully
    assert!(
        output.status.success(),
        "Process failed to run: {output:?}"
    );

    // Check that both Prover and Verifier files exist in the temp directory
    let blitzar_handle_path = format!("{temp_path}/blitzar_handle_nu_4.bin");
    let verifier_setup_path = format!("{temp_path}/verifier_setup_nu_4.bin");
    let public_parameters_path = format!("{temp_path}/public_parameters_nu_4.bin");

    assert!(
        Path::new(&blitzar_handle_path).exists(),
        "Prover setup file is missing"
    );
    assert!(
        Path::new(&verifier_setup_path).exists(),
        "Verifier setup file is missing"
    );
    assert!(
        Path::new(&public_parameters_path).exists(),
        "Public parameters file is missing"
    );

    // Load the ProverSetup and VerifierSetup from their files
    let handle = blitzar::compute::MsmHandle::new_from_file(&blitzar_handle_path);
    let params = PublicParameters::load_from_file(Path::new(&public_parameters_path)).unwrap();

    let _prover_setup = ProverSetup::from_public_parameters_and_blitzar_handle(&params, handle);
    let _verifier_setup = VerifierSetup::load_from_file(Path::new(&verifier_setup_path))
        .expect("Failed to load VerifierSetup");
}
