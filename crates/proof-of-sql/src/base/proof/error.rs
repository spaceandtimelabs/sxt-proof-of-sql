use thiserror::Error;

#[derive(Error, Debug)]
/// These errors occur when a proof failed to verify.
pub enum ProofError {
    #[error("Verification error: {error}")]
    /// This error occurs when a proof failed to verify.
    VerificationError { error: &'static str },
}
