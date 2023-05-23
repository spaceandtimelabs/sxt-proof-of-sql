/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::base::polynomial::Scalar;

use crate::base::polynomial::{ArkScalar, CompositePolynomial, DenseMultilinearExtension};

pub struct ProverState {
    /// sampled randomness given by the verifier
    pub randomness: Vec<Scalar>,
    /// Stores the list of products that is meant to be added together. Each multiplicand is represented by
    /// the index in flattened_ml_extensions
    pub list_of_products: Vec<(ArkScalar, Vec<usize>)>,
    /// Stores a list of multilinear extensions in which `self.list_of_products` points to
    pub flattened_ml_extensions: Vec<DenseMultilinearExtension>,
    pub num_vars: usize,
    pub max_multiplicands: usize,
    pub round: usize,
}

impl ProverState {
    #[tracing::instrument(
        name = "proofs.proof_primitive.sumcheck.prover_state.create",
        level = "info",
        skip_all
    )]
    pub fn create(polynomial: &CompositePolynomial) -> ProverState {
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
