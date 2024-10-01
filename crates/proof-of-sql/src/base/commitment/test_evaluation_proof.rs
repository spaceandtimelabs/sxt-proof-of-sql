use super::{naive_commitment::NaiveCommitment, CommitmentEvaluationProof};
use crate::base::{
    polynomial::compute_evaluation_vector, proof::Transcript, scalar::test_scalar::TestScalar,
};

/// This should only be used for the purpose of unit testing.
pub struct TestEvaluationProof;

/// This should only be used for the purpose of unit testing.
/// For now it is only being created for the purpose of implementing
/// CommitmentEvaluationProof for TestEvaluationProof.
pub struct TestErrorType;

impl CommitmentEvaluationProof for TestEvaluationProof {
    type Scalar = TestScalar;

    type Commitment = NaiveCommitment;

    type Error = TestErrorType;

    type ProverPublicSetup<'a> = ();

    type VerifierPublicSetup<'a> = ();

    fn new(
        _transcript: &mut impl Transcript,
        _a: &[Self::Scalar],
        _b_point: &[Self::Scalar],
        _generators_offset: u64,
        _setup: &Self::ProverPublicSetup<'_>,
    ) -> Self {
        Self
    }

    fn verify_batched_proof(
        &self,
        _transcript: &mut impl Transcript,
        commit_batch: &[Self::Commitment],
        batching_factors: &[Self::Scalar],
        product: &Self::Scalar,
        b_point: &[Self::Scalar],
        _generators_offset: u64,
        _table_length: usize,
        _setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        let mut v = vec![TestScalar::default(); 1 << b_point.len()];
        compute_evaluation_vector(&mut v, b_point);
        (batching_factors.len() == commit_batch.len()
            && commit_batch.iter().all(|c| c.0.len() <= 1 << b_point.len())
            && batching_factors
                .iter()
                .zip(commit_batch)
                .map(|(f, c)| v.iter().zip(&c.0).map(|(a, b)| *a * *b).sum::<TestScalar>() * *f)
                .sum::<TestScalar>()
                == *product)
            .then_some(())
            .ok_or(TestErrorType)
    }
}
