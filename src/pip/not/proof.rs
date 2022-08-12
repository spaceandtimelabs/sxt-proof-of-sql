use crate::base::proof::{Column, Commitment, PipProve, PipVerify, ProofError, Transcript};

use serde::{Deserialize, Serialize};

/// Implementation of Not logical operator. This uses the fact that !x = 1-x.
#[derive(Serialize, Deserialize)]
pub struct NotProof {
    pub input_commitment: Commitment,
}

impl PipProve<(Column<bool>,), Column<bool>> for NotProof {
    fn prove(
        transcript: &mut Transcript,
        _input: (Column<bool>,),
        _output: Column<bool>,
        input_commitments: (Commitment,),
    ) -> Self {
        transcript.not_domain_sep(input_commitments.0.length as u64);
        NotProof {
            input_commitment: input_commitments.0,
        }
    }
}

impl PipVerify<(Commitment,), Commitment> for NotProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Commitment,),
    ) -> Result<(), ProofError> {
        transcript.not_domain_sep(input_commitments.0.length as u64);
        // Note: this isn't really checking much, because as long as the input commitments match, the output commitment is guaranteed to be correct.
        if input_commitments.0 == self.input_commitment {
            Ok(())
        } else {
            Err(ProofError::VerificationError)
        }
    }

    fn get_output_commitments(&self) -> Commitment {
        Commitment::from_ones(self.input_commitment.length) - self.input_commitment
    }
}
