use ark_ff::Field;
use ark_std::vec::Vec;

/// Prover Message
#[derive(Clone)]
pub struct ProverMessage<F: Field> {
    /// evaluations on P(0), P(1), P(2), ...
    pub(crate) evaluations: Vec<F>,
}
