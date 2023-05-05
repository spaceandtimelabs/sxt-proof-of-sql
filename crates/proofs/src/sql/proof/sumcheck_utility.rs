use crate::base::polynomial::{to_ark_scalar, ArkScalar, DenseMultilinearExtension};
use crate::base::scalar::ToScalar;
use num_traits::Zero;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::rc::Rc;

/// Form a multilinear extension that can be added to a sumcheck polynomial.
///
/// Note: Currently our sumcheck algorithm doesn't support working on MLEs in-place
/// so we use this function to copy and convert the MLE into a form that works with
/// sumcheck.
#[tracing::instrument(
    name = "proofs.sql.proof.sumcheck_utility.make_sumcheck_term",
    level = "info",
    skip_all
)]
pub fn make_sumcheck_term<T: ToScalar + Sync>(
    num_vars: usize,
    values: &[T],
) -> Rc<DenseMultilinearExtension> {
    let n = 1 << num_vars;
    assert!(n >= values.len());
    let scalars = values
        .par_iter()
        .map(|val| to_ark_scalar(&val.to_scalar()))
        .chain(rayon::iter::repeatn(ArkScalar::zero(), n - values.len()))
        .collect();
    Rc::new(DenseMultilinearExtension::from_evaluations_vec(
        num_vars, scalars,
    ))
}
