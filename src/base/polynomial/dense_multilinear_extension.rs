use ark_poly;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::ark_scalar::ArkScalar;

pub struct DenseMultilinearExtension(ark_poly::DenseMultilinearExtension<ArkScalar>);

impl DenseMultilinearExtension {
    /// Construct a new polynomial from a list of evaluations where the index
    /// represents a point in {0,1}^`num_vars` in little endian form. For
    /// example, `0b1011` represents `P(1,1,0,1)`
    #[allow(unused_variables)]
    pub fn from_evaluations_slice(num_vars: usize, evaluations: &[Scalar]) {
    }
}
