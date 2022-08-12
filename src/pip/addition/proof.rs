use crate::base::{
    proof::{Column, Commitment, PipProve, PipVerify, ProofError, Transcript},
    scalar::IntoScalar,
};
use std::ops::Add;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdditionProof {
    pub c_sum: Commitment,
}

impl<T> PipProve<(Column<T>, Column<T>), Column<T>> for AdditionProof
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
        let sum = output;
        let (c_a, c_b) = input_commitments;

        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), sum.len());
        assert_eq!(a.len(), c_a.length);
        assert_eq!(b.len(), c_b.length);

        transcript.addition_domain_sep(a.len() as u64);

        let c_sum = c_a + c_b;
        transcript.append_point(b"c_sum", &c_sum.commitment);
        AdditionProof { c_sum }
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for AdditionProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Commitment, Commitment),
    ) -> Result<(), ProofError> {
        let (c_a, c_b) = input_commitments;
        transcript.addition_domain_sep(c_a.length as u64);

        transcript.append_point(b"c_sum", &self.c_sum.commitment);

        if c_a + c_b == self.c_sum {
            Ok(())
        } else {
            Err(ProofError::VerificationError)
        }
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_sum
    }
}
