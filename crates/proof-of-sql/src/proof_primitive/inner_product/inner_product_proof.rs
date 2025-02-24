use crate::base::{
    commitment::CommitmentEvaluationProof,
    proof::Transcript,
    scalar::{MontScalar, Scalar},
    slice_ops,
};
#[cfg(feature = "blitzar")]
use blitzar::proof::{InnerProductProof, ProofError};
use curve25519_dalek::RistrettoPoint;

#[cfg(feature = "blitzar")]
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
