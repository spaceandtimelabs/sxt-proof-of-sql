mod error;
pub use error::ProofError;

#[warn(missing_docs)]
/// Transcript protocol used to construct a proof.
mod transcript_protocol;
#[cfg(test)]
mod transcript_protocol_test;
pub use transcript_protocol::{MessageLabel, TranscriptProtocol};
