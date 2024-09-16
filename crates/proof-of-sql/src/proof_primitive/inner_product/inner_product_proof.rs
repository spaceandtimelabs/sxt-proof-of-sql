use crate::base::commitment::CommitmentEvaluationProof;
#[cfg(feature = "blitzar")]
use crate::base::{
    scalar::{MontScalar, Scalar},
    slice_ops,
};
#[cfg(feature = "blitzar")]
pub use blitzar::proof::{InnerProductProof, ProofError};
#[cfg(feature = "blitzar")]
use curve25519_dalek::RistrettoPoint;
use merlin::Transcript;

#[cfg(feature = "blitzar")]
impl CommitmentEvaluationProof for InnerProductProof {
    type Scalar = MontScalar<ark_curve25519::FrConfig>;
    type Commitment = RistrettoPoint;
    type Error = ProofError;
    type ProverPublicSetup<'a> = ();
    type VerifierPublicSetup<'a> = ();
    fn new(
        transcript: &mut Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _setup: &Self::ProverPublicSetup<'_>,
    ) -> Self {
        assert!(!a.is_empty());
        let b = &mut vec![Default::default(); a.len()];
        if b_point.is_empty() {
            assert_eq!(b.len(), 1);
            b[0] = Self::Scalar::ONE;
        } else {
            crate::base::polynomial::compute_evaluation_vector(b, b_point);
        }
        Self::create(
            transcript,
            &slice_ops::slice_cast(a),
            &slice_ops::slice_cast(b),
            generators_offset,
        )
    }

    fn verify_batched_proof(
        &self,
        transcript: &mut Transcript,
        commit_batch: &[Self::Commitment],
        batching_factors: &[Self::Scalar],
        product: &Self::Scalar,
        b_point: &[Self::Scalar],
        generators_offset: u64,
        table_length: usize,
        _setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        assert!(table_length > 0);
        let b = &mut vec![Default::default(); table_length];
        if b_point.is_empty() {
            assert_eq!(b.len(), 1);
            b[0] = Self::Scalar::ONE;
        } else {
            crate::base::polynomial::compute_evaluation_vector(b, b_point);
        }
        self.verify(
            transcript,
            &commit_batch
                .iter()
                .zip(batching_factors.iter())
                .map(|(c, m)| *m * c)
                .fold(Default::default(), |mut a, c| {
                    a += c;
                    a
                }),
            &product.into(),
            &slice_ops::slice_cast(b),
            generators_offset,
        )
    }
}
