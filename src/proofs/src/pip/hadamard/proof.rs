use curve25519_dalek::constants;
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::Identity;
use pedersen::compute::compute_commitments;
use pedersen::compute::get_generators;
use std::cmp;
use std::slice;

use crate::base::math::{is_pow2, log2_up};
use crate::base::polynomial::CompositePolynomialInfo;
use crate::base::proof::MessageLabel;
use crate::base::proof::{Column, Commitment, PipProve, PipVerify, ProofError, Transcript};
use crate::base::scalar::inner_product;
use crate::pip::hadamard::{compute_evaluation_vector, make_sumcheck_polynomial};
use crate::proof_primitive::inner_product::InnerProductProof;
use crate::proof_primitive::sumcheck::SumcheckProof;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HadamardProof {
    pub(super) commit_ab: Commitment,
    sumcheck_proof: SumcheckProof,
    pub(super) f_a: Scalar,
    f_a_proof: InnerProductProof,
    pub(super) f_b: Scalar,
    f_b_proof: InnerProductProof,
    f_ab_proof: InnerProductProof,
}

impl PipProve<(Column<Scalar>, Column<Scalar>), Column<Scalar>> for HadamardProof {
    /// Create a hadamard proof.
    ///
    /// See protocols/multiplication.pdf
    fn prove(
        transcript: &mut Transcript,
        input: (Column<Scalar>, Column<Scalar>),
        output: Column<Scalar>,
        _input_commitments: (Commitment, Commitment),
    ) -> Self {
        let a_vec = input.0;
        let b_vec = input.1;
        let ab_vec = output;

        let n = a_vec.len();
        assert!(n > 0);
        assert_eq!(a_vec.len(), n);
        assert_eq!(b_vec.len(), n);
        assert_eq!(ab_vec.len(), n);

        let num_vars = compute_num_variables(n);
        if is_pow2(n) && n > 1 {
            return create_proof_impl(transcript, &a_vec, &b_vec, &ab_vec, num_vars, n);
        }
        let n_p = 1 << num_vars;
        let a_vec = extend_scalar_vector(a_vec.as_slice(), n_p);
        let b_vec = extend_scalar_vector(b_vec.as_slice(), n_p);
        let ab_vec = extend_scalar_vector(ab_vec.as_slice(), n_p);
        create_proof_impl(transcript, &a_vec, &b_vec, &ab_vec, num_vars, n)
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for HadamardProof {
    /// Verifies that a hadamard proof is correct given the associated commitments.
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Commitment, Commitment),
    ) -> Result<(), ProofError> {
        let commit_a = input_commitments.0;
        let commit_b = input_commitments.1;
        assert_eq!(input_commitments.0.length, input_commitments.1.length);
        let n = input_commitments.0.length;

        let num_vars = compute_num_variables(n);

        let n = 1 << num_vars;
        transcript
            .append_auto(
                MessageLabel::Hadamard,
                &(num_vars, self.commit_ab.as_compressed()),
            )
            .unwrap();
        let mut r_vec = vec![Scalar::from(0u64); n];
        transcript.challenge_scalars(&mut r_vec, MessageLabel::HadamardChallenge);

        let polynomial_info = CompositePolynomialInfo {
            max_multiplicands: 3,
            num_variables: num_vars,
        };
        let subclaim = self.sumcheck_proof.verify_without_evaluation(
            transcript,
            polynomial_info,
            &Scalar::zero(),
        )?;
        let evaluation_vec = compute_evaluation_vector(&subclaim.evaluation_point);
        let f_r = inner_product(&r_vec, &evaluation_vec);

        // subclam.expected_evaluation == f_r * (f_a * f_b - f_ab) or
        // f_ab == f_a * f_b - subclam.expected_evaluation / f_r
        if f_r == Scalar::zero() {
            // Note: This happens with probability nearly zero
            return Ok(());
        }
        let f_ab = self.f_a * self.f_b - subclaim.expected_evaluation * f_r.invert();

        let mut generators = vec![constants::RISTRETTO_BASEPOINT_POINT; n + 1];

        get_generators(&mut generators[..], 0);

        let product_g = generators[n];

        // verify f_a
        let f_commit = commit_a.try_as_decompressed()? + self.f_a * product_g;
        self.f_a_proof.verify(
            transcript,
            &f_commit,
            &product_g,
            &generators[0..n],
            &evaluation_vec,
        )?;

        // verify f_b
        let f_commit = commit_b.try_as_decompressed()? + self.f_b * product_g;
        self.f_b_proof.verify(
            transcript,
            &f_commit,
            &product_g,
            &generators[0..n],
            &evaluation_vec,
        )?;

        // verify f_ab
        let f_commit = self.commit_ab.try_as_decompressed()? + f_ab * product_g;
        self.f_ab_proof.verify(
            transcript,
            &f_commit,
            &product_g,
            &generators[0..n],
            &evaluation_vec,
        )?;

        Ok(())
    }

    fn get_output_commitments(&self) -> Commitment {
        self.commit_ab
    }
}

fn compute_num_variables(n: usize) -> usize {
    // Note: This isn't a space efficient way of handling
    // the case n == 1, but keeping it simple for the first iteration
    cmp::max(log2_up(n), 1)
}

fn extend_scalar_vector(a_vec: &[Scalar], n: usize) -> Vec<Scalar> {
    let mut vec = Vec::with_capacity(n);
    for a in a_vec {
        vec.push(*a);
    }
    for _ in a_vec.len()..n {
        vec.push(Scalar::from(0u64));
    }
    vec
}

fn create_proof_impl(
    transcript: &mut Transcript,
    a_vec: &[Scalar],
    b_vec: &[Scalar],
    ab_vec: &[Scalar],
    num_vars: usize,
    length: usize,
) -> HadamardProof {
    let mut c_ab = CompressedRistretto::identity();
    compute_commitments(slice::from_mut(&mut c_ab), &[ab_vec]);

    transcript
        .append_auto(MessageLabel::Hadamard, &(num_vars, c_ab))
        .unwrap();
    let n = a_vec.len();

    let mut r_vec = vec![Scalar::zero(); n];
    transcript.challenge_scalars(&mut r_vec, MessageLabel::HadamardChallenge);

    let poly = make_sumcheck_polynomial(num_vars, a_vec, b_vec, ab_vec, &r_vec);
    let mut evaluation_point = vec![Scalar::zero(); poly.num_variables];
    let sumcheck_proof = SumcheckProof::create(transcript, &mut evaluation_point, &poly);

    let evaluation_vec = compute_evaluation_vector(&evaluation_point);
    let mut generators = vec![constants::RISTRETTO_BASEPOINT_POINT; n + 1];

    get_generators(&mut generators, 0);

    let product_g = generators[n];

    let f_a = inner_product(&evaluation_vec, a_vec);
    let f_a_proof = InnerProductProof::create(
        transcript,
        &product_g,
        &generators[0..n],
        a_vec,
        &evaluation_vec,
    );

    let f_b = inner_product(&evaluation_vec, b_vec);
    let f_b_proof = InnerProductProof::create(
        transcript,
        &product_g,
        &generators[0..n],
        b_vec,
        &evaluation_vec,
    );

    let f_ab_proof = InnerProductProof::create(
        transcript,
        &product_g,
        &generators[0..n],
        ab_vec,
        &evaluation_vec,
    );

    HadamardProof {
        commit_ab: Commitment::from_compressed(c_ab, length),
        sumcheck_proof,
        f_a,
        f_a_proof,
        f_b,
        f_b_proof,
        f_ab_proof,
    }
}
