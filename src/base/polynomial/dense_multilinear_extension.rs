use ark_poly;
use ark_poly::MultilinearExtension;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::ark_scalar::{ArkScalar, to_ark_scalar, from_ark_scalar};

pub struct DenseMultilinearExtension {
    pub ark_impl: ark_poly::DenseMultilinearExtension<ArkScalar>,
}

impl DenseMultilinearExtension {
    /// Construct a new polynomial from a list of evaluations where the index
    /// represents a point in {0,1}^`num_vars` in little endian form. For
    /// example, `0b1011` represents `P(1,1,0,1)`
    #[allow(unused_variables)]
    pub fn from_evaluations_slice(num_vars: usize, evaluations : &[Scalar]) -> DenseMultilinearExtension {
        let evaluations_p
            : Vec<ArkScalar> =
                  evaluations.iter().map(| x | to_ark_scalar(x)).collect();
        DenseMultilinearExtension{
            ark_impl: ark_poly::DenseMultilinearExtension::from_evaluations_vec(num_vars, evaluations_p),
        }
    }

    pub fn evaluate(&self, point: &[Scalar]) -> Option<Scalar> {
        if point.len() == self.ark_impl.num_vars {
            let point_p
                : Vec<ArkScalar> =
                      point.iter().map(| x | to_ark_scalar(x)).collect();
            let value = self.ark_impl.fix_variables(&point_p)[0];
            Some(from_ark_scalar(&value))
        } else {
            None
        }
    }
}
