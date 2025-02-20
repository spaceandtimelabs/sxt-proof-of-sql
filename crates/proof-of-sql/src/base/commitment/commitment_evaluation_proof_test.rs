use super::CommitmentEvaluationProof;
use crate::base::{
    commitment::{
        naive_evaluation_proof::NaiveEvaluationProof, vec_commitment_ext::VecCommitmentExt,
    },
    database::Column,
};
use ark_std::UniformRand;
use merlin::Transcript;
use num_traits::{One, Zero};

pub fn test_simple_commitment_evaluation_proof<CP: CommitmentEvaluationProof>(
    prover_setup: &CP::ProverPublicSetup<'_>,
    verifier_setup: &CP::VerifierPublicSetup<'_>,
) {
    let mut transcript = Transcript::new(b"evaluation_proof");
    let proof = CP::new(
        &mut transcript,
        &[CP::Scalar::one(), CP::Scalar::one() + CP::Scalar::one()],
        &[CP::Scalar::zero()],
        0,
        prover_setup,
    );

    let commits = Vec::from_columns_with_offset(
        [Column::Scalar(&[
            CP::Scalar::one(),
            CP::Scalar::one() + CP::Scalar::one(),
        ])],
        0,
        prover_setup,
    );

    let mut transcript = Transcript::new(b"evaluation_proof");
    let r = proof.verify_proof(
        &mut transcript,
        &commits[0],
        &CP::Scalar::one(),
        &[CP::Scalar::zero()],
        0,
        2,
        verifier_setup,
    );
    assert!(r.is_ok());
}

pub fn test_commitment_evaluation_proof_with_length_1<CP: CommitmentEvaluationProof>(
    prover_setup: &CP::ProverPublicSetup<'_>,
    verifier_setup: &CP::VerifierPublicSetup<'_>,
) {
    let mut rng = ark_std::test_rng();
    let r = CP::Scalar::rand(&mut rng);
    let mut transcript = Transcript::new(b"evaluation_proof");
    let proof = CP::new(&mut transcript, &[r], &[], 0, prover_setup);

    let commits = Vec::from_columns_with_offset([Column::Scalar(&[r])], 0, prover_setup);

    let mut transcript = Transcript::new(b"evaluation_proof");
    let r = proof.verify_proof(&mut transcript, &commits[0], &r, &[], 0, 1, verifier_setup);
    assert!(r.is_ok());
}

pub fn test_random_commitment_evaluation_proof<CP: CommitmentEvaluationProof>(
    table_length: usize,
    offset: usize,
    prover_setup: &CP::ProverPublicSetup<'_>,
    verifier_setup: &CP::VerifierPublicSetup<'_>,
) {
    let nu = table_length.next_power_of_two().trailing_zeros() as usize;
    assert!(table_length <= 1 << nu);
    assert!(1 << (nu - 1) < table_length);

    let mut rng = ark_std::test_rng();
    let a = core::iter::repeat_with(|| CP::Scalar::rand(&mut rng))
        .take(table_length)
        .collect::<Vec<_>>();
    let b_point = core::iter::repeat_with(|| CP::Scalar::rand(&mut rng))
        .take(nu)
        .collect::<Vec<_>>();

    let mut transcript = Transcript::new(b"evaluation_proof");
    let proof = CP::new(&mut transcript, &a, &b_point, offset as u64, prover_setup);

    let commits = Vec::from_columns_with_offset([Column::Scalar(&a)], offset, prover_setup);

    let mut b = vec![CP::Scalar::zero(); a.len()];
    crate::base::polynomial::compute_evaluation_vector(&mut b, &b_point);
    let product: CP::Scalar = a.iter().zip(b.iter()).map(|(a, b)| *a * *b).sum();

    let mut transcript = Transcript::new(b"evaluation_proof");
    let r = proof.verify_proof(
        &mut transcript,
        &commits[0],
        &product,
        &b_point,
        offset as u64,
        table_length,
        verifier_setup,
    );
    assert!(r.is_ok(), "verification improperly failed");

    // Invalid Transcript
    let mut transcript = Transcript::new(b"evaluation_proof_wrong");
    let r = proof.verify_proof(
        &mut transcript,
        &commits[0],
        &product,
        &b_point,
        offset as u64,
        table_length,
        verifier_setup,
    );
    assert!(r.is_err(), "verification improperly succeeded");

    // Invalid Product
    let mut transcript = Transcript::new(b"evaluation_proof");
    let r = proof.verify_proof(
        &mut transcript,
        &commits[0],
        &(product + CP::Scalar::one()),
        &b_point,
        offset as u64,
        table_length,
        verifier_setup,
    );
    assert!(r.is_err(), "verification improperly succeeded");

    // Invalid offset
    let wrong_offset = u64::from(offset == 0);
    let mut transcript = Transcript::new(b"evaluation_proof");
    let r = proof.verify_proof(
        &mut transcript,
        &commits[0],
        &product,
        &b_point,
        wrong_offset,
        table_length,
        verifier_setup,
    );
    assert!(r.is_err(), "verification improperly succeeded");
}

#[test]
fn test_simple_ipa() {
    test_simple_commitment_evaluation_proof::<NaiveEvaluationProof>(&(), &());
}

#[test]
fn test_random_ipa_with_length_1() {
    test_commitment_evaluation_proof_with_length_1::<NaiveEvaluationProof>(&(), &());
}
