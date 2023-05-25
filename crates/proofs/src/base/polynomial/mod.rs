mod ark_scalar;
#[cfg(test)]
mod ark_scalar_test;
pub use ark_scalar::ArkScalar;

pub type DenseMultilinearExtension = Vec<ArkScalar>;

mod composite_polynomial;
pub use composite_polynomial::{CompositePolynomial, CompositePolynomialInfo};
#[cfg(test)]
mod composite_polynomial_test;

mod interpolate;
#[cfg(test)]
mod interpolate_test;
pub use interpolate::interpolate_uni_poly;
