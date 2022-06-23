use curve25519_dalek::scalar::Scalar;

use crate::base::proof::{Commitment, ProofError, Transcript};

pub trait PIPProof /*: serde::ser::Serialize + serde::ser::Deserialize*/ {
    fn create(
        //The merlin transcript for the prover
        transcript: &mut Transcript,
        //The inputs to the PIP. This is several columns. We may eventually wish for this to be a arrow::record_batch::RecordBatch instead.
        input_columns: &[&[Scalar]],
        //The output of the PIP. Note: these are not computed by the PIP itself. The PIP simply produces a proof that these are correct.
        output_columns: &[&[Scalar]],
        //The commitments of the inputs to the PIP. This is redundant since it can be computed from input_columns, but they will already have been computed
        input_commitments: &[Commitment],
    ) -> Self;
    fn verify(
        &self,
        //The merlin transcript for the verifier
        transcript: &mut Transcript,
        //The commitments of the inputs to the PIP. Typically, these are known by the verifier.
        input_commitments: &[Commitment],
    ) -> Result<(), ProofError>;
    //The commitments of the outputs to the PIP. These should be included in the proof itself.
    fn get_output_commitments(&self) -> &[Commitment];
}
