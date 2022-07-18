use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum ProofError {
    /// This error occurs when a proof failed to verify.
    #[error("verification error")]
    VerificationError,
    /// This error occurs when the proof encoding is malformed.
    #[error("Proof data could not be parsed.")]
    FormatError,
    /// This error occurs when there is no proof
    #[error("Proof data could not be found.")]
    NoProofError,
}
