//! Contains the transcript protocol used to construct a proof,
//! as well as an error type which can occur when verification fails.
mod error;
pub use error::{PlaceholderError, PlaceholderResult, ProofError, ProofSizeMismatch};

/// Contains an extension trait for `merlin::Transcript`, which is used to construct a proof.
#[cfg(any(test, feature = "blitzar"))]
mod merlin_transcript_core;

mod transcript;
pub use transcript::Transcript;

mod transcript_core;
#[cfg(test)]
mod transcript_core_test;

mod keccak256_transcript;
pub use keccak256_transcript::Keccak256Transcript;
