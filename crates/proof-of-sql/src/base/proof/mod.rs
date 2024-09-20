//! Contains the transcript protocol used to construct a proof,
//! as well as an error type which can occur when verification fails.
#[cfg(feature = "std")]
mod error;
#[cfg(feature = "std")]
pub use error::ProofError;

/// Contains an extension trait for `merlin::Transcript`, which is used to construct a proof.
#[cfg(feature = "std")]
mod transcript_protocol;
#[cfg(test)]
#[cfg(feature = "std")]
mod transcript_protocol_test;
#[cfg(feature = "std")]
pub use transcript_protocol::{MessageLabel, TranscriptProtocol};

mod transcript;
pub use transcript::Transcript;

mod transcript_core;
#[cfg(test)]
mod transcript_core_test;

mod keccak256_transcript;
#[allow(unused_imports)]
pub use keccak256_transcript::Keccak256Transcript;
