//! Implementation of `HyperKZG` PCS for usage with proof-of-sql.
//!
//! The prover side of this implementation simply wraps around nova's hyper-kzg implementation.
//!
//! While the `Commitment` for this commitment scheme is always available, the corresponding
//! `CommitmentEvaluationProof` is gated behind the `hyperkzg_proof` feature flag.
//! This is done to preserve `no_std` compatibility for `no_std` commitment generation apps.

mod scalar;
pub use scalar::BNScalar;

mod commitment;
pub use commitment::HyperKZGCommitment;

#[cfg(feature = "hyperkzg_proof")]
mod nova_commitment;

#[cfg(feature = "hyperkzg_proof")]
mod nova_engine;
#[cfg(feature = "hyperkzg_proof")]
pub use nova_engine::{nova_commitment_key_to_hyperkzg_public_setup, HyperKZGEngine};

#[cfg(feature = "hyperkzg_proof")]
mod commitment_evaluation_proof;
#[cfg(feature = "hyperkzg_proof")]
pub use commitment_evaluation_proof::HyperKZGCommitmentEvaluationProof;
