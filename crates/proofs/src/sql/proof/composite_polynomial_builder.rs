use super::{make_sumcheck_term, MultilinearExtension};

use crate::base::polynomial::{CompositePolynomial, DenseMultilinearExtension};
use curve25519_dalek::scalar::Scalar;
use std::collections::HashMap;
use std::ffi::c_void;
use std::rc::Rc;

// Build up a composite polynomial from individual MLE expressions
pub struct CompositePolynomialBuilder {
    num_sumcheck_variables: usize,
    fr_multiplicands_degree1: Vec<Scalar>,
    fr_multiplicands_rest: Vec<(Scalar, Vec<Rc<DenseMultilinearExtension>>)>,
    fr: Rc<DenseMultilinearExtension>,
    mles: HashMap<*const c_void, Rc<DenseMultilinearExtension>>,
}

impl CompositePolynomialBuilder {
    pub fn new(num_sumcheck_variables: usize, fr: &[Scalar]) -> Self {
        assert!(1 << num_sumcheck_variables >= fr.len());
        Self {
            num_sumcheck_variables,
            fr_multiplicands_degree1: vec![Scalar::zero(); fr.len()],
            fr_multiplicands_rest: vec![],
            fr: make_sumcheck_term(num_sumcheck_variables, fr),
            mles: HashMap::new(),
        }
    }

    /// Produce a polynomial term of the form
    ///    mult * f_r(X1, .., Xr) * term1(X1, ..., Xr) * ... * termK(X1, ..., Xr)
    /// where f_r is an MLE of random scalars
    pub fn produce_fr_multiplicand(
        &mut self,
        mult: &Scalar,
        terms: &[Box<dyn MultilinearExtension + '_>],
    ) {
        assert!(!terms.is_empty());
        if terms.len() == 1 {
            terms[0].mul_add(&mut self.fr_multiplicands_degree1, mult);
            return;
        }
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
        self.fr_multiplicands_rest.push((*mult, terms_p));
    }

    /// Create a composite polynomial that is the sum of all of the
    /// produced MLE expressions
    pub fn make_composite_polynomial(&self) -> CompositePolynomial {
        let mut res = CompositePolynomial::new(self.num_sumcheck_variables);
        res.add_product(
            [
                self.fr.clone(),
                make_sumcheck_term(self.num_sumcheck_variables, &self.fr_multiplicands_degree1),
            ],
            Scalar::one(),
        );
        for (mult, terms) in self.fr_multiplicands_rest.iter() {
            let fr_iter = std::iter::once(self.fr.clone());
            let terms_iter = terms.iter().cloned();
            res.add_product(fr_iter.chain(terms_iter), *mult)
        }
        res
    }
}
