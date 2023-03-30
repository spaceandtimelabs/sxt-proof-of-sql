use crate::base::scalar::ToScalar;
use curve25519_dalek::scalar::Scalar;
use rayon::iter::*;

/// Interface for operating on multilinear extension's in-place
pub trait MultilinearExtension {
    /// Given an evaluation vector, compute the evaluation of the multilinear
    /// extension
    fn evaluate(&self, evaluation_vec: &[Scalar]) -> Scalar;

    /// multiply and add the MLE to a scalar vector
    fn mul_add(&self, res: &mut [Scalar], multiplier: &Scalar);
}

/// Treat scalar convertible columns as a multilinear extensions
pub struct MultilinearExtensionImpl<'a, T: ToScalar> {
    data: &'a [T],
}

impl<'a, T: ToScalar> MultilinearExtensionImpl<'a, T> {
    /// Create MLE from slice
    pub fn new(data: &'a [T]) -> Self {
        Self { data }
    }
}

impl<'a, T: ToScalar + Sync> MultilinearExtension for MultilinearExtensionImpl<'a, T> {
    fn evaluate(&self, evaluation_vec: &[Scalar]) -> Scalar {
        self.data
            .par_iter()
            .zip(evaluation_vec)
            .map(|(xi, yi)| xi.to_scalar() * yi)
            .reduce(Scalar::zero, std::ops::Add::add)
    }

    fn mul_add(&self, res: &mut [Scalar], multiplier: &Scalar) {
        assert!(res.len() >= self.data.len());
        res.par_iter_mut()
            .zip(self.data)
            .for_each(|(res_i, data_i)| {
                *res_i += multiplier * data_i.to_scalar();
            })
    }
}
