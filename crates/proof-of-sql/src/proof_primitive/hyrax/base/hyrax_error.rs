use snafu::Snafu;

#[derive(Snafu, Debug)]
pub enum HyraxError {
    /// Hyrax does not currently support an offset for the generators.
    #[snafu(display("invalid generators offset: {offset}"))]
    InvalidGeneratorsOffset { offset: u64 },
    /// This error occurs when the proof fails to verify.
    #[snafu(display("verification error"))]
    VerificationError,
}
