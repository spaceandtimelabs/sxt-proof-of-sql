use crate::base::proof::{
    Commit, Commitment, GeneralColumn, PipProve, PipVerify, ProofError, Transcript,
};

#[derive(Clone, Debug)]
pub struct ColumnProof {
    pub c_out: Commitment,
}

impl PipProve<(), GeneralColumn> for ColumnProof {
    fn prove(
        transcript: &mut Transcript,
        _input: (),
        output: GeneralColumn,
        _input_commitment: (),
    ) -> Self {
        transcript.column_domain_sep();
        let c_out = output.commit();
        transcript.append_commitment(b"c_out", &c_out);
        ColumnProof { c_out }
    }
}

impl PipVerify<(), Commitment> for ColumnProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        _input_commitments: (),
    ) -> Result<(), ProofError> {
        transcript.column_domain_sep();
        transcript.append_commitment(b"c_out", &self.c_out);
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
