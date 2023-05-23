//! This module defines the inverse function for various types.
//! This is useful for writing generic code.

use num_traits::Inv;

pub trait Inverse {
    fn inverse(&self) -> Self;
}
impl Inverse for crate::base::polynomial::ArkScalar {
    fn inverse(&self) -> Self {
        self.inv()
    }
}
impl Inverse for curve25519_dalek::scalar::Scalar {
    fn inverse(&self) -> Self {
        self.invert()
    }
}
