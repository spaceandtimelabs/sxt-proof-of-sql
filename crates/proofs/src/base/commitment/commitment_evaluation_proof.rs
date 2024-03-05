use super::{Commitment, VecCommitmentExt};
use crate::base::{
    scalar::{MontScalar, Scalar},
    slice_ops,
};
use blitzar::proof::{InnerProductProof, ProofError};
use curve25519_dalek::{ristretto::CompressedRistretto, RistrettoPoint};
use merlin::Transcript;
use serde::{Deserialize, Serialize};

/// A trait for using commitment schemes generically. Specifically, this trait is for the evaluation proof of a commitment scheme.
pub trait CommitmentEvaluationProof {
    /// The associated scalar that the commitment is for.
    type Scalar: Scalar + Serialize + for<'a> Deserialize<'a>;
    /// The associated commitment type.
    type Commitment: Commitment<Scalar = Self::Scalar>;
    /// A collection of commitments. Most commonly this is a `Vec`.
    type VecCommitment: VecCommitmentExt<
            DecompressedCommitment = Self::Commitment,
            CommitmentPublicSetup = Self::ProverPublicSetup,
        > + Serialize
        + Clone
        + for<'a> Deserialize<'a>;
    /// The error type for the proof.
    type Error;
    /// The public setup parameters required by the prover.
    /// This is simply precomputed data that is required by the prover to create a proof.
    type ProverPublicSetup;
    /// The public setup parameters required by the verifier.
    /// This is simply precomputed data that is required by the verifier to verify a proof.
    type VerifierPublicSetup;
    /// Create a new proof.
    ///
    /// Note: b_point must have length `nu`, where `2^nu` is at least the length of `a`.
    /// `b_point` are the values for the variables that are being evaluated.
    /// The resulting evaluation is the the inner product of `a` and `b`, where `b` is the expanded vector form of `b_point`.
    fn new(
        transcript: &mut Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        setup: &Self::ProverPublicSetup,
    ) -> Self;
    /// Verify a proof.
    ///
    /// Note: b_point must have length `nu`, where `2^nu` is at least the length of `a`.
    /// `b_point` are the values for the variables that are being evaluated.
    /// The resulting evaluation is the the inner product of `a` and `b`, where `b` is the expanded vector form of `b_point`.
    #[allow(clippy::too_many_arguments)]
    fn verify_proof(
        &self,
        transcript: &mut Transcript,
        a_commit: &Self::Commitment,
        product: &Self::Scalar,
        b_point: &[Self::Scalar],
        generators_offset: u64,
        table_length: usize,
        setup: &Self::VerifierPublicSetup,
    ) -> Result<(), Self::Error>;
}

impl CommitmentEvaluationProof for InnerProductProof {
    type Scalar = MontScalar<ark_curve25519::FrConfig>;
    type Commitment = RistrettoPoint;
    type VecCommitment = Vec<CompressedRistretto>;
    type Error = ProofError;
    type ProverPublicSetup = ();
    type VerifierPublicSetup = ();
    fn new(
        transcript: &mut Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _setup: &Self::ProverPublicSetup,
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
    fn verify_proof(
        &self,
        transcript: &mut Transcript,
        a_commit: &Self::Commitment,
        product: &Self::Scalar,
        b_point: &[Self::Scalar],
        generators_offset: u64,
        table_length: usize,
        _setup: &Self::VerifierPublicSetup,
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
            a_commit,
            &product.into(),
            &slice_ops::slice_cast(b),
            generators_offset,
        )
    }
}
