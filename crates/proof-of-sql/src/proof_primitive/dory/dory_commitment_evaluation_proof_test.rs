use super::{test_rng, DoryEvaluationProof, DoryProverPublicSetup, DoryScalar};
use crate::base::commitment::{commitment_evaluation_proof_test::*, CommitmentEvaluationProof};
use ark_std::UniformRand;
use merlin::Transcript;

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

#[test]
fn we_can_serialize_and_deserialize_dory_evaluation_proofs() {
    let mut rng = ark_std::test_rng();
    let prover_setup = DoryProverPublicSetup::rand(4, 3, &mut rng);
    let a = core::iter::repeat_with(|| DoryScalar::rand(&mut rng))
        .take(30)
        .collect::<Vec<_>>();
    let b_point = core::iter::repeat_with(|| DoryScalar::rand(&mut rng))
        .take(5)
        .collect::<Vec<_>>();
    let mut transcript = Transcript::new(b"evaluation_proof");
    let proof = DoryEvaluationProof::new(&mut transcript, &a, &b_point, 0, &prover_setup);
    let encoded = postcard::to_allocvec(&proof).unwrap();
    let decoded: DoryEvaluationProof = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(decoded, proof);
}
