use super::{
    eval_vmv_re_prove, eval_vmv_re_verify, extended_dory_inner_product_prove,
    extended_dory_inner_product_verify, DoryCommitment, DoryMessages, DoryProverPublicSetup,
    DoryScalar, DoryVerifierPublicSetup, G1Affine, G1Projective, ProverSetup, VMVProverState,
    VMVVerifierState, F,
};
use crate::base::{commitment::CommitmentEvaluationProof, polynomial::compute_evaluation_vector};
use ark_ec::{AffineRepr, VariableBaseMSM};
use merlin::Transcript;
use num_traits::{One, Zero};
use thiserror::Error;

/// The `CommitmentEvaluationProof` for the Dory PCS.
pub type DoryEvaluationProof = DoryMessages;

/// The error type for the Dory PCS.
#[derive(Error, Debug)]
pub enum DoryError {
    /// This error occurs when the generators offset is invalid.
    #[error("invalid generators offset: {0}")]
    InvalidGeneratorsOffset(u64),
    /// This error occurs when the proof fails to verify.
    #[error("verification error")]
    VerificationError,
    /// This error occurs when the setup is too small.
    #[error("setup is too small: the setup is {0}, but the proof requires a setup of size {1}")]
    SmallSetup(usize, usize),
}

impl CommitmentEvaluationProof for DoryEvaluationProof {
    type Scalar = DoryScalar;
    type Commitment = DoryCommitment;
    type Error = DoryError;
    type ProverPublicSetup = DoryProverPublicSetup;
    type VerifierPublicSetup = DoryVerifierPublicSetup;

    #[tracing::instrument(
        name = "proofs.proof_primitive_dory.dory_commitment_evaluation_proof.new",
        level = "info",
        skip_all
    )]
    fn new(
        transcript: &mut Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        setup: &Self::ProverPublicSetup,
    ) -> Self {
        // Dory PCS Logic
        if generators_offset != 0 {
            // TODO: support offsets other than 0.
            // Note: this will always result in a verification error.
            return Default::default();
        }
        let a: &[F] = bytemuck::TransparentWrapper::peel_slice(a);
        let b_point: &[F] = bytemuck::TransparentWrapper::peel_slice(b_point);
        let prover_setup: &ProverSetup = &setup.public_parameters().into();
        let nu = compute_nu(b_point.len(), setup.sigma());
        if nu > prover_setup.max_nu {
            return Default::default(); // Note: this will always result in a verification error.
        }
        let (L_vec, R_vec) = compute_l_r(b_point, setup.sigma(), nu);
        let v_vec = compute_v_vec(a, &L_vec, setup.sigma(), nu);
        let T_vec_prime = compute_T_vec_prime(a, setup.sigma(), nu, prover_setup);
        let state = VMVProverState {
            v_vec,
            T_vec_prime,
            L_vec,
            R_vec,
            nu,
        };

        let mut messages = Default::default();
        let extended_state = eval_vmv_re_prove(&mut messages, transcript, state, prover_setup);
        extended_dory_inner_product_prove(&mut messages, transcript, extended_state, prover_setup);
        messages
    }

    fn verify_proof(
        &self,
        transcript: &mut Transcript,
        a_commit: &Self::Commitment,
        product: &Self::Scalar,
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _table_length: usize,
        setup: &Self::VerifierPublicSetup,
    ) -> Result<(), Self::Error> {
        // Dory PCS Logic
        if generators_offset != 0 {
            return Err(DoryError::InvalidGeneratorsOffset(generators_offset));
        }
        let b_point: &[F] = bytemuck::TransparentWrapper::peel_slice(b_point);
        let verifier_setup = setup.verifier_setup();
        let mut messages = self.clone();
        let nu = compute_nu(b_point.len(), setup.sigma());
        if nu > verifier_setup.max_nu {
            return Err(DoryError::SmallSetup(verifier_setup.max_nu, nu));
        }
        let (L_vec, R_vec) = compute_l_r(b_point, setup.sigma(), nu);
        let state = VMVVerifierState {
            y: product.0,
            T: a_commit.0,
            L_vec,
            R_vec,
            nu,
        };
        let extended_state = eval_vmv_re_verify(&mut messages, transcript, state, verifier_setup)
            .ok_or(DoryError::VerificationError)?;
        if !extended_dory_inner_product_verify(
            &mut messages,
            transcript,
            extended_state,
            verifier_setup,
        ) {
            Err(DoryError::VerificationError)?;
        }
        Ok(())
    }
}

