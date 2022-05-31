use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use sha3::Sha3_512;

use crate::base::math::{is_pow2, log2_up};
use crate::base::proof::ProofError;
use crate::base::proof::TranscriptProtocol;
use crate::pip::multiplication::make_sumcheck_polynomial;
use crate::pip::sumcheck::SumcheckProof;

pub struct MultiplicationProof {
    pub num_vars: usize,
    pub commit_ab: CompressedRistretto,
    pub sumcheck_proof: SumcheckProof,
}

impl MultiplicationProof {
    /// Create a multiplication proof.
    ///
    /// See protocols/multiplication.pdf
    #[allow(unused_variables)]
    pub fn create(
        transcript: &mut Transcript,
        a_vec: &[Scalar],
        b_vec: &[Scalar],
    ) -> MultiplicationProof {
        let n = a_vec.len();
        assert!(n > 0);
        assert_eq!(a_vec.len(), n);
        assert_eq!(b_vec.len(), n);

        let c_ab = RistrettoPoint::hash_from_bytes::<Sha3_512>(b"ab").compress(); // pretend like this is the commitment of ab

        let num_vars = log2_up(n);
        if is_pow2(n) {
            return create_proof_impl(transcript, a_vec, b_vec, c_ab, num_vars);
        }
        let n = 1 << num_vars;
        let a_vec = extend_scalar_vector(a_vec, n);
        let b_vec = extend_scalar_vector(b_vec, n);
        create_proof_impl(transcript, &a_vec, &b_vec, c_ab, num_vars)
    }

    /// Verifies that a multiplication proof is correct given the associated commitments.
    #[allow(unused_variables)]
    pub fn verify(
        &self,
        transcript: &mut Transcript,
        commit_a: &CompressedRistretto,
        commit_b: &CompressedRistretto,
    ) -> Result<(), ProofError> {
        let n = 1 << self.num_vars;
        transcript.validate_and_append_point(b"c_ab", &self.commit_ab).unwrap();
        let mut r_vec = vec![Scalar::from(0u64); n];
        transcript.challenge_scalars(&mut r_vec, b"r_vec");
        
        let mut evaluation_point = vec![Scalar::from(0u64); self.num_vars];
        self.sumcheck_proof.verify_without_evaluation(&mut evaluation_point, transcript).unwrap();

        // TODO(rnburn): verify bullet proofs

        Ok(())
    }
}

fn extend_scalar_vector(a_vec: &[Scalar], n: usize) -> Vec<Scalar> {
    let mut vec = Vec::with_capacity(n);
    for i in 0..a_vec.len() {
        vec.push(a_vec[i]);
    }
    for _ in a_vec.len()..n {
        vec.push(Scalar::from(0u64));
    }
    vec
}

#[allow(unused_variables)]
fn create_proof_impl(
    transcript: &mut Transcript,
    a_vec: &[Scalar],
    b_vec: &[Scalar],
    c_ab: CompressedRistretto,
    num_vars: usize,
) -> MultiplicationProof {
    let n = a_vec.len();
    transcript.append_point(b"c_ab", &c_ab);
    let mut r_vec = vec![Scalar::from(0u64); a_vec.len()];
    transcript.challenge_scalars(&mut r_vec, b"r_vec");
    let ab_vec: Vec<Scalar> = a_vec.iter().zip(b_vec.iter()).map(|(a, b)| a * b).collect();
    let poly = make_sumcheck_polynomial(num_vars, a_vec, b_vec, &ab_vec, &r_vec);
    let sumcheck_proof = SumcheckProof::create(transcript, &poly);

    // TODO(rnburn: create bullet proofs

    MultiplicationProof { 
        num_vars: num_vars,
        commit_ab: c_ab,
        sumcheck_proof: sumcheck_proof,
    }
}
