//! Dory is the commitment scheme described in https://eprint.iacr.org/2020/1274.pdf.
//!
//! This module contains the implementation of the Dory inner product argument for the BLS12-381 curve.
//!
//! Note:
//! We use nu = m and k = m-i or m-j.
//! This indexing is more convenient for coding because lengths of the arrays used are typically 2^k rather than 2^i or 2^j.
//!
//! Note: from the paper:
//! > In our initial presentation of the protocols, and discussions of completeness
//! > and soundness, we will highlight that which is required only to achieve hiding
//! > in commitments and zero-knowledge in the protocols in blue.
//!
//! This implementation only implements the computational integrity component of Dory.
//! This can be extended in the future to achieve hiding, but that isn't needed for our initial use-case.

#![warn(missing_docs)]
// This is so that the naming in the code more closely matches the naming in the paper, since the paper used both capital and non-capital letters.
#![allow(non_snake_case)]

use ark_bls12_381::{Fr as F, G1Projective as G1, G2Projective as G2};
type GT = ark_ec::pairing::PairingOutput<ark_bls12_381::Bls12_381>;

#[cfg(test)]
mod rand_util;
#[cfg(test)]
use rand_util::{rand_F_vecs, rand_G_vecs, test_rng};

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

mod dory_reduce;
mod dory_reduce_helper;
mod scalar_product;

use dory_reduce::{dory_reduce_prove, dory_reduce_verify};
use scalar_product::{scalar_product_prove, scalar_product_verify};

mod dory_inner_product;
pub use dory_inner_product::{dory_inner_product_prove, dory_inner_product_verify};

#[cfg(test)]
mod dory_inner_product_test;

mod extended_state;
pub use extended_state::{ExtendedProverState, ExtendedVerifierState};
#[cfg(test)]
mod extended_state_test;

mod extended_dory_reduce;
mod extended_dory_reduce_helper;
mod fold_scalars;

pub use extended_dory_reduce::{extended_dory_reduce_prove, extended_dory_reduce_verify};
pub use fold_scalars::{fold_scalars_0_prove, fold_scalars_0_verify};

#[cfg(test)]
mod fold_scalars_test;

mod extended_dory_inner_product;
pub use extended_dory_inner_product::{
    extended_dory_inner_product_prove, extended_dory_inner_product_verify,
};

#[cfg(test)]
mod extended_dory_inner_product_test;

mod public_parameters;
pub use public_parameters::PublicParameters;
