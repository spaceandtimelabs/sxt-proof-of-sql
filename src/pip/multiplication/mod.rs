mod sumcheck_polynomial;
pub use sumcheck_polynomial::make_sumcheck_polynomial;
#[cfg(test)]
mod sumcheck_polynomial_test;

mod proof;
pub use proof::MultiplicationProof;
#[cfg(test)]
mod proof_test;
