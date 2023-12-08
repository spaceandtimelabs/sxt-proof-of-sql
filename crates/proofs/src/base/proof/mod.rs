//! Contains the transcript protocol used to construct a proof,
//! as well as an error type which can occur when verification fails.
mod error;
pub use error::ProofError;

#[warn(missing_docs)]
/// Contains an extension trait for `merlin::Transcript`, which is used to construct a proof.
mod transcript_protocol;
#[cfg(test)]
mod transcript_protocol_test;
pub use transcript_protocol::{MessageLabel, TranscriptProtocol};
