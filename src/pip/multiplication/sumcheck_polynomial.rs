use curve25519_dalek::scalar::Scalar;
use ark_std::rc::Rc;

use crate::base::polynomial::DenseMultilinearExtension;
use crate::base::polynomial::CompositePolynomial;

pub fn make_sumcheck_polynomial(
        num_vars: usize,
        a_vec: &[Scalar],
        b_vec: &[Scalar],
        ab_vec: &[Scalar],
        r_vec: &[Scalar],
        ) -> CompositePolynomial {
    let n = a_vec.len();
    assert_eq!(b_vec.len(), n);
    assert_eq!(ab_vec.len(), n);
    assert_eq!(r_vec.len(), n);

    let fa = Rc::new(DenseMultilinearExtension::from_evaluations_slice(num_vars, a_vec));
    let fb = Rc::new(DenseMultilinearExtension::from_evaluations_slice(num_vars, b_vec));
    let fab = Rc::new(DenseMultilinearExtension::from_evaluations_slice(num_vars, ab_vec));
    let fr = Rc::new(DenseMultilinearExtension::from_evaluations_slice(num_vars, r_vec));
    
    let mut p = CompositePolynomial::new(num_vars);

    let multiplier = Scalar::from(1u64);
    p.add_product([fr.clone(), fa, fb], multiplier);
    p.add_product([fr, fab], -multiplier);

    p
}
