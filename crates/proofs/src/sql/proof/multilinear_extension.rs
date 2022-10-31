use crate::base::scalar::IntoScalar;
use curve25519_dalek::scalar::Scalar;

/// Interface for operating on multilinear extension's in-place
pub trait MultilinearExtension {
    /// Given an evaluation vector, compute the evaluation of the multilinear
    /// extension
    fn evaluate(&self, evaluation_vec: &[Scalar]) -> Scalar;

    /// multiply and add the MLE to a scalar vector
    fn mul_add(&self, res: &mut [Scalar], multiplier: &Scalar);
}

/// Treat scalar convertible columns as a multilinear extensions
pub struct MultilinearExtensionImpl<'a, T: IntoScalar> {
    data: &'a [T],
}

impl<'a, T: IntoScalar> MultilinearExtensionImpl<'a, T> {
    /// Create MLE from slice
    pub fn new(data: &'a [T]) -> Self {
        Self { data }
    }
}

impl<'a, T: IntoScalar> MultilinearExtension for MultilinearExtensionImpl<'a, T> {
    fn evaluate(&self, evaluation_vec: &[Scalar]) -> Scalar {
        let mut res = Scalar::zero();
        for (xi, yi) in self.data.iter().zip(evaluation_vec.iter()) {
            res += xi.into_scalar() * yi;
        }
        res
    }

    fn mul_add(&self, res: &mut [Scalar], multiplier: &Scalar) {
        assert!(res.len() >= self.data.len());
        for (res_i, data_i) in res.iter_mut().zip(self.data) {
            *res_i += multiplier * data_i.into_scalar();
        }
    }
}
