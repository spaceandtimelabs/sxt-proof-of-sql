use crate::base::polynomial::{ArkScalar, DenseMultilinearExtension};
use crate::base::scalar::{ToArkScalar, Zero};
use crate::base::slice_ops;
use rayon::iter::*;
use std::ffi::c_void;
use std::rc::Rc;

/// Interface for operating on multilinear extension's in-place
pub trait MultilinearExtension {
    /// Given an evaluation vector, compute the evaluation of the multilinear
    /// extension
    fn inner_product(&self, evaluation_vec: &[ArkScalar]) -> ArkScalar;

    /// multiply and add the MLE to a scalar vector
    fn mul_add(&self, res: &mut [ArkScalar], multiplier: &ArkScalar);

    /// convert the MLE to a form that can be used in sumcheck
    fn to_sumcheck_term(&self, num_vars: usize) -> Rc<DenseMultilinearExtension>;

    /// pointer to identify the slice forming the MLE
    fn id(&self) -> *const c_void;
}

/// Treat scalar convertible columns as a multilinear extensions
pub struct MultilinearExtensionImpl<'a, T: ToArkScalar> {
    data: &'a [T],
}

impl<'a, T: ToArkScalar> MultilinearExtensionImpl<'a, T> {
    /// Create MLE from slice
    pub fn new(data: &'a [T]) -> Self {
        Self { data }
    }
}

impl<'a, T: ToArkScalar + Sync> MultilinearExtension for MultilinearExtensionImpl<'a, T> {
    fn inner_product(&self, evaluation_vec: &[ArkScalar]) -> ArkScalar {
        slice_ops::inner_product(
            evaluation_vec,
            &slice_ops::slice_cast_with(self.data, ToArkScalar::to_ark_scalar),
        )
    }

    fn mul_add(&self, res: &mut [ArkScalar], multiplier: &ArkScalar) {
        slice_ops::mul_add_assign(
            res,
            *multiplier,
            &slice_ops::slice_cast_with(self.data, ToArkScalar::to_ark_scalar),
        );
    }

    fn to_sumcheck_term(&self, num_vars: usize) -> Rc<DenseMultilinearExtension> {
        let values = self.data;
        let n = 1 << num_vars;
        assert!(n >= values.len());
        let scalars = values
            .par_iter()
            .map(|val| val.to_ark_scalar())
            .chain(rayon::iter::repeatn(Zero::zero(), n - values.len()))
            .collect();
        Rc::new(DenseMultilinearExtension::from_evaluations_vec(
            num_vars, scalars,
        ))
    }

    fn id(&self) -> *const c_void {
        self.data.as_ptr() as *const c_void
    }
}
