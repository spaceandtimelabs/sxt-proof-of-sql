//! Dory is the commitment scheme described in https://eprint.iacr.org/2020/1274.pdf.
//!
//! This module contains the implementation of the Dory inner product argument for the BLS12-381 curve.
//!
//! Note:
//! We use nu = m and k = m-i or m-j.
//! This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.

#![warn(missing_docs)]
// This is so that the naming in the code more closely matches the naming in the paper, since the paper used both capital and non-capital letters.
#![allow(non_snake_case)]

use ark_bls12_381::{Fr as F, G1Projective as G1, G2Projective as G2};
type GT = ark_ec::pairing::PairingOutput<ark_bls12_381::Bls12_381>;

#[cfg(test)]
mod rand_util;
#[cfg(test)]
use rand_util::{rand_G_vecs, test_rng};

mod dory_messages;
pub use dory_messages::DoryMessages;
#[cfg(test)]
mod dory_messages_test;

mod setup;
pub use setup::{ProverSetup, VerifierSetup};
#[cfg(test)]
mod setup_test;

mod state;
pub use state::{ProverState, VerifierState};
#[cfg(test)]
mod state_test;
