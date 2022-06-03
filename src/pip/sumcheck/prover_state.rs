use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::DenseMultilinearExtension;

#[allow(dead_code)]
pub struct ProverState {
    /// sampled randomness given by the verifier
    pub randomness: Vec<Scalar>,
    /// Stores the list of products that is meant to be added together. Each multiplicand is represented by
    /// the index in flattened_ml_extensions
    pub list_of_products: Vec<(Scalar, Vec<usize>)>,
    /// Stores a list of multilinear extensions in which `self.list_of_products` points to
    pub flattened_ml_extensions: Vec<DenseMultilinearExtension>,
    num_vars: usize,
    max_multiplicands: usize,
    round: usize,
}
