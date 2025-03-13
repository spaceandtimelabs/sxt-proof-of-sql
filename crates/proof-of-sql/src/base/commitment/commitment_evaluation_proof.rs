use super::Commitment;
use crate::base::{proof::Transcript, scalar::Scalar};
use serde::{Deserialize, Serialize};

/// A trait for using commitment schemes generically. Specifically, this trait is for the evaluation proof of a commitment scheme.
pub trait CommitmentEvaluationProof {
    /// The associated scalar that the commitment is for.
    type Scalar: Scalar + Serialize + for<'a> Deserialize<'a>;
    /// The associated commitment type.
    type Commitment: for<'a> Commitment<Scalar = Self::Scalar, PublicSetup<'a> = Self::ProverPublicSetup<'a>>
        + Serialize
        + for<'a> Deserialize<'a>;
    /// The error type for the proof.
    type Error;
    /// The public setup parameters required by the prover.
    /// This is simply precomputed data that is required by the prover to create a proof.
    type ProverPublicSetup<'a>: Copy;
    /// The public setup parameters required by the verifier.
    /// This is simply precomputed data that is required by the verifier to verify a proof.
    type VerifierPublicSetup<'a>: Copy;
    /// Create a new proof.
    ///
    /// Note: `b_point` must have length `nu`, where `2^nu` is at least the length of `a`.
    /// `b_point` are the values for the variables that are being evaluated.
    /// The resulting evaluation is the inner product of `a` and `b`, where `b` is the expanded vector form of `b_point`.
    fn new(
        transcript: &mut impl Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        setup: &Self::ProverPublicSetup<'_>,
    ) -> Self;
    /// Verify a proof.
    ///
    /// Note: `b_point` must have length `nu`, where `2^nu` is at least the length of `a`.
    /// `b_point` are the values for the variables that are being evaluated.
    /// The resulting evaluation is the inner product of `a` and `b`, where `b` is the expanded vector form of `b_point`.
    #[expect(clippy::too_many_arguments)]
    fn verify_proof(
        &self,
        transcript: &mut impl Transcript,
        a_commit: &Self::Commitment,
        product: &Self::Scalar,
        b_point: &[Self::Scalar],
        generators_offset: u64,
        table_length: usize,
        setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        self.verify_batched_proof(
            transcript,
            core::slice::from_ref(a_commit),
            &[Self::Scalar::ONE],
            core::slice::from_ref(product),
            b_point,
            generators_offset,
            table_length,
            setup,
        )
    }
    /// Verify a batch proof. This can be more efficient than verifying individual proofs for some schemes.
    #[expect(clippy::too_many_arguments)]
    fn verify_batched_proof(
        &self,
        transcript: &mut impl Transcript,
        commit_batch: &[Self::Commitment],
        batching_factors: &[Self::Scalar],
        evaluations: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        table_length: usize,
        setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error>;
}
