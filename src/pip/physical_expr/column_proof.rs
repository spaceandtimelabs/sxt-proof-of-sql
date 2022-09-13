use crate::base::proof::{
    Commit, Commitment, GeneralColumn, MessageLabel, PipProve, PipVerify, ProofError, Transcript,
};

#[derive(Clone, Debug)]
pub struct ColumnProof {
    c_out: Commitment,
}

impl PipProve<(), GeneralColumn> for ColumnProof {
    fn prove(
        transcript: &mut Transcript,
        _input: (),
        output: GeneralColumn,
        _input_commitment: (),
    ) -> Self {
        let c_out = output.commit();
        transcript
            .append_auto(MessageLabel::Column, &c_out.as_compressed())
            .unwrap();
        ColumnProof { c_out }
    }
}

impl PipVerify<(), Commitment> for ColumnProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        _input_commitments: (),
    ) -> Result<(), ProofError> {
        transcript.append_auto(MessageLabel::Column, &self.c_out.as_compressed())?;
        Ok(())
    }
    fn get_output_commitments(&self) -> Commitment {
        self.c_out
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_column_proof() {
        let output = GeneralColumn::SafeIntColumn(vec![1, 2, 3].into());

        let mut transcript = Transcript::new(b"columntest");
        let column_proof = ColumnProof::prove(&mut transcript, (), output.clone(), ());

        //the proof confirms as correct
        let mut transcript = Transcript::new(b"columntest");
        assert!(column_proof.verify(&mut transcript, ()).is_ok());

        //the output commitment is correct as well
        assert_eq!(output.commit(), column_proof.get_output_commitments());
    }
}
