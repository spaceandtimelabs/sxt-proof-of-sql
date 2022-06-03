use ark_std::vec::Vec;
use curve25519_dalek::scalar::Scalar;

/// Prover Message
#[allow(dead_code)]
pub struct ProverMessage {
    /// evaluations on P(0), P(1), P(2), ...
    pub(crate) evaluations: Vec<Scalar>,
}

/// Verifier Message
pub struct VerifierMessage {
    /// randomness sampled by verifier
    pub randomness: Scalar,
}
