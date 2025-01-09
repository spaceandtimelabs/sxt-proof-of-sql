use super::{
    ristretto_hyrax_configuration::RistrettoHyraxConfiguration,
    ristretto_hyrax_public_setup::RistrettoHyraxPublicSetup,
};
use crate::{
    base::commitment::commitment_evaluation_proof_test::{
        test_commitment_evaluation_proof_with_length_1, test_random_commitment_evaluation_proof,
        test_simple_commitment_evaluation_proof,
    },
    proof_primitive::hyrax::base::hyrax_commitment_evaluation_proof::HyraxCommitmentEvaluationProof,
};
use core::iter;
use curve25519_dalek::RistrettoPoint;
use rand::SeedableRng;

pub type RistrettoHyraxCommitmentEvaluationProof =
    HyraxCommitmentEvaluationProof<RistrettoHyraxConfiguration>;

#[test]
fn we_can_test_simple_test_hyrax_commitment_evaluation_proof() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let generators = iter::repeat_with(|| RistrettoPoint::random(&mut rng))
        .take(10)
        .collect::<Vec<_>>();
    let public_setup = RistrettoHyraxPublicSetup {
        generators: &generators,
    };
    test_simple_commitment_evaluation_proof::<RistrettoHyraxCommitmentEvaluationProof>(
        &public_setup,
        &public_setup,
    );
}

#[test]
fn we_can_test_simple_test_hyrax_commitment_evaluation_proof_with_length_1() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let generators = iter::repeat_with(|| RistrettoPoint::random(&mut rng))
        .take(10)
        .collect::<Vec<_>>();
    let public_setup = RistrettoHyraxPublicSetup {
        generators: &generators,
    };
    test_commitment_evaluation_proof_with_length_1::<RistrettoHyraxCommitmentEvaluationProof>(
        &public_setup,
        &public_setup,
    );
}

#[test]
fn we_can_test_random_test_hyrax_commitment_evaluation_proofs() {
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let generators = iter::repeat_with(|| RistrettoPoint::random(&mut rng))
        .take(1000)
        .collect::<Vec<_>>();
    let public_setup = RistrettoHyraxPublicSetup {
        generators: &generators,
    };
    test_random_commitment_evaluation_proof::<RistrettoHyraxCommitmentEvaluationProof>(
        50,
        0,
        &public_setup,
        &public_setup,
    );
}
