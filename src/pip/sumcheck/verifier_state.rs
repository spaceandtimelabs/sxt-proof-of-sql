use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::CompositePolynomialInfo;

/// Verifier State
pub struct VerifierState {
    pub round: usize,
    pub nv: usize,
    pub max_multiplicands: usize,
    pub finished: bool,
    /// a list storing the univariate polynomial in evaluation form sent by the prover at each round
    pub polynomials_received: Vec<Vec<Scalar>>,
    /// a list storing the randomness sampled by the verifier at each round
    pub randomness: Vec<Scalar>,
}

pub fn init_verifier_state(index_info: CompositePolynomialInfo) -> VerifierState {
    VerifierState {
        round: 1,
        nv: index_info.num_variables,
        max_multiplicands: index_info.max_multiplicands,
        finished: false,
        polynomials_received: Vec::with_capacity(index_info.num_variables),
        randomness: Vec::with_capacity(index_info.num_variables),
    }
}
