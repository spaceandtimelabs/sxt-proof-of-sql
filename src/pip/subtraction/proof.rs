use crate::base::{
    proof::{Column, Commitment, PipProve, PipVerify, ProofError, Transcript},
    scalar::IntoScalar,
};
use serde::{Deserialize, Serialize};
use std::ops::Add;

#[derive(Debug, Serialize, Deserialize)]
pub struct SubtractionProof {
    pub c_diff: Commitment,
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
        let (a, b) = input;
        let diff = output;
        let (c_a, c_b) = input_commitments;

        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), diff.len());
        assert_eq!(a.len(), c_a.length);
        assert_eq!(b.len(), c_b.length);

        transcript.subtraction_domain_sep(a.len() as u64);

        let c_diff = c_a - c_b;
        transcript.append_point(b"c_diff", &c_diff.commitment);
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

        transcript.append_point(b"c_diff", &self.c_diff.commitment);

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
