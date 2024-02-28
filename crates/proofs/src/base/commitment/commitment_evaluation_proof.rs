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
    type VecCommitment: VecCommitmentExt<DecompressedCommitment = Self::Commitment>
        + Serialize
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
    fn new(
        transcript: &mut Transcript,
        a: &[Self::Scalar],
        b: &[Self::Scalar],
        generators_offset: u64,
        setup: &Self::ProverPublicSetup,
    ) -> Self;
    /// Verify a proof.
    fn verify_proof(
        &self,
        transcript: &mut Transcript,
        a_commit: &Self::Commitment,
        product: &Self::Scalar,
        b: &[Self::Scalar],
        generators_offset: u64,
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
        b: &[Self::Scalar],
        generators_offset: u64,
        _setup: &Self::ProverPublicSetup,
    ) -> Self {
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
        b: &[Self::Scalar],
        generators_offset: u64,
        _setup: &Self::VerifierPublicSetup,
    ) -> Result<(), Self::Error> {
        self.verify(
            transcript,
            a_commit,
            &product.into(),
            &slice_ops::slice_cast(b),
            generators_offset,
        )
    }
}
