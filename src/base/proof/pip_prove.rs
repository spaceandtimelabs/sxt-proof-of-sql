use crate::base::proof::{Commit, ProofError, Transcript};

/// Provides construction for Partial Interactive Protocol (PIP) proofs.
///
/// Generic over the input and output of the proof.
/// However, for best interoperability with other proofs, you'll most likely want to use a tuple as
/// the input.
///
/// Note that this trait is not responsible for producing the output of the proof itself.
/// It is only meant to produce a proof that the output is correct.
///
/// Requires implementing [PipVerify].
pub trait PipProve<I, O>: PipVerify<I::Commitment, O::Commitment>
where
    I: Commit,
    O: Commit,
{
    /// Construct the proof.
    fn prove(
        //The merlin transcript for the prover
        transcript: &mut Transcript,
        //The inputs to the PIP
        input: I,
        //The output of the PIP. Note: these are not computed by the PIP itself. The PIP simply produces a proof that these are correct.
        output: O,
        //The commitments of the inputs to the PIP. This is redundant since it can be computed from input_columns, but they will already have been computed
        input_commitment: I::Commitment,
    ) -> Self;
}

/// Provides verification for Partial Interactive Protocol (PIP) proofs.
///
/// Generic over the input commitments and output commitments of the proof.
///
/// This trait is meant to be used along with [PipProve].
/// These traits could have been combined, but separating them allows for more a more ergonomic API
/// when dealing with generic data types in the input and output of the proof.
/// This is because the methods provided by this trait don't require knowledge of the original
/// input/output types, just their associated commitment types.
pub trait PipVerify<IC, OC> {
    /// Verify the proof.
    fn verify(
        &self,
        //The merlin transcript for the verifier
        transcript: &mut Transcript,
        //The commitments of the inputs to the PIP. Typically, these are known by the verifier.
        input_commitments: IC,
    ) -> Result<(), ProofError>;

    /// The commitments of the outputs to the PIP.
    /// These should be included in the proof itself.
    fn get_output_commitments(&self) -> OC;
}
