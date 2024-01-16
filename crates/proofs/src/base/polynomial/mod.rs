mod composite_polynomial;
pub use composite_polynomial::{CompositePolynomial, CompositePolynomialInfo};
#[cfg(test)]
mod composite_polynomial_test;

mod interpolate;
#[cfg(test)]
mod interpolate_test;
pub use interpolate::interpolate_uni_poly;

#[warn(missing_docs)]
mod evaluation_vector;
pub use evaluation_vector::compute_evaluation_vector;
#[cfg(test)]
mod evaluation_vector_test;

#[warn(missing_docs)]
mod lagrange_basis_evaluation;
pub use lagrange_basis_evaluation::{
    compute_truncated_lagrange_basis_inner_product, compute_truncated_lagrange_basis_sum,
};
#[cfg(test)]
mod lagrange_basis_evaluation_test;
