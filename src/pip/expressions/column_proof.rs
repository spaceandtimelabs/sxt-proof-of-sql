use crate::base::{
    proof::{Column, Commit, Commitment, PipProve, PipVerify, ProofError, Transcript},
    scalar::IntoScalar,
};

#[derive(Clone, Debug)]
pub struct ColumnProof {
    pub c_out: Commitment,
}

impl<T> PipProve<(), Column<T>> for ColumnProof
where
    T: IntoScalar + Clone,
{
    fn prove(
        transcript: &mut Transcript,
        _input: (),
        output: Column<T>,
        _input_commitment: (),
    ) -> Self {
        create_column_proof(transcript, output)
    }
}

impl PipVerify<(), Commitment> for ColumnProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        _input_commitments: (),
    ) -> Result<(), ProofError> {
        transcript.column_domain_sep();
        transcript.append_point(b"c_out", &self.c_out.commitment);
        Ok(())
    }
    fn get_output_commitments(&self) -> Commitment {
        self.c_out
    }
}

fn create_column_proof<T>(transcript: &mut Transcript, output: Column<T>) -> ColumnProof
where
    T: IntoScalar + Clone,
{
    transcript.column_domain_sep();
    let c_out = output.commit();
    transcript.append_point(b"c_out", &c_out.commitment);
    ColumnProof { c_out }
}

#[cfg(test)]
mod tests {

    use super::*;
    use curve25519_dalek::scalar::Scalar;

    #[test]
    fn test_column() {
        let output: Column<Scalar> = vec![
            Scalar::from(3_u32),
            Scalar::from(4_u32),
            Scalar::from(5_u32),
            Scalar::from(7_u32),
            Scalar::from(9_u32),
            Scalar::from(1_u32),
            Scalar::from(2_u32),
        ]
        .into();

        let mut transcript = Transcript::new(b"columntest");
        let column_proof = ColumnProof::prove(&mut transcript, (), output.clone(), ());

        //the proof confirms as correct
        let mut transcript = Transcript::new(b"columntest");
        assert!(column_proof.verify(&mut transcript, ()).is_ok());

        //the output commitment is correct as well
        assert_eq!(output.commit(), column_proof.get_output_commitments());
    }
}
