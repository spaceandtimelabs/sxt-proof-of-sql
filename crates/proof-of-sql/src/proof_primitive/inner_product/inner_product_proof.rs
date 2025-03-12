use crate::base::{
    commitment::CommitmentEvaluationProof,
    proof::Transcript,
    scalar::{MontScalar, Scalar},
    slice_ops,
};
use blitzar::proof::{InnerProductProof, ProofError};
use curve25519_dalek::RistrettoPoint;

impl CommitmentEvaluationProof for InnerProductProof {
    type Scalar = MontScalar<ark_curve25519::FrConfig>;
    type Commitment = RistrettoPoint;
    type Error = ProofError;
    type ProverPublicSetup<'a> = ();
    type VerifierPublicSetup<'a> = ();
    fn new(
        transcript: &mut impl Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _setup: &Self::ProverPublicSetup<'_>,
    ) -> Self {
        assert!(!a.is_empty());
        let b = &mut vec![MontScalar::default(); a.len()];
        if b_point.is_empty() {
            assert_eq!(b.len(), 1);
            b[0] = Self::Scalar::ONE;
        } else {
            crate::base::polynomial::compute_evaluation_vector(b, b_point);
        }
        // The InnerProductProof from blitzar only works with the merlin Transcript.
        // So, we wrap the call to it.
        transcript.wrap_transcript(|transcript| {
            Self::create(
                transcript,
                &slice_ops::slice_cast(a),
                &slice_ops::slice_cast(b),
                generators_offset,
            )
        })
    }

    fn verify_batched_proof(
        &self,
        transcript: &mut impl Transcript,
        commit_batch: &[Self::Commitment],
        batching_factors: &[Self::Scalar],
        evaluations: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        table_length: usize,
        _setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        assert!(table_length > 0);
        let b = &mut vec![MontScalar::default(); table_length];
        if b_point.is_empty() {
            assert_eq!(b.len(), 1);
            b[0] = Self::Scalar::ONE;
        } else {
            crate::base::polynomial::compute_evaluation_vector(b, b_point);
        }
        let product: Self::Scalar = evaluations
            .iter()
            .zip(batching_factors)
            .map(|(&e, &f)| e * f)
            .sum();
        // The InnerProductProof from blitzar only works with the merlin Transcript.
        // So, we wrap the call to it.
        transcript.wrap_transcript(|transcript| {
            self.verify(
                transcript,
                &commit_batch
                    .iter()
                    .zip(batching_factors.iter())
                    .map(|(c, m)| *m * c)
                    .fold(RistrettoPoint::default(), |mut a, c| {
                        a += c;
                        a
                    }),
                &product.into(),
                &slice_ops::slice_cast(b),
                generators_offset,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::commitment::commitment_evaluation_proof_test::{
        test_commitment_evaluation_proof_with_length_1, test_random_commitment_evaluation_proof,
        test_simple_commitment_evaluation_proof,
    };

    #[test]
    fn test_simple_ipa() {
        test_simple_commitment_evaluation_proof::<InnerProductProof>(&(), &());
    }

    #[test]
    fn test_random_ipa_with_length_1() {
        test_commitment_evaluation_proof_with_length_1::<InnerProductProof>(&(), &());
    }

    #[test]
    fn test_random_ipa_with_length_128() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(128, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(128, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(128, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(128, 64, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(128, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_100() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(100, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(100, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(100, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(100, 64, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(100, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_64() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(64, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(64, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(64, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(64, 32, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(64, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_50() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(50, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(50, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(50, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(50, 32, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(50, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_32() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(32, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(32, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(32, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(32, 16, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(32, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_20() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(20, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(20, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(20, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(20, 16, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(20, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_16() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(16, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(16, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(16, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(16, 8, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(16, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_10() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(10, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(10, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(10, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(10, 8, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(10, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_8() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(8, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(8, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(8, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(8, 4, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(8, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_5() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(5, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(5, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(5, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(5, 4, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(5, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_4() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(4, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(4, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(4, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(4, 2, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(4, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_3() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(3, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(3, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(3, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(3, 2, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(3, 200, &(), &());
    }

    #[test]
    fn test_random_ipa_with_length_2() {
        test_random_commitment_evaluation_proof::<InnerProductProof>(2, 0, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(2, 1, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(2, 10, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(2, 2, &(), &());
        test_random_commitment_evaluation_proof::<InnerProductProof>(2, 200, &(), &());
    }
}
