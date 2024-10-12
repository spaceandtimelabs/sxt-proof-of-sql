use super::{DeferredGT, G1Affine, F};
use alloc::vec::Vec;

/// The state of the verifier during the VMV evaluation proof verification.
/// See section 5 of <https://eprint.iacr.org/2020/1274.pdf> for details.
pub struct VMVVerifierState {
    /// The evaluation of the matrix. That is, y = LMR.
    pub(super) y: F,
    /// The commitment to the entire matrix. That is, `T = <T_vec_prime, Gamma_2[nu]>`.
    pub(super) T: DeferredGT,
    /// The left tensor, l.
    pub(super) l_tensor: Vec<F>,
    /// The right tensor, r.
    pub(super) r_tensor: Vec<F>,
    /// The power of 2 that determines the size of the matrix. That is, the matrix is 2^nu x 2^nu.
    pub(super) nu: usize,
}

/// The state of the prover during the VMV evaluation proof generation.
/// See section 5 of <https://eprint.iacr.org/2020/1274.pdf> for details.
pub struct VMVProverState {
    /// Evaluations of the columns of the matrix. That is, v = transpose(L) * M. In other words, v[j] = <L, M[_, j]> = sum_{i=0}^{2^nu} M[i,j] L[i].
    pub(super) v_vec: Vec<F>,
    /// Commitments to the rows of the matrix. That is `T_vec_prime[i] = <M[i, _], Gamma_1[nu]> = sum_{j=0}^{2^nu} M[i,j] Gamma_1[nu][j]`.
    pub(super) T_vec_prime: Vec<G1Affine>,
    /// The left tensor, l.
    #[cfg(test)]
    pub(super) l_tensor: Vec<F>,
    /// The right tensor, r.
    #[cfg(test)]
    pub(super) r_tensor: Vec<F>,
    /// The left vector, L.
    pub(super) L_vec: Vec<F>,
    /// The right vector, R.
    pub(super) R_vec: Vec<F>,
    /// The power of 2 that determines the size of the matrix. That is, the matrix is 2^nu x 2^nu.
    pub(super) nu: usize,
}

impl VMVProverState {
    /// Create a new `VMVVerifierState` from a `VMVProverState` and setup information.
    #[cfg(test)]
    pub(super) fn calculate_verifier_state(self, setup: &super::ProverSetup) -> VMVVerifierState {
        use ark_ec::pairing::Pairing;
        let T = Pairing::multi_pairing(self.T_vec_prime, setup.Gamma_2[self.nu]).into();
        let y = self
            .v_vec
            .iter()
            .zip(self.R_vec.iter())
            .map(|(v, r)| r * v)
            .sum();
        VMVVerifierState {
            y,
            T,
            l_tensor: self.l_tensor,
            r_tensor: self.r_tensor,
            nu: self.nu,
        }
    }
}

/// A struct that holds the matrix and vectors for a vector-matrix-vector product. This is used for testing purposes.
#[cfg(test)]
#[allow(clippy::upper_case_acronyms)]
pub(super) struct VMV {
    pub(super) M: Vec<Vec<F>>,
    pub(super) l_tensor: Vec<F>,
    pub(super) r_tensor: Vec<F>,
    pub(super) L: Vec<F>,
    pub(super) R: Vec<F>,
    pub(super) nu: usize,
}

#[cfg(test)]
impl VMV {
    /// Create a new `VMV` from the matrix and vectors.
    pub(super) fn new(M: Vec<Vec<F>>, L: Vec<F>, R: Vec<F>, nu: usize) -> Self {
        Self {
            M,
            L,
            R,
            l_tensor: vec![],
            r_tensor: vec![],
            nu,
        }
    }
    /// Create a new `VMV` from the matrix and tensors.
    pub(super) fn new_tensor(
        M: Vec<Vec<F>>,
        l_tensor: Vec<F>,
        r_tensor: Vec<F>,
        nu: usize,
    ) -> Self {
        use crate::base::polynomial::compute_evaluation_vector;
        use ark_ff::Fp;

        let mut L = vec![Fp::default(); 1 << l_tensor.len()];
        let mut R = vec![Fp::default(); 1 << r_tensor.len()];
        compute_evaluation_vector(&mut L, &l_tensor);
        compute_evaluation_vector(&mut R, &r_tensor);
        Self {
            M,
            l_tensor,
            r_tensor,
            L,
            R,
            nu,
        }
    }
    /// Calculate the VMV prover state from a vector-matrix-vector product and setup information.
    pub(super) fn calculate_prover_state(&self, setup: &super::ProverSetup) -> VMVProverState {
        use super::G1Projective;
        use ark_ec::VariableBaseMSM;
        let v_vec: Vec<_> = (0..self.R.len())
            .map(|i| {
                self.L
                    .iter()
                    .zip(self.M.iter())
                    .map(|(l, row)| row[i] * l)
                    .sum()
            })
            .collect();
        let T_vec_prime: Vec<_> = self
            .M
            .iter()
            .map(|row| G1Projective::msm_unchecked(setup.Gamma_1[self.nu], row).into())
            .collect();
        VMVProverState {
            v_vec,
            T_vec_prime,
            L_vec: self.L.clone(),
            R_vec: self.R.clone(),
            r_tensor: self.r_tensor.clone(),
            l_tensor: self.l_tensor.clone(),
            nu: self.nu,
        }
    }
    /// Calculate the VMV verifier state from a vector-matrix-vector product and setup information.
    pub(super) fn calculate_verifier_state(&self, setup: &super::ProverSetup) -> VMVVerifierState {
        self.calculate_prover_state(setup)
            .calculate_verifier_state(setup)
    }

    pub fn rand<R>(nu: usize, rng: &mut R) -> Self
    where
        R: ark_std::rand::Rng + ?Sized,
    {
        use super::rand_F_tensors;
        use ark_std::UniformRand;
        let size = 1 << nu;
        let M = (0..size)
            .map(|_| (0..size).map(|_| F::rand(rng)).collect())
            .collect();
        let (l_tensor, r_tensor) = rand_F_tensors(nu, rng);
        Self::new_tensor(M, l_tensor, r_tensor, nu)
    }
}
