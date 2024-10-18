use crate::base::{database::Column, if_rayon, scalar::Scalar, slice_ops};
use alloc::{rc::Rc, vec::Vec};
use core::ffi::c_void;
use num_traits::Zero;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

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

    #[cfg(test)]
    /// Given an evaluation point, compute the evaluation of the multilinear
    /// extension. This is inefficient and should only be used for testing.
    fn evaluate_at_point(&self, evaluation_point: &[S]) -> S {
        let mut evaluation_vec = vec![Default::default(); 1 << evaluation_point.len()];
        super::compute_evaluation_vector(&mut evaluation_vec, evaluation_point);
        self.inner_product(&evaluation_vec)
    }
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
        let scalars = if_rayon!(values.par_iter(), values.iter())
            .map(Into::into)
            .chain(if_rayon!(
                rayon::iter::repeatn(Zero::zero(), n - values.len()),
                itertools::repeat_n(Zero::zero(), n - values.len())
            ))
            .collect();
        Rc::new(scalars)
    }

    fn id(&self) -> *const c_void {
        self.as_ptr().cast::<c_void>()
    }
}

/// TODO: add docs
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

impl<S: Scalar> MultilinearExtension<S> for &Column<'_, S> {
    fn inner_product(&self, evaluation_vec: &[S]) -> S {
        match self {
            Column::Boolean(_, c) => c.inner_product(evaluation_vec),
            Column::Scalar(_, c) | Column::VarChar(_, (_, c)) | Column::Decimal75(.., c) => {
                c.inner_product(evaluation_vec)
            }
            Column::TinyInt(_, c) => c.inner_product(evaluation_vec),
            Column::SmallInt(_, c) => c.inner_product(evaluation_vec),
            Column::Int(_, c) => c.inner_product(evaluation_vec),
            Column::BigInt(_, c) | Column::TimestampTZ(.., c) => c.inner_product(evaluation_vec),
            Column::Int128(_, c) => c.inner_product(evaluation_vec),
        }
    }

    fn mul_add(&self, res: &mut [S], multiplier: &S) {
        match self {
            Column::Boolean(_, c) => c.mul_add(res, multiplier),
            Column::Scalar(_, c) | Column::VarChar(_, (_, c)) | Column::Decimal75(.., c) => {
                c.mul_add(res, multiplier);
            }
            Column::TinyInt(_, c) => c.mul_add(res, multiplier),
            Column::SmallInt(_, c) => c.mul_add(res, multiplier),
            Column::Int(_, c) => c.mul_add(res, multiplier),
            Column::BigInt(_, c) | Column::TimestampTZ(.., c) => c.mul_add(res, multiplier),
            Column::Int128(_, c) => c.mul_add(res, multiplier),
        }
    }

    fn to_sumcheck_term(&self, num_vars: usize) -> Rc<Vec<S>> {
        match self {
            Column::Boolean(_, c) => c.to_sumcheck_term(num_vars),
            Column::Scalar(_, c) | Column::VarChar(_, (_, c)) | Column::Decimal75(.., c) => {
                c.to_sumcheck_term(num_vars)
            }
            Column::TinyInt(_, c) => c.to_sumcheck_term(num_vars),
            Column::SmallInt(_, c) => c.to_sumcheck_term(num_vars),
            Column::Int(_, c) => c.to_sumcheck_term(num_vars),
            Column::BigInt(_, c) | Column::TimestampTZ(.., c) => c.to_sumcheck_term(num_vars),
            Column::Int128(_, c) => c.to_sumcheck_term(num_vars),
        }
    }

    fn id(&self) -> *const c_void {
        match self {
            Column::Boolean(_, c) => MultilinearExtension::<S>::id(c),
            Column::Scalar(_, c) | Column::VarChar(_, (_, c)) | Column::Decimal75(_, _, _, c) => {
                MultilinearExtension::<S>::id(c)
            }
            Column::TinyInt(_, c) => MultilinearExtension::<S>::id(c),
            Column::SmallInt(_, c) => MultilinearExtension::<S>::id(c),
            Column::Int(_, c) => MultilinearExtension::<S>::id(c),
            Column::BigInt(_, c) | Column::TimestampTZ(_, _, _, c) => {
                MultilinearExtension::<S>::id(c)
            }
            Column::Int128(_, c) => MultilinearExtension::<S>::id(c),
        }
    }
}

impl<S: Scalar> MultilinearExtension<S> for Column<'_, S> {
    fn inner_product(&self, evaluation_vec: &[S]) -> S {
        (&self).inner_product(evaluation_vec)
    }

    fn mul_add(&self, res: &mut [S], multiplier: &S) {
        (&self).mul_add(res, multiplier);
    }

    fn to_sumcheck_term(&self, num_vars: usize) -> Rc<Vec<S>> {
        (&self).to_sumcheck_term(num_vars)
    }

    fn id(&self) -> *const c_void {
        (&self).id()
    }
}
