use crate::base::{
    proof::{Column, Commitment, PipProve, PipVerify, ProofError, Transcript},
    scalar::IntoScalar,
};

#[derive(Clone, Debug)]
pub struct NegativeProof {
    pub c_out: Commitment,
}

impl<T> PipProve<(Column<T>,), Column<T>> for NegativeProof
where
    T: IntoScalar + Clone,
{
    fn prove(
        transcript: &mut Transcript,
        input: (Column<T>,),
        _output: Column<T>,
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
    transcript.negative_domain_sep();
    let c_out = -c_in;
    transcript.append_point(b"c_out", &c_out.commitment);

    NegativeProof { c_out }
}

fn verify_proof(
    transcript: &mut Transcript,
    proof: &NegativeProof,
    c_in: Commitment,
) -> Result<(), ProofError> {
    transcript.negative_domain_sep();
    let c_out = -c_in;
    transcript.append_point(b"c_out", &proof.c_out.commitment);
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
    use curve25519_dalek::scalar::Scalar;

    #[test]
    fn test_negative_proof() {
        let input: Column<Scalar> = vec![
            3_i32.into_scalar(),
            4_i32.into_scalar(),
            5_i32.into_scalar(),
            7_i32.into_scalar(),
            9_i32.into_scalar(),
            1_i32.into_scalar(),
            2_i32.into_scalar(),
        ]
        .into();
        let output: Column<Scalar> = vec![
            (-3_i32).into_scalar(),
            (-4_i32).into_scalar(),
            (-5_i32).into_scalar(),
            (-7_i32).into_scalar(),
            (-9_i32).into_scalar(),
            (-1_i32).into_scalar(),
            (-2_i32).into_scalar(),
        ]
        .into();

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
