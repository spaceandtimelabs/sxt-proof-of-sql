use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProofError {
    /// This error occurs when a proof failed to verify.
    #[error("Verification error")]
    VerificationError,
}
