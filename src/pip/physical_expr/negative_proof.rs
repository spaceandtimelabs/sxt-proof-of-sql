use crate::base::proof::{Commitment, GeneralColumn, PipProve, PipVerify, ProofError, Transcript, MessageLabel};

#[derive(Clone, Debug)]
pub struct NegativeProof {
    pub c_out: Commitment,
}

impl PipProve<(GeneralColumn,), GeneralColumn> for NegativeProof {
    fn prove(
        transcript: &mut Transcript,
        input: (GeneralColumn,),
        _output: GeneralColumn,
        input_commitment: (Commitment,),
    ) -> Self {
        let c_in = input_commitment.0;
        assert_eq!(input.0.len(), c_in.length);
        create_negative_proof(transcript, c_in)
    }
}

impl PipVerify<(Commitment,), Commitment> for NegativeProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Commitment,),
    ) -> Result<(), ProofError> {
        verify_proof(transcript, self, input_commitments.0)
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_out
    }
}

fn create_negative_proof(transcript: &mut Transcript, c_in: Commitment) -> NegativeProof {
    let c_out = -c_in;
    transcript.append_auto(MessageLabel::Negative, &c_out.as_compressed()).unwrap();

    NegativeProof { c_out }
}

fn verify_proof(
    transcript: &mut Transcript,
    proof: &NegativeProof,
    c_in: Commitment,
) -> Result<(), ProofError> {
    let c_out = -c_in;
    transcript.append_auto(MessageLabel::Negative, &proof.c_out.as_compressed())?;
    if proof.c_out != c_out {
        Err(ProofError::VerificationError)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::base::proof::Commit;

    #[test]
    fn test_negative_proof() {
        let input = GeneralColumn::SafeIntColumn(vec![1, -2, 3].into());
        let output = GeneralColumn::SafeIntColumn(vec![-1, 2, -3].into());

        let mut transcript = Transcript::new(b"negativetest");
        let c_in = input.commit();
        let negative_proof =
            NegativeProof::prove(&mut transcript, (input.clone(),), output.clone(), (c_in,));

        //the proof confirms as correct
        let mut transcript = Transcript::new(b"negativetest");
        assert!(negative_proof.verify(&mut transcript, (c_in,)).is_ok());

        //the output commitment is correct as well
        assert_eq!(output.commit(), negative_proof.get_output_commitments());

        //wrong input commitments
        let mut transcript = Transcript::new(b"negativetest");
        assert!(negative_proof.verify(&mut transcript, (-c_in,)).is_err());
    }
}
