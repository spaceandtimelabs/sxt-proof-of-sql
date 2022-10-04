mod error;
pub use error::{IntoProofResult, ProofError, ProofResult};

mod transcript_protocol;
#[cfg(test)]
mod transcript_protocol_test;
pub use transcript_protocol::{MessageLabel, TranscriptProtocol};
