use ark_std::rc::Rc;
use ark_std::vec::Vec;
use ark_std::cmp::max;
use hashbrown::HashMap;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::dense_multilinear_extension::DenseMultilinearExtension;

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
pub struct CompositePolynomial {
    /// max number of multiplicands in each product
    pub max_multiplicands: usize,
    /// number of variables of the polynomial
    pub num_variables: usize,
    /// list of reference to products (as usize) of multilinear extension
    pub products: Vec<(Scalar, Vec<usize>)>,
    /// Stores multilinear extensions in which product multiplicand can refer to.
    pub flattened_ml_extensions: Vec<Rc<DenseMultilinearExtension>>,
    raw_pointers_lookup_table: HashMap<*const DenseMultilinearExtension, usize>,
}

/// Stores the number of variables and max number of multiplicands of the added polynomial used by the prover.
/// This data structures will is used as the verifier key.
pub struct CompositePolynomialInfo {
    /// max number of multiplicands in each product
    pub max_multiplicands: usize,
    /// number of variables of the polynomial
    pub num_variables: usize,
}

impl CompositePolynomial {
    /// Returns an empty polynomial
    pub fn new(num_variables: usize) -> Self {
        CompositePolynomial {
            max_multiplicands: 0,
            num_variables,
            products: Vec::new(),
            flattened_ml_extensions: Vec::new(),
            raw_pointers_lookup_table: HashMap::new(),
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
    pub fn add_product(
        &mut self,
        product: impl IntoIterator<Item = Rc<DenseMultilinearExtension>>,
        coefficient: Scalar,
    ) {
        let product: Vec<Rc<DenseMultilinearExtension>> = product.into_iter().collect();
        let mut indexed_product = Vec::with_capacity(product.len());
        assert!(product.len() > 0);
        self.max_multiplicands = max(self.max_multiplicands, product.len());
        for m in product {
            assert_eq!(
                m.ark_impl.num_vars, self.num_variables,
                "product has a multiplicand with wrong number of variables"
            );
            let m_ptr: *const DenseMultilinearExtension = Rc::as_ptr(&m);
            if let Some(index) = self.raw_pointers_lookup_table.get(&m_ptr) {
                indexed_product.push(*index)
            } else {
                let curr_index = self.flattened_ml_extensions.len();
                self.flattened_ml_extensions.push(m.clone());
                self.raw_pointers_lookup_table.insert(m_ptr, curr_index);
                indexed_product.push(curr_index);
            }
        }
        self.products.push((coefficient, indexed_product));
    }

    /// Evaluate the polynomial at point `point`
    pub fn evaluate(&self, point: &[Scalar]) -> Scalar {
        Scalar::from(123u64)
        // self.products
        //     .iter()
        //     .map(|(c, p)| {
        //         *c * p
        //             .iter()
        //             .map(|&i| self.flattened_ml_extensions[i].evaluate(point).unwrap())
        //             .product::<F>()
        //     })
        //     .sum()
    }
}
