use crate::base::{database::Column, scalar::ArkScalar, slice_ops};
use num_traits::Zero;
use rayon::iter::*;
use std::{ffi::c_void, rc::Rc};

/// Interface for operating on multilinear extension's in-place
pub trait MultilinearExtension {
    /// Given an evaluation vector, compute the evaluation of the multilinear
    /// extension
    fn inner_product(&self, evaluation_vec: &[ArkScalar]) -> ArkScalar;

    /// multiply and add the MLE to a scalar vector
    fn mul_add(&self, res: &mut [ArkScalar], multiplier: &ArkScalar);

    /// convert the MLE to a form that can be used in sumcheck
    fn to_sumcheck_term(&self, num_vars: usize) -> Rc<Vec<ArkScalar>>;

    /// pointer to identify the slice forming the MLE
    fn id(&self) -> *const c_void;
}

impl<'a, T: Sync> MultilinearExtension for &'a [T]
where
    &'a T: Into<ArkScalar>,
{
    fn inner_product(&self, evaluation_vec: &[ArkScalar]) -> ArkScalar {
        slice_ops::inner_product(evaluation_vec, &slice_ops::slice_cast(self))
    }

    fn mul_add(&self, res: &mut [ArkScalar], multiplier: &ArkScalar) {
        slice_ops::mul_add_assign(res, *multiplier, &slice_ops::slice_cast(self));
    }

    fn to_sumcheck_term(&self, num_vars: usize) -> Rc<Vec<ArkScalar>> {
        let values = self;
        let n = 1 << num_vars;
        assert!(n >= values.len());
        let scalars = values
            .par_iter()
            .map(|val| val.into())
            .chain(rayon::iter::repeatn(Zero::zero(), n - values.len()))
            .collect();
        Rc::new(scalars)
    }

    fn id(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }
}

macro_rules! slice_like_mle_impl {
    () => {
        fn inner_product(&self, evaluation_vec: &[ArkScalar]) -> ArkScalar {
            (&self[..]).inner_product(evaluation_vec)
        }

        fn mul_add(&self, res: &mut [ArkScalar], multiplier: &ArkScalar) {
            (&self[..]).mul_add(res, multiplier)
        }

        fn to_sumcheck_term(&self, num_vars: usize) -> Rc<Vec<ArkScalar>> {
            (&self[..]).to_sumcheck_term(num_vars)
        }

        fn id(&self) -> *const c_void {
            (&self[..]).id()
        }
    };
}

impl<'a, T: Sync> MultilinearExtension for &'a Vec<T>
where
    &'a T: Into<ArkScalar>,
{
    slice_like_mle_impl!();
}

impl<'a, T: Sync, const N: usize> MultilinearExtension for &'a [T; N]
where
    &'a T: Into<ArkScalar>,
{
    slice_like_mle_impl!();
}

impl MultilinearExtension for Column<'_, ArkScalar> {
    fn inner_product(&self, evaluation_vec: &[ArkScalar]) -> ArkScalar {
        match self {
            Column::Scalar(c) => c.inner_product(evaluation_vec),
            Column::BigInt(c) => c.inner_product(evaluation_vec),
            Column::VarChar((_, c)) => c.inner_product(evaluation_vec),
            Column::Int128(c) => c.inner_product(evaluation_vec),
        }
    }

    fn mul_add(&self, res: &mut [ArkScalar], multiplier: &ArkScalar) {
        match self {
            Column::Scalar(c) => c.mul_add(res, multiplier),
            Column::BigInt(c) => c.mul_add(res, multiplier),
            Column::VarChar((_, c)) => c.mul_add(res, multiplier),
            Column::Int128(c) => c.mul_add(res, multiplier),
        }
    }

    fn to_sumcheck_term(&self, num_vars: usize) -> Rc<Vec<ArkScalar>> {
        match self {
            Column::Scalar(c) => c.to_sumcheck_term(num_vars),
            Column::BigInt(c) => c.to_sumcheck_term(num_vars),
            Column::VarChar((_, c)) => c.to_sumcheck_term(num_vars),
            Column::Int128(c) => c.to_sumcheck_term(num_vars),
        }
    }

    fn id(&self) -> *const c_void {
        match self {
            Column::Scalar(c) => c.id(),
            Column::BigInt(c) => c.id(),
            Column::VarChar((_, c)) => c.id(),
            Column::Int128(c) => c.id(),
        }
    }
}
