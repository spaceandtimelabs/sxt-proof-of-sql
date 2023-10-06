use super::{MultilinearExtension, MultilinearExtensionImpl};

use crate::base::polynomial::{CompositePolynomial, DenseMultilinearExtension};
use crate::base::scalar::ArkScalar;
use num_traits::{One, Zero};
use std::collections::HashMap;
use std::ffi::c_void;
use std::rc::Rc;

// Build up a composite polynomial from individual MLE expressions
pub struct CompositePolynomialBuilder {
    num_sumcheck_variables: usize,
    fr_multiplicands_degree1: Vec<ArkScalar>,
    fr_multiplicands_rest: Vec<(ArkScalar, Vec<Rc<DenseMultilinearExtension>>)>,
    zerosum_multiplicands: Vec<(ArkScalar, Vec<Rc<DenseMultilinearExtension>>)>,
    fr: Rc<DenseMultilinearExtension>,
    mles: HashMap<*const c_void, Rc<DenseMultilinearExtension>>,
}

impl CompositePolynomialBuilder {
    pub fn new(num_sumcheck_variables: usize, fr: &[ArkScalar]) -> Self {
        assert!(1 << num_sumcheck_variables >= fr.len());
        Self {
            num_sumcheck_variables,
            fr_multiplicands_degree1: vec![Zero::zero(); fr.len()],
            fr_multiplicands_rest: vec![],
            zerosum_multiplicands: vec![],
            fr: MultilinearExtensionImpl::new(fr).to_sumcheck_term(num_sumcheck_variables),
            mles: HashMap::new(),
        }
    }

    /// Produce a polynomial term of the form
    ///    mult * f_r(X1, .., Xr) * term1(X1, ..., Xr) * ... * termK(X1, ..., Xr)
    /// where f_r is an MLE of random scalars
    pub fn produce_fr_multiplicand(
        &mut self,
        mult: &ArkScalar,
        terms: &[Box<dyn MultilinearExtension + '_>],
    ) {
        assert!(!terms.is_empty());
        if terms.len() == 1 {
            terms[0].mul_add(&mut self.fr_multiplicands_degree1, mult);
        } else {
            let multiplicand = self.create_multiplicand_with_deduplicated_mles(terms);
            self.fr_multiplicands_rest.push((*mult, multiplicand));
        }
    }
    /// Produce a polynomial term of the form
    ///    mult * term1(X1, ..., Xr) * ... * termK(X1, ..., Xr)
    pub fn produce_zerosum_multiplicand(
        &mut self,
        mult: &ArkScalar,
        terms: &[Box<dyn MultilinearExtension + '_>],
    ) {
        assert!(!terms.is_empty());
        let multiplicand = self.create_multiplicand_with_deduplicated_mles(terms);
        self.zerosum_multiplicands.push((*mult, multiplicand));
    }

    fn create_multiplicand_with_deduplicated_mles(
        &mut self,
        terms: &[Box<dyn MultilinearExtension + '_>],
    ) -> Vec<Rc<DenseMultilinearExtension>> {
        let mut terms_p = Vec::with_capacity(terms.len());
        for term in terms {
            let id = term.id();
            if let Some(term_p) = self.mles.get(&id) {
                terms_p.push(term_p.clone());
            } else {
                let term_p = term.to_sumcheck_term(self.num_sumcheck_variables);
                self.mles.insert(id, term_p.clone());
                terms_p.push(term_p);
            }
        }
        terms_p
    }

    /// Create a composite polynomial that is the sum of all of the
    /// produced MLE expressions
    pub fn make_composite_polynomial(&self) -> CompositePolynomial {
        let mut res = CompositePolynomial::new(self.num_sumcheck_variables);
        res.add_product(
            [
                self.fr.clone(),
                MultilinearExtensionImpl::new(&self.fr_multiplicands_degree1)
                    .to_sumcheck_term(self.num_sumcheck_variables),
            ],
            One::one(),
        );
        for (mult, terms) in self.fr_multiplicands_rest.iter() {
            let fr_iter = std::iter::once(self.fr.clone());
            let terms_iter = terms.iter().cloned();
            res.add_product(fr_iter.chain(terms_iter), *mult)
        }
        for (mult, terms) in self.zerosum_multiplicands.iter() {
            let terms_iter = terms.iter().cloned();
            res.add_product(terms_iter, *mult)
        }

        res.annotate_trace();

        res
    }
}
