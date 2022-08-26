use crate::base::{
    proof::{Column, Commitment, GeneralColumn, PipProve, PipVerify, ProofError, Transcript},
    scalar::IntoScalar,
};
use crate::pip::hadamard::HadamardProof;
use curve25519_dalek::scalar::Scalar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CaseWhenProof {
    pub c_c: Commitment,          //Commitment: output (claimed) of CASE-WHEN query.
    pub proof_pzy: HadamardProof, //Hadamard proof for p*(a-b) = c-b
}

impl PipProve<(GeneralColumn, GeneralColumn, Column<bool>), GeneralColumn> for CaseWhenProof {
    fn prove(
        transcript: &mut Transcript,
        (a, b, p): (GeneralColumn, GeneralColumn, Column<bool>), //Input columns: a, b, (predicate) p
        c: GeneralColumn, //Outputs: (claimed CASEWHEN column) c
        (c_a, c_b, c_p): (Commitment, Commitment, Commitment), //Input commitments
    ) -> Self {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), c.len());
        assert_eq!(a.len(), p.len());
        assert_eq!(a.len(), c_a.length);
        assert_eq!(a.len(), c_b.length);
        assert_eq!(a.len(), c_p.length);

        create_casewhen_proof(transcript, (a, b, p), c, (c_a, c_b, c_p))
    }
}

impl PipVerify<(Commitment, Commitment, Commitment), Commitment> for CaseWhenProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        (c_a, c_b, c_p): (Commitment, Commitment, Commitment), //Input commitments
    ) -> Result<(), ProofError> {
        verify_proof(transcript, self, c_a, c_b, c_p)
    }
    fn get_output_commitments(&self) -> Commitment {
        self.c_c
    }
}

fn create_casewhen_proof(
    transcript: &mut Transcript,
    (a, b, p): (GeneralColumn, GeneralColumn, Column<bool>), //inputs
    c: GeneralColumn,                                        //output
    (c_a, c_b, c_p): (Commitment, Commitment, Commitment),   //input commitments
) -> CaseWhenProof {
    //Generating columns for the HadamardProof: p*(a-b) = c-b.
    //Let z = a-b and y = c-b.
    //HadamardProof: p*z = y.

    let a_column = Column::<Scalar>::from(a);
    let b_column = Column::<Scalar>::from(b);
    let c_column = Column::<Scalar>::from(c);

    let z_vec: Vec<Scalar> = a_column
        .iter()
        .zip(b_column.iter())
        .map(|(ai, bi)| {
            let zi: Scalar = ai - bi;
            zi
        })
        .collect();
    let y_vec: Vec<Scalar> = c_column
        .iter()
        .zip(b_column.iter())
        .map(|(ci, bi)| {
            let yi: Scalar = ci - bi;
            yi
        })
        .collect();

    //Converts boolean array p into scalar array p_scalar
    //Needed for HadamardProof
    let p_scalar: Vec<Scalar> = p.iter().map(|pi| pi.into_scalar()).collect();

    let c_z = c_a - c_b;

    let c_c = Commitment::from(c_column.as_slice()); //Commits to c

    //Add c_c to the transcript
    transcript.append_point(b"c_c", &c_c.commitment);

    //Generate HadamardProof for p*(a-b).
    let proof_pzy = HadamardProof::prove(
        transcript,
        (Column { data: p_scalar }, Column { data: z_vec }),
        Column { data: y_vec },
        (c_p, c_z),
    );

    CaseWhenProof { c_c, proof_pzy }
}

fn verify_proof(
    transcript: &mut Transcript,
    proof: &CaseWhenProof,
    c_a: Commitment,
    c_b: Commitment,
    c_p: Commitment,
) -> Result<(), ProofError> {
    let c_z = c_a - c_b;

    transcript.append_point(b"c_c", &proof.c_c.commitment);
    proof.proof_pzy.verify(transcript, (c_p, c_z))?;
    if (proof.proof_pzy.commit_ab + c_b) != proof.c_c {
        Err(ProofError::VerificationError)
    } else {
        Ok(())
    }
}
