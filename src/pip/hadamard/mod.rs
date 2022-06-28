mod sumcheck_polynomial;
#[cfg(test)]
mod sumcheck_polynomial_test;
pub use sumcheck_polynomial::make_sumcheck_polynomial;

mod proof;
#[cfg(test)]
mod proof_test;
pub use proof::HadamardProof;

mod evaluation_vector;
#[cfg(test)]
mod evaluation_vector_test;
pub use evaluation_vector::compute_evaluation_vector;