/// Compute the evaluations of the columns of the matrix M that is derived from `a`.
fn compute_v_vec(a: &[F], L_vec: &[F], sigma: usize, nu: usize) -> Vec<F> {
    a.chunks(1 << sigma)
        .zip(L_vec.iter())
        .fold(vec![F::zero(); 1 << nu], |mut v, (row, l)| {
            v.iter_mut().zip(row).for_each(|(v, a)| *v += l * a);
            v
        })
}

/// Compute the commitments to the rows of the matrix M that is derived from `a`.
#[tracing::instrument(
    name = "proofs.proof_primitive.dory.compute_T_vec_prime",
    level = "info",
    skip_all
)]
fn compute_T_vec_prime(
    a: &[F],
    sigma: usize,
    nu: usize,
    prover_setup: &ProverSetup,
) -> Vec<G1Affine> {
    a.chunks(1 << sigma)
        .map(|row| G1Projective::msm_unchecked(prover_setup.Gamma_1[nu], row).into())
        .chain(core::iter::repeat(G1Affine::zero()))
        .take(1 << nu)
        .collect()
}

/// Compute the size of the matrix M that is derived from `a`.
/// More specifically compute `nu`, where 2^nu is the side length the square matrix, M.
/// `num_vars` is the number of variables in the polynomial. In other words, it is the length of `b_points`, which is `ceil(log2(len(a)))`.
fn compute_nu(num_vars: usize, sigma: usize) -> usize {
    if num_vars <= sigma * 2 {
        // This is the scenario where we don't need to pad the columns.
        sigma
    } else {
        // This is the scenario where we need to pad the columns.
        num_vars - sigma
    }
}

