use crate::base::polynomial::CompositePolynomial;
/**
 * Adopted from arkworks
 *
 * See `third_party/license/arkworks.LICENSE`
 */
use crate::base::scalar::Scalar;
use alloc::vec::Vec;

pub struct ProverState<S: Scalar> {
    /// sampled randomness given by the verifier
    pub randomness: Vec<S>,
    /// Stores the list of products that is meant to be added together. Each multiplicand is represented by
    /// the index in `flattened_ml_extensions`
    pub list_of_products: Vec<(S, Vec<usize>)>,
    /// Stores a list of multilinear extensions in which `self.list_of_products` points to
    pub flattened_ml_extensions: Vec<Vec<S>>,
    pub num_vars: usize,
    pub max_multiplicands: usize,
    pub round: usize,
}

impl<S: Scalar> ProverState<S> {
    #[tracing::instrument(name = "ProverState::create", level = "debug", skip_all)]
    pub fn create(polynomial: &CompositePolynomial<S>) -> Self {
        if polynomial.num_variables == 0 {
            panic!("Attempt to prove a constant.")
        }

        // create a deep copy of all unique MLExtensions
        let flattened_ml_extensions = polynomial
            .flattened_ml_extensions
            .iter()
            .map(|x| x.as_ref().clone())
            .collect();

        ProverState {
            randomness: Vec::with_capacity(polynomial.num_variables),
            list_of_products: polynomial.products.clone(),
            flattened_ml_extensions,
            num_vars: polynomial.num_variables,
            max_multiplicands: polynomial.max_multiplicands,
            round: 0,
        }
    }
}
