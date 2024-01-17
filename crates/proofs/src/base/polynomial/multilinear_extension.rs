use crate::base::{database::Column, scalar::Scalar, slice_ops};
use num_traits::Zero;
use rayon::iter::*;
use std::{ffi::c_void, rc::Rc};

/// Interface for operating on multilinear extension's in-place
pub trait MultilinearExtension<S: Scalar> {
    /// Given an evaluation vector, compute the evaluation of the multilinear
    /// extension
    fn inner_product(&self, evaluation_vec: &[S]) -> S;

    /// multiply and add the MLE to a scalar vector
    fn mul_add(&self, res: &mut [S], multiplier: &S);

    /// convert the MLE to a form that can be used in sumcheck
    fn to_sumcheck_term(&self, num_vars: usize) -> Rc<Vec<S>>;

    /// pointer to identify the slice forming the MLE
    fn id(&self) -> *const c_void;
}

impl<'a, T: Sync, S: Scalar> MultilinearExtension<S> for &'a [T]
where
    &'a T: Into<S>,
{
    fn inner_product(&self, evaluation_vec: &[S]) -> S {
        slice_ops::inner_product(evaluation_vec, &slice_ops::slice_cast(self))
    }

    fn mul_add(&self, res: &mut [S], multiplier: &S) {
        slice_ops::mul_add_assign(res, *multiplier, &slice_ops::slice_cast(self));
    }

    fn to_sumcheck_term(&self, num_vars: usize) -> Rc<Vec<S>> {
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
        fn inner_product(&self, evaluation_vec: &[S]) -> S {
            (&self[..]).inner_product(evaluation_vec)
        }

        fn mul_add(&self, res: &mut [S], multiplier: &S) {
            (&self[..]).mul_add(res, multiplier)
        }

        fn to_sumcheck_term(&self, num_vars: usize) -> Rc<Vec<S>> {
            (&self[..]).to_sumcheck_term(num_vars)
        }

        fn id(&self) -> *const c_void {
            (&self[..]).id()
        }
    };
}

impl<'a, T: Sync, S: Scalar> MultilinearExtension<S> for &'a Vec<T>
where
    &'a T: Into<S>,
{
    slice_like_mle_impl!();
}

impl<'a, T: Sync, const N: usize, S: Scalar> MultilinearExtension<S> for &'a [T; N]
where
    &'a T: Into<S>,
{
    slice_like_mle_impl!();
}

impl<S: Scalar> MultilinearExtension<S> for Column<'_, S> {
    fn inner_product(&self, evaluation_vec: &[S]) -> S {
        match self {
            Column::Scalar(c) => c.inner_product(evaluation_vec),
            Column::BigInt(c) => c.inner_product(evaluation_vec),
            Column::VarChar((_, c)) => c.inner_product(evaluation_vec),
            Column::Int128(c) => c.inner_product(evaluation_vec),
        }
    }

    fn mul_add(&self, res: &mut [S], multiplier: &S) {
        match self {
            Column::Scalar(c) => c.mul_add(res, multiplier),
            Column::BigInt(c) => c.mul_add(res, multiplier),
            Column::VarChar((_, c)) => c.mul_add(res, multiplier),
            Column::Int128(c) => c.mul_add(res, multiplier),
        }
    }

    fn to_sumcheck_term(&self, num_vars: usize) -> Rc<Vec<S>> {
        match self {
            Column::Scalar(c) => c.to_sumcheck_term(num_vars),
            Column::BigInt(c) => c.to_sumcheck_term(num_vars),
            Column::VarChar((_, c)) => c.to_sumcheck_term(num_vars),
            Column::Int128(c) => c.to_sumcheck_term(num_vars),
        }
    }

    fn id(&self) -> *const c_void {
        match self {
            Column::Scalar(c) => MultilinearExtension::<S>::id(c),
            Column::BigInt(c) => MultilinearExtension::<S>::id(c),
            Column::VarChar((_, c)) => MultilinearExtension::<S>::id(c),
            Column::Int128(c) => MultilinearExtension::<S>::id(c),
        }
    }
}
