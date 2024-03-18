use super::{test_rng, DoryEvaluationProof, DoryProverPublicSetup};
use crate::base::commitment::commitment_evaluation_proof_test::*;

#[test]
fn test_simple_ipa() {
    let prover_setup = DoryProverPublicSetup::rand(4, 4, &mut test_rng());
    let verifier_setup = (&prover_setup).into();
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(&prover_setup, &verifier_setup);
    let prover_setup = DoryProverPublicSetup::rand(4, 3, &mut test_rng());
    let verifier_setup = (&prover_setup).into();
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(&prover_setup, &verifier_setup);
    let prover_setup = DoryProverPublicSetup::rand(6, 2, &mut test_rng());
    let verifier_setup = (&prover_setup).into();
    test_simple_commitment_evaluation_proof::<DoryEvaluationProof>(&prover_setup, &verifier_setup);
}

#[test]
fn test_random_ipa_with_length_1() {
    let prover_setup = DoryProverPublicSetup::rand(4, 4, &mut test_rng());
    let verifier_setup = (&prover_setup).into();
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &prover_setup,
        &verifier_setup,
    );
    let prover_setup = DoryProverPublicSetup::rand(4, 3, &mut test_rng());
    let verifier_setup = (&prover_setup).into();
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &prover_setup,
        &verifier_setup,
    );
    let prover_setup = DoryProverPublicSetup::rand(6, 2, &mut test_rng());
    let verifier_setup = (&prover_setup).into();
    test_commitment_evaluation_proof_with_length_1::<DoryEvaluationProof>(
        &prover_setup,
        &verifier_setup,
    );
}

#[test]
fn test_random_ipa_with_various_lengths() {
    let lengths = [128, 100, 64, 50, 32, 20, 16, 10, 8, 5, 4, 3, 2];
    let setup_params = [(4, 4), (4, 3), (6, 2)];
    for setup_p in setup_params {
        let prover_setup = DoryProverPublicSetup::rand(setup_p.0, setup_p.1, &mut test_rng());
        let verifier_setup = (&prover_setup).into();
        for length in lengths {
            test_random_commitment_evaluation_proof::<DoryEvaluationProof>(
                length,
                0,
                &prover_setup,
                &verifier_setup,
            );
        }
    }
}
