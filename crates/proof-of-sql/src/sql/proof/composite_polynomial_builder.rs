use crate::base::{
    if_rayon,
    map::IndexMap,
    polynomial::{CompositePolynomial, MultilinearExtension},
    scalar::Scalar,
};
use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
use core::{ffi::c_void, iter};
use num_traits::{One, Zero};
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

// Build up a composite polynomial from individual MLE expressions
pub struct CompositePolynomialBuilder<S: Scalar> {
    num_sumcheck_variables: usize,
    fr_multiplicands_degree1: Vec<S>,
    fr_multiplicands_rest: Vec<(S, Vec<Rc<Vec<S>>>)>,
    zerosum_multiplicands: Vec<(S, Vec<Rc<Vec<S>>>)>,
    fr: Rc<Vec<S>>,
    mles: IndexMap<*const c_void, Rc<Vec<S>>>,
}

impl<S: Scalar> CompositePolynomialBuilder<S> {
    #[allow(clippy::missing_panics_doc, reason = "The assertion ensures that the length of 'fr' does not exceed the allowable range based on 'num_sumcheck_variables', making the panic clear from context.")]
    pub fn new(num_sumcheck_variables: usize, fr: &[S]) -> Self {
        assert!(1 << num_sumcheck_variables >= fr.len());
        Self {
            num_sumcheck_variables,
            fr_multiplicands_degree1: vec![Zero::zero(); fr.len()],
            fr_multiplicands_rest: vec![],
            zerosum_multiplicands: vec![],
            fr: fr.to_sumcheck_term(num_sumcheck_variables),
            mles: IndexMap::default(),
        }
    }

    /// Produce a polynomial term of the form
    ///    mult * f_r(X1, .., Xr) * term1(X1, ..., Xr) * ... * termK(X1, ..., Xr)
    /// where f_r is an MLE of random scalars
    pub fn produce_fr_multiplicand(
        &mut self,
        mult: &S,
        terms: &[Box<dyn MultilinearExtension<S> + '_>],
    ) {
        if terms.is_empty() {
            if_rayon!(
                self.fr_multiplicands_degree1
                    .par_iter_mut()
                    .with_min_len(crate::base::slice_ops::MIN_RAYON_LEN),
                self.fr_multiplicands_degree1.iter_mut()
            )
            .for_each(|val| *val += *mult);
        } else if terms.len() == 1 {
            terms[0].mul_add(&mut self.fr_multiplicands_degree1, mult);
        } else {
            let multiplicand = self.create_multiplicand_with_deduplicated_mles(terms);
            self.fr_multiplicands_rest.push((*mult, multiplicand));
        }
    }
    /// Produce a polynomial term of the form
    ///    mult * term1(X1, ..., Xr) * ... * termK(X1, ..., Xr)
    #[allow(clippy::missing_panics_doc, reason = "The assertion guarantees that terms are not empty, which is inherently clear from the context of this function.")]
    pub fn produce_zerosum_multiplicand(
        &mut self,
        mult: &S,
        terms: &[Box<dyn MultilinearExtension<S> + '_>],
    ) {
        // There is a more efficient way of handling constant zerosum terms,
        // since we know the sum will be constant * length, so this assertion should be here.
        assert!(!terms.is_empty());
        let multiplicand = self.create_multiplicand_with_deduplicated_mles(terms);
        self.zerosum_multiplicands.push((*mult, multiplicand));
    }

    fn create_multiplicand_with_deduplicated_mles(
        &mut self,
        terms: &[Box<dyn MultilinearExtension<S> + '_>],
    ) -> Vec<Rc<Vec<S>>> {
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
    pub fn make_composite_polynomial(&self) -> CompositePolynomial<S> {
        let mut res = CompositePolynomial::new(self.num_sumcheck_variables);
        res.add_product(
            [
                self.fr.clone(),
                (&self.fr_multiplicands_degree1).to_sumcheck_term(self.num_sumcheck_variables),
            ],
            One::one(),
        );
        for (mult, terms) in &self.fr_multiplicands_rest {
            let fr_iter = iter::once(self.fr.clone());
            let terms_iter = terms.iter().cloned();
            res.add_product(fr_iter.chain(terms_iter), *mult);
        }
        for (mult, terms) in &self.zerosum_multiplicands {
            let terms_iter = terms.iter().cloned();
            res.add_product(terms_iter, *mult);
        }

        res.annotate_trace();

        res
    }
}
