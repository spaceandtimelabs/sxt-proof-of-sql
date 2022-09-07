use crate::base::proof::{
    Commit, Commitment, GeneralColumn, PipProve, PipVerify, ProofError, Transcript,
};

#[derive(Clone, Debug)]
pub struct LiteralProof {
    pub c_out: Commitment,
}

impl PipProve<(), GeneralColumn> for LiteralProof {
    fn prove(
        transcript: &mut Transcript,
        _input: (),
        output: GeneralColumn,
        _input_commitment: (),
    ) -> Self {
        transcript.literal_domain_sep();
        let c_out = output.commit();
        transcript.append_commitment(b"c_out", &c_out);
        LiteralProof { c_out }
    }
}

impl PipVerify<(), Commitment> for LiteralProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        _input_commitments: (),
    ) -> Result<(), ProofError> {
        transcript.literal_domain_sep();
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
        let output = GeneralColumn::SafeIntColumn(vec![5].into());

        let mut transcript = Transcript::new(b"literaltest");
        let column_proof = LiteralProof::prove(&mut transcript, (), output.clone(), ());

        //the proof confirms as correct
        let mut transcript = Transcript::new(b"literaltest");
        assert!(column_proof.verify(&mut transcript, ()).is_ok());

        //the output commitment is correct as well
        assert_eq!(output.commit(), column_proof.get_output_commitments());
    }
}
