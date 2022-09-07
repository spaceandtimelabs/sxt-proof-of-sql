use crate::base::{
    proof::{Column, Commitment, GeneralColumn, PipProve, PipVerify, ProofError, Transcript},
    scalar::IntoScalar,
};
use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubtractionProof {
    pub c_diff: Commitment,
}

impl PipProve<(GeneralColumn, GeneralColumn), GeneralColumn> for SubtractionProof {
    fn prove(
        //The merlin transcript for the prover
        transcript: &mut Transcript,
        //The inputs to the PIP
        (left, right): (GeneralColumn, GeneralColumn),
        //The output of the PIP. Note: these are not computed by the PIP itself. The PIP simply produces a proof that these are correct.
        output: GeneralColumn,
        //The commitments of the inputs to the PIP. This is redundant since it can be computed from input_columns, but they will already have been computed
        input_commitment: (Commitment, Commitment),
    ) -> Self {
        // general implementation
        // This will match against the type variants of the input and output columns,
        // and error if the combination of column types aren't valid for this proof.
        // The actual proof construction is handled in the core implementation.
        use GeneralColumn::*;
        match (left, right, output) {
            (Int8Column(left), Int8Column(right), Int8Column(output)) => {
                SubtractionProof::prove(transcript, (left, right), output, input_commitment)
            }
            (Int16Column(left), Int16Column(right), Int16Column(output)) => {
                SubtractionProof::prove(transcript, (left, right), output, input_commitment)
            }
            (Int32Column(left), Int32Column(right), Int32Column(output)) => {
                SubtractionProof::prove(transcript, (left, right), output, input_commitment)
            }
            (Int64Column(left), Int64Column(right), Int64Column(output)) => {
                SubtractionProof::prove(transcript, (left, right), output, input_commitment)
            }
            _ => {
                panic!("type error");
            }
        }
    }
}

impl<T> PipProve<(Column<T>, Column<T>), Column<T>> for SubtractionProof
where
    T: IntoScalar + Clone + Add,
{
    fn prove(
        transcript: &mut Transcript,
        input: (Column<T>, Column<T>),
        output: Column<T>,
        input_commitments: (Commitment, Commitment),
    ) -> Self {
        // core implementation
        let (a, b) = input;
        let diff = output;
        let (c_a, c_b) = input_commitments;

        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), diff.len());
        assert_eq!(a.len(), c_a.length);
        assert_eq!(b.len(), c_b.length);

        transcript.subtraction_domain_sep(a.len() as u64);

        let c_diff = c_a - c_b;
        transcript.append_commitment(b"c_diff", &c_diff);
        SubtractionProof { c_diff }
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for SubtractionProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Commitment, Commitment),
    ) -> Result<(), ProofError> {
        let (c_a, c_b) = input_commitments;
        transcript.subtraction_domain_sep(c_a.length as u64);

        transcript.append_commitment(b"c_diff", &self.c_diff);

        if c_a - c_b == self.c_diff {
            Ok(())
        } else {
            Err(ProofError::VerificationError)
        }
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_diff
    }
}
