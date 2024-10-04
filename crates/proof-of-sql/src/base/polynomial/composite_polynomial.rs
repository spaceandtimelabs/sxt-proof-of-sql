use crate::base::{map::IndexMap, scalar::Scalar};
use alloc::{rc::Rc, vec::Vec};
/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use core::cmp::max;
#[cfg(test)]
use core::iter;
#[cfg(test)]
use itertools::Itertools;

/// Stores a list of products of `DenseMultilinearExtension` that is meant to be added together.
///
/// The polynomial is represented by a list of products of polynomials along with its coefficient that is meant to be added together.
///
/// This data structure of the polynomial is a list of list of `(coefficient, DenseMultilinearExtension)`.
/// * Number of products n = `self.products.len()`,
/// * Number of multiplicands of ith product m_i = `self.products[i].1.len()`,
/// * Coefficient of ith product c_i = `self.products[i].0`
///
/// The resulting polynomial is
///
/// $$\sum_{i=0}^{n}C_i\cdot\prod_{j=0}^{m_i}P_{ij}$$
///
/// The result polynomial is used as the prover key.
#[derive(Clone, Debug)]
pub struct CompositePolynomial<S: Scalar> {
    /// max number of multiplicands in each product
    pub max_multiplicands: usize,
    /// number of variables of the polynomial
    pub num_variables: usize,
    /// list of reference to products (as usize) of multilinear extension
    pub products: Vec<(S, Vec<usize>)>,
    /// Stores multilinear extensions in which product multiplicand can refer to.
    pub flattened_ml_extensions: Vec<Rc<Vec<S>>>,
    raw_pointers_lookup_table: IndexMap<*const Vec<S>, usize>,
}

/// Stores the number of variables and max number of multiplicands of the added polynomial used by the prover.
/// This data structures will is used as the verifier key.
#[derive(Clone, Debug)]
pub struct CompositePolynomialInfo {
    /// max number of multiplicands in each product
    pub max_multiplicands: usize,
    /// number of variables of the polynomial
    pub num_variables: usize,
}

impl<S: Scalar> CompositePolynomial<S> {
    /// Returns an empty polynomial
    pub fn new(num_variables: usize) -> Self {
        CompositePolynomial {
            max_multiplicands: 0,
            num_variables,
            products: Vec::new(),
            flattened_ml_extensions: Vec::new(),
            raw_pointers_lookup_table: IndexMap::default(),
        }
    }

    /// Extract the max number of multiplicands and number of variables of the list of products.
    pub fn info(&self) -> CompositePolynomialInfo {
        CompositePolynomialInfo {
            max_multiplicands: self.max_multiplicands,
            num_variables: self.num_variables,
        }
    }

    /// Add a list of multilinear extensions that is meant to be multiplied together.
    /// The resulting polynomial will be multiplied by the scalar `coefficient`.
    pub fn add_product(&mut self, product: impl IntoIterator<Item = Rc<Vec<S>>>, coefficient: S) {
        let product: Vec<Rc<Vec<S>>> = product.into_iter().collect();
        let mut indexed_product = Vec::with_capacity(product.len());
        assert!(!product.is_empty());
        self.max_multiplicands = max(self.max_multiplicands, product.len());
        for m in product {
            let m_ptr: *const Vec<S> = Rc::as_ptr(&m);
            if let Some(index) = self.raw_pointers_lookup_table.get(&m_ptr) {
                indexed_product.push(*index);
            } else {
                let curr_index = self.flattened_ml_extensions.len();
                self.flattened_ml_extensions.push(m.clone());
                self.raw_pointers_lookup_table.insert(m_ptr, curr_index);
                indexed_product.push(curr_index);
            }
        }
        self.products.push((coefficient, indexed_product));
    }
    /// Generate random `CompositePolnomial`.
    #[cfg(test)]
    pub fn rand(
        num_variables: usize,
        max_multiplicands: usize,
        multiplicands_length: impl IntoIterator<Item = usize>,
        products: impl IntoIterator<Item = impl IntoIterator<Item = usize>>,
        rng: &mut (impl ark_std::rand::Rng + ?Sized),
    ) -> Self {
        let mut result = CompositePolynomial::new(num_variables);
        result.max_multiplicands = max_multiplicands;
        result.products = products
            .into_iter()
            .map(|p| (S::rand(rng), p.into_iter().collect()))
            .collect();
        result.flattened_ml_extensions = multiplicands_length
            .into_iter()
            .map(|length| Rc::new(iter::repeat_with(|| S::rand(rng)).take(length).collect()))
            .collect();
        result
    }

    #[cfg(test)]
    /// Returns the product of the flattened_ml_extensions with referenced (as usize) by `terms` at the index `i`.
    fn term_product(&self, terms: &[usize], i: usize) -> S {
        terms
            .iter()
            .map(|&j| *self.flattened_ml_extensions[j].get(i).unwrap_or(&S::ZERO))
            .product::<S>()
    }
    /// Returns the sum of the evaluations of the `CompositePolynomial` on the boolean hypercube.
    #[cfg(test)]
    pub fn hypercube_sum(&self, length: usize) -> S {
        (0..length)
            .cartesian_product(&self.products)
            .map(|(i, (coeff, terms))| *coeff * self.term_product(terms, i))
            .sum::<S>()
    }

    /// Evaluate the polynomial at point `point`
    #[cfg(test)]
    pub fn evaluate(&self, point: &[S]) -> S {
        let mut evaluation_vector = vec![S::default(); 1 << self.num_variables];
        super::evaluation_vector::compute_evaluation_vector(&mut evaluation_vector, point);

        let result = self
            .products
            .iter()
            .map(|(c, p)| {
                *c * p
                    .iter()
                    .map(|&i| {
                        crate::base::slice_ops::inner_product(
                            &evaluation_vector,
                            &self.flattened_ml_extensions[i],
                        )
                    })
                    .product::<S>()
            })
            .sum();
        result
    }
    #[tracing::instrument(
        name = "CompositePolynomial::annotate_trace",
        level = "debug",
        skip_all
    )]
    pub fn annotate_trace(&self) {
        for i in 0..self.products.len() {
            tracing::info!(
                "Product #{:?}: {:#} * {:?}",
                i,
                self.products[i].0,
                self.products[i].1
            );
        }
    }
}
