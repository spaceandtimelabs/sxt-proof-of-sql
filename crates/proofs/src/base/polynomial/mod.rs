pub type DenseMultilinearExtension = Vec<crate::base::scalar::ArkScalar>;

mod composite_polynomial;
pub use composite_polynomial::{CompositePolynomial, CompositePolynomialInfo};
#[cfg(test)]
mod composite_polynomial_test;

mod interpolate;
#[cfg(test)]
mod interpolate_test;
pub use interpolate::interpolate_uni_poly;
