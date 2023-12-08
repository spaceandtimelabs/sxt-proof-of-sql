use thiserror::Error;

#[derive(Error, Debug)]
/// These errors occur when a proof failed to verify.
pub enum ProofError {
    #[error("Verification error: {0}")]
    /// This error occurs when a proof failed to verify.
    VerificationError(&'static str),
}
