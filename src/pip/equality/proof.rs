use crate::base::proof::{Commitment, PIPProof, ProofError, Transcript};
use crate::pip::hadamard::HadamardProof;
use curve25519_dalek::scalar::Scalar;
use std::iter;
use std::slice;

pub struct EqualityProof {
    pub c_c: Commitment,
    pub c_e: Commitment,
    pub proof_ez0: HadamardProof,
    pub proof_czd: HadamardProof,
}

impl PIPProof for EqualityProof {
    fn create(
        transcript: &mut Transcript,
        input_columns: &[&[Scalar]],
        output_columns: &[&[Scalar]],
        input_commitments: &[Commitment],
    ) -> Self {
        let a = input_columns[0];
        let b = input_columns[1];
        let e = output_columns[0];
        let c_a = input_commitments[0];
        let c_b = input_commitments[1];
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), e.len());
        assert_eq!(a.len(), c_a.length);
        assert_eq!(a.len(), c_b.length);
        create_equality_proof(transcript, a, b, e, c_a, c_b)
    }

    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: &[Commitment],
    ) -> Result<(), ProofError> {
        let c_a = input_commitments[0];
        let c_b = input_commitments[1];
        verify_proof(transcript, self, c_a, c_b)
    }

    fn get_output_commitments(&self) -> &[Commitment] {
        slice::from_ref(&self.c_e)
    }
}

fn create_equality_proof(
    transcript: &mut Transcript,
    a: &[Scalar],
    b: &[Scalar],
    e: &[Scalar],
    c_a: Commitment,
    c_b: Commitment,
) -> EqualityProof {
    transcript.equality_domain_sep(c_a.length as u64);
    let (z_vec, c_vec): (Vec<Scalar>, Vec<Scalar>) = a
        .iter()
        .zip(b)
        .map(|(ai, bi)| {
            let zi = ai - bi;
            let ci = if zi == Scalar::zero() {
                Scalar::zero()
            } else {
                zi.invert()
            };
            (zi, ci)
        })
        .unzip();

    let d_vec: Vec<Scalar> = e.iter().map(|ei| Scalar::one() - ei).collect();
    let zero_vec: Vec<Scalar> = iter::repeat(Scalar::zero()).take(a.len()).collect();

    let c_c = Commitment::from(&c_vec[..]);
    let c_e = Commitment::from(e);
    let c_z = c_a - c_b;
    transcript.append_point(b"c_c", &c_c.commitment);
    transcript.append_point(b"c_e", &c_e.commitment);
    let proof_ez0 = HadamardProof::create(transcript, &[e, &z_vec], &[&zero_vec], &[c_e, c_z]);
    let proof_czd = HadamardProof::create(transcript, &[&c_vec, &z_vec], &[&d_vec], &[c_c, c_z]);

    EqualityProof {
        c_c,
        c_e,
        proof_ez0,
        proof_czd,
    }
}

fn verify_proof(
    transcript: &mut Transcript,
    proof: &EqualityProof,
    c_a: Commitment,
    c_b: Commitment,
) -> Result<(), ProofError> {
    transcript.equality_domain_sep(c_a.length as u64);
    // Computing c_0 and c_1 here is terrible. It should be cached.
    let (zero, one): (Vec<Scalar>, Vec<Scalar>) = iter::repeat((Scalar::zero(), Scalar::one()))
        .take(c_a.length)
        .unzip();
    let c_0 = Commitment::from(&zero[..]);
    let c_1 = Commitment::from(&one[..]);
    let c_d = c_1 - proof.c_e;
    let c_z = c_a - c_b;
    transcript.append_point(b"c_c", &proof.c_c.commitment);
    transcript.append_point(b"c_e", &proof.c_e.commitment);
    proof.proof_ez0.verify(transcript, &[proof.c_e, c_z])?;
    proof.proof_czd.verify(transcript, &[proof.c_c, c_z])?;
    if proof.proof_ez0.commit_ab != c_0 || proof.proof_czd.commit_ab != c_d {
        Err(ProofError::VerificationError)
    } else {
        Ok(())
    }
}
