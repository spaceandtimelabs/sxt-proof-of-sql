use crate::base::polynomial::{CompositePolynomial, DenseMultilinearExtension};
use curve25519_dalek::scalar::Scalar;
use std::rc::Rc;

/// A polynomial that sums to zero across binary values and can be aggregated
/// into a single sumcheck polynomial
pub struct SumcheckSubpolynomial {
    terms: Vec<(Scalar, Vec<Rc<DenseMultilinearExtension>>)>,
}

impl SumcheckSubpolynomial {
    /// Form subpolynomial from a sum of multilinear extension products
    pub fn new(terms: Vec<(Scalar, Vec<Rc<DenseMultilinearExtension>>)>) -> Self {
        SumcheckSubpolynomial { terms }
    }

    /// Multiply and add the subpolynomial to a compositie polynomial
    pub fn mul_add(
        &self,
        poly: &mut CompositePolynomial,
        fr: Rc<DenseMultilinearExtension>,
        group_multiplier: Scalar,
    ) {
        for (multiplier, mles) in self.terms.iter() {
            let mut term = Vec::with_capacity(mles.len() + 1);
            for mle in mles.iter() {
                term.push(mle.clone());
            }
            term.push(fr.clone());
            poly.add_product(term, multiplier * group_multiplier);
        }
    }
}