/// Compute the vectors L and R that are derived from `b_point`.
/// L and R are the vectors such that LMR is exactly the evaluation of `a` at the point `b_point`.
fn compute_l_r(b_point: &[F], sigma: usize, nu: usize) -> (Vec<F>, Vec<F>) {
    let mut R_vec = vec![Zero::zero(); 1 << nu];
    let mut L_vec = vec![Zero::zero(); 1 << nu];
    let num_vars = b_point.len();
    if num_vars == 0 {
        // This is the scenario where we only need a single element in the matrix.
        R_vec[0] = One::one();
        L_vec[0] = One::one();
    } else if num_vars <= sigma {
        // This is the scenario where we only need a single row of the matrix.
        compute_evaluation_vector(&mut R_vec[..1 << num_vars], b_point);
        L_vec[0] = One::one();
    } else if num_vars <= sigma * 2 {
        // This is the scenario where we need more than a single row, but we don't need to pad the columns.
        compute_evaluation_vector(&mut R_vec, &b_point[..nu]);
        compute_evaluation_vector(&mut L_vec[..1 << (num_vars - nu)], &b_point[nu..]);
    } else {
        // This is the scenario where we need to pad the columns.
        compute_evaluation_vector(&mut R_vec[..(1 << sigma)], &b_point[..sigma]);
        compute_evaluation_vector(&mut L_vec, &b_point[sigma..]);
    }

    (L_vec, R_vec)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::UniformRand;
    use core::iter::repeat_with;

    /// This method is simply computing the product LMR where M is the matrix filled, row by row, with `a` but with row length 2^sigma.
    /// Note: because L and R are the same length, and have length at least 2^sigma, we simply pad everything else in the matrix with zeros.
    fn compute_LMR(a: &[F], L: &[F], R: &[F], sigma: usize) -> F {
        assert_eq!(L.len(), R.len());
        assert!(L.len() >= 1 << sigma);
        assert!(R.len() >= 1 << sigma);

        let num_columns = 1 << sigma;
        let M = a.chunks(num_columns);
        M.zip(L)
            .map(|(row, l)| row.iter().zip(R).map(|(a, r)| l * a * r).sum::<F>())
            .sum()
    }
    /// This is the naive inner product. It is used to check the correctness of the `compute_LMR` method.
    fn compute_ab_inner_product(a: &[F], b_point: &[F]) -> F {
        let mut b = vec![Default::default(); a.len()];
        compute_evaluation_vector(&mut b, b_point);
        a.iter().zip(b.iter()).map(|(a, b)| a * b).sum()
    }
    fn check_L_R_with_random_a(b_point: &[F], L: &[F], R: &[F], sigma: usize) {
        let rng = &mut ark_std::test_rng();
        let a: Vec<_> = repeat_with(|| F::rand(rng))
            .take(1 << b_point.len())
            .collect();
        let LMR = compute_LMR(&a, L, R, sigma);
        let ab = compute_ab_inner_product(&a, b_point);
        assert_eq!(LMR, ab);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_is_0() {
        let b_point = vec![];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 2);

        let (L_vec, R_vec) = compute_l_r(&b_point, sigma, nu);
        assert_eq!(L_vec, vec![F::from(1), F::from(0), F::from(0), F::from(0)]);
        assert_eq!(R_vec, vec![F::from(1), F::from(0), F::from(0), F::from(0)]);

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_is_positive_and_less_than_sigma() {
        let b_point = vec![F::from(10)];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 2);

        let (L_vec, R_vec) = compute_l_r(&b_point, sigma, nu);
        assert_eq!(L_vec, vec![F::from(1), F::from(0), F::from(0), F::from(0)]);
        assert_eq!(
            R_vec,
            vec![F::from(1 - 10), F::from(10), F::from(0), F::from(0)]
        );

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_equals_sigma() {
        let b_point = vec![F::from(10), F::from(20)];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 2);

        let (L_vec, R_vec) = compute_l_r(&b_point, sigma, nu);
        assert_eq!(L_vec, vec![F::from(1), F::from(0), F::from(0), F::from(0)]);
        assert_eq!(
            R_vec,
            vec![
                F::from((1 - 10) * (1 - 20)),
                F::from(10 * (1 - 20)),
                F::from((1 - 10) * 20),
                F::from(10 * 20),
            ]
        );

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_is_more_than_sigma_but_less_than_2sigma() {
        let b_point = vec![F::from(10), F::from(20), F::from(30)];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 2);

        let (L_vec, R_vec) = compute_l_r(&b_point, sigma, nu);
        assert_eq!(
            L_vec,
            vec![F::from(1 - 30), F::from(30), F::from(0), F::from(0)]
        );
        assert_eq!(
            R_vec,
            vec![
                F::from((1 - 10) * (1 - 20)),
                F::from(10 * (1 - 20)),
                F::from((1 - 10) * 20),
                F::from(10 * 20),
            ]
        );

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_equals_2_sigma() {
        let b_point = vec![F::from(10), F::from(20), F::from(30), F::from(40)];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 2);

        let (L_vec, R_vec) = compute_l_r(&b_point, sigma, nu);
        assert_eq!(
            L_vec,
            vec![
                F::from((1 - 30) * (1 - 40)),
                F::from(30 * (1 - 40)),
                F::from((1 - 30) * 40),
                F::from(30 * 40),
            ]
        );
        assert_eq!(
            R_vec,
            vec![
                F::from((1 - 10) * (1 - 20)),
                F::from(10 * (1 - 20)),
                F::from((1 - 10) * 20),
                F::from(10 * 20),
            ]
        );

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);
    }

    #[test]
    fn we_can_compute_l_and_r_when_num_vars_is_more_than_2_sigma() {
        let b_point = vec![
            F::from(10),
            F::from(20),
            F::from(30),
            F::from(40),
            F::from(50),
        ];
        let sigma = 2;
        let nu = compute_nu(b_point.len(), sigma);

        assert_eq!(nu, 3);

        let (L_vec, R_vec) = compute_l_r(&b_point, sigma, nu);
        assert_eq!(
            L_vec,
            vec![
                F::from((1 - 30) * (1 - 40) * (1 - 50)),
                F::from(30 * (1 - 40) * (1 - 50)),
                F::from((1 - 30) * 40 * (1 - 50)),
                F::from(30 * 40 * (1 - 50)),
                F::from((1 - 30) * (1 - 40) * 50),
                F::from(30 * (1 - 40) * 50),
                F::from((1 - 30) * 40 * 50),
                F::from(30 * 40 * 50),
            ]
        );
        assert_eq!(
            R_vec,
            vec![
                F::from((1 - 10) * (1 - 20)),
                F::from(10 * (1 - 20)),
                F::from((1 - 10) * 20),
                F::from(10 * 20),
                F::from(0),
                F::from(0),
                F::from(0),
                F::from(0)
            ]
        );

        check_L_R_with_random_a(&b_point, &L_vec, &R_vec, sigma);
    }
}
