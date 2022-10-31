use crate::base::polynomial::DenseMultilinearExtension;
use crate::base::scalar::IntoScalar;
use curve25519_dalek::scalar::Scalar;
use std::rc::Rc;

/// Form a multilinear extension that can be added to a sumcheck polynomial.
///
/// Note: Currently our sumcheck algorithm doesn't support working on MLEs in-place
/// so we use this function to copy and convert the MLE into a form that works with
/// sumcheck.
pub fn make_sumcheck_term<T: IntoScalar>(
    num_vars: usize,
    values: &[T],
) -> Rc<DenseMultilinearExtension> {
    let n = 1 << num_vars;
    assert!(n >= values.len());
    let mut scalars = Vec::with_capacity(n);
    for val in values.iter() {
        scalars.push(val.into_scalar());
    }
    for _ in values.len()..n {
        scalars.push(Scalar::zero());
    }
    Rc::new(DenseMultilinearExtension::from_evaluations_slice(
        num_vars, &scalars,
    ))
}
