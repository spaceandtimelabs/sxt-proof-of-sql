use snafu::Snafu;

#[derive(Snafu, Debug)]
/// These errors occur when a proof failed to verify.
pub enum ProofError {
    #[snafu(display("Verification error: {error}"))]
    /// This error occurs when a proof failed to verify.
    VerificationError { error: &'static str },
    /// Unsupported error
    #[snafu(display("Unsupported error: {error}"))]
    UnsupportedError { error: &'static str },
}
