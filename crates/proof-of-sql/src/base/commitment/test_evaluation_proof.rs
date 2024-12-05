use super::{naive_commitment::NaiveCommitment, CommitmentEvaluationProof};
use crate::base::{
    polynomial::compute_evaluation_vector,
    proof::Transcript,
    scalar::{test_scalar::TestScalar, Scalar},
};
use core::ops::Add;

/// This should only be used for the purpose of unit testing.
pub struct TestEvaluationProof {
    a: NaiveCommitment,
    b_point: Vec<TestScalar>,
    challenge: [u8; 32],
}

/// This should only be used for the purpose of unit testing.
/// For now it is only being created for the purpose of implementing
/// [`CommitmentEvaluationProof`] for [`NaiveEvaluationProof`].
pub struct NaiveEvaluationProofError;

impl CommitmentEvaluationProof for TestEvaluationProof {
    type Scalar = TestScalar;

    type Commitment = NaiveCommitment;

    type Error = NaiveEvaluationProofError;

    type ProverPublicSetup<'a> = ();

    type VerifierPublicSetup<'a> = ();

    fn new(
        transcript: &mut impl Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _setup: &Self::ProverPublicSetup<'_>,
    ) -> Self {
        let challenge = transcript.challenge_as_le();
        let result = Self {
            a: NaiveCommitment(
                itertools::repeat_n(TestScalar::ZERO, generators_offset.try_into().unwrap())
                    .chain(a.iter().copied())
                    .collect(),
            ),
            b_point: b_point.to_vec(),
            challenge,
        };
        transcript.extend_scalars_as_be(&result.a.0);
        transcript.extend_scalars_as_be(&result.b_point);
        result
    }

    fn verify_batched_proof(
        &self,
        transcript: &mut impl Transcript,
        commit_batch: &[Self::Commitment],
        batching_factors: &[Self::Scalar],
        product: &Self::Scalar,
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _table_length: usize,
        _setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        let challenge = transcript.challenge_as_le();
        if challenge != self.challenge {
            return Err(NaiveEvaluationProofError);
        }
        if self.b_point != b_point {
            return Err(NaiveEvaluationProofError);
        }
        let folded_commits = commit_batch
            .iter()
            .zip(batching_factors)
            .map(|(c, m)| *m * c)
            .fold(NaiveCommitment(vec![]), Add::add);
        if folded_commits != self.a {
            return Err(NaiveEvaluationProofError);
        }
        let mut b_vec = vec![TestScalar::ZERO; 1 << b_point.len()];
        compute_evaluation_vector(&mut b_vec, b_point);
        let expected_product = self
            .a
            .0
            .iter()
            .skip(generators_offset.try_into().unwrap())
            .zip(b_vec)
            .map(|(&a, b)| a * b)
            .sum::<TestScalar>();
        if expected_product != *product {
            return Err(NaiveEvaluationProofError);
        }
        transcript.extend_scalars_as_be(&self.a.0);
        transcript.extend_scalars_as_be(&self.b_point);
        Ok(())
    }
}

mod tests {
    use super::TestEvaluationProof;
    use crate::base::commitment::commitment_evaluation_proof_test::{
        test_commitment_evaluation_proof_with_length_1, test_random_commitment_evaluation_proof,
        test_simple_commitment_evaluation_proof,
    };

    #[test]
    fn test_simple_ipa() {
        test_simple_commitment_evaluation_proof::<TestEvaluationProof>(&(), &());
    }

    #[test]
    fn test_random_ipa_with_length_1() {
        test_commitment_evaluation_proof_with_length_1::<TestEvaluationProof>(&(), &());
    }

    #[test]
    fn test_random_ipa_with_length_128() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(128, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(128, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(128, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(128, 64, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(128, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_100() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(100, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(100, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(100, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(100, 64, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(100, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_64() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(64, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(64, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(64, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(64, 32, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(64, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_50() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(50, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(50, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(50, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(50, 32, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(50, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_32() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(32, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(32, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(32, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(32, 16, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(32, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_20() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(20, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(20, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(20, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(20, 16, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(20, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_16() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(16, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(16, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(16, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(16, 8, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(16, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_10() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(10, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(10, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(10, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(10, 8, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(10, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_8() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(8, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(8, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(8, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(8, 4, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(8, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_5() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(5, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(5, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(5, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(5, 4, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(5, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_4() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(4, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(4, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(4, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(4, 2, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(4, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_3() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(3, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(3, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(3, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(3, 2, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(3, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_2() {
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(2, 0, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(2, 1, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(2, 10, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(2, 2, &(), &());
        test_random_commitment_evaluation_proof::<TestEvaluationProof>(2, 200, &(), &());
    }
}
