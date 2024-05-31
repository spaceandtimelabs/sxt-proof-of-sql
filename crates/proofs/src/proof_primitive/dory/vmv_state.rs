use super::{DeferredGT, G1Affine, F};

/// The state of the verifier during the VMV evaluation proof verification.
/// See section 5 of https://eprint.iacr.org/2020/1274.pdf for details.
pub struct VMVVerifierState {
    /// The evaluation of the matrix. That is, y = LMR.
    pub(super) y: F,
    /// The commitment to the entire matrix. That is, T = <T_vec_prime, Gamma_2[nu]>.
    pub(super) T: DeferredGT,
    /// The left vector, L.
    pub(super) L_vec: Vec<F>,
    /// The right vector, R.
    pub(super) R_vec: Vec<F>,
    /// The power of 2 that determines the size of the matrix. That is, the matrix is 2^nu x 2^nu.
    pub(super) nu: usize,
}

/// The state of the prover during the VMV evaluation proof generation.
/// See section 5 of https://eprint.iacr.org/2020/1274.pdf for details.
pub struct VMVProverState {
    /// Evaluations of the columns of the matrix. That is, v = transpose(L) * M. In other words, v[j] = <L, M[_, j]> = sum_{i=0}^{2^nu} M[i,j] L[i].
    pub(super) v_vec: Vec<F>,
    /// Commitments to the rows of the matrix. That is T_vec_prime[i] = <M[i, _], Gamma_1[nu]> = sum_{j=0}^{2^nu} M[i,j] Gamma_1[nu][j].
    pub(super) T_vec_prime: Vec<G1Affine>,
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
            L_vec: self.L_vec,
            R_vec: self.R_vec,
            nu: self.nu,
        }
    }
}

/// A struct that holds the matrix and vectors for a vector-matrix-vector product. This is used for testing purposes.
#[cfg(test)]
#[allow(clippy::upper_case_acronyms)]
pub(super) struct VMV {
    pub(super) M: Vec<Vec<F>>,
    pub(super) L: Vec<F>,
    pub(super) R: Vec<F>,
    pub(super) nu: usize,
}

#[cfg(test)]
impl VMV {
    /// Create a new `VMV` from the matrix and vectors.
    pub(super) fn new(M: Vec<Vec<F>>, L: Vec<F>, R: Vec<F>, nu: usize) -> Self {
        Self { M, L, R, nu }
    }
    /// Calculate the VMV prover state from a vector-matrix-vector product and setup information.
    pub(super) fn calculate_prover_state(&self, setup: &super::ProverSetup) -> VMVProverState {
        use super::G1Projective;
        use ark_ec::VariableBaseMSM;
        let v_vec = Vec::from_iter((0..self.R.len()).map(|i| {
            self.L
                .iter()
                .zip(self.M.iter())
                .map(|(l, row)| row[i] * l)
                .sum()
        }));
        let T_vec_prime = Vec::from_iter(
            self.M
                .iter()
                .map(|row| G1Projective::msm_unchecked(setup.Gamma_1[self.nu], row).into()),
        );
        VMVProverState {
            v_vec,
            T_vec_prime,
            L_vec: self.L.clone(),
            R_vec: self.R.clone(),
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
        use super::rand_F_vecs;
        use ark_std::UniformRand;
        let size = 1 << nu;
        let M = (0..size)
            .map(|_| (0..size).map(|_| F::rand(rng)).collect())
            .collect();
        let (L, R) = rand_F_vecs(nu, rng);
        Self { M, L, R, nu }
    }
}
