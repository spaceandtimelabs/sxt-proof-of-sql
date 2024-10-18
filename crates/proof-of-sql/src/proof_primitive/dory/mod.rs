//! Dory is the commitment scheme described in <https://eprint.iacr.org/2020/1274.pdf>.
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

// This is so that the naming in the code more closely matches the naming in the paper, since the paper used both capital and non-capital letters.
#![allow(non_snake_case)]

use ark_bls12_381::{Fr as F, G1Affine, G1Projective, G2Affine, G2Projective};
/// The pairing output of the BLS12-381 curve.
type GT = ark_ec::pairing::PairingOutput<ark_bls12_381::Bls12_381>;

#[cfg(any(test, feature = "test"))]
mod rand_util;
#[cfg(test)]
use rand_util::rand_F_tensors;
#[cfg(test)]
use rand_util::rand_G_vecs;
#[cfg(test)]
pub use rand_util::test_rng;

mod dory_messages;
pub(crate) use dory_messages::DoryMessages;
#[cfg(test)]
mod dory_messages_test;

mod setup;
pub use setup::{ProverSetup, VerifierSetup};
#[cfg(test)]
mod setup_test;

mod state;
pub(crate) use state::{ProverState, VerifierState};
#[cfg(test)]
mod state_test;

#[cfg(test)]
mod dory_reduce;
mod dory_reduce_helper;
mod scalar_product;

#[cfg(test)]
use dory_reduce::{dory_reduce_prove, dory_reduce_verify};
use scalar_product::{scalar_product_prove, scalar_product_verify};

#[cfg(test)]
mod dory_inner_product;
#[cfg(test)]
pub(crate) use dory_inner_product::{dory_inner_product_prove, dory_inner_product_verify};

#[cfg(test)]
mod dory_inner_product_test;

mod extended_state;
pub(crate) use extended_state::{ExtendedProverState, ExtendedVerifierState};
#[cfg(test)]
mod extended_state_test;

mod extended_dory_reduce;
mod extended_dory_reduce_helper;
mod fold_scalars;

pub(crate) use extended_dory_reduce::{extended_dory_reduce_prove, extended_dory_reduce_verify};
pub(crate) use fold_scalars::{fold_scalars_0_prove, fold_scalars_0_verify};

#[cfg(test)]
mod fold_scalars_test;

mod extended_dory_inner_product;
pub(crate) use extended_dory_inner_product::{
    extended_dory_inner_product_prove, extended_dory_inner_product_verify,
};

#[cfg(test)]
mod extended_dory_inner_product_test;

mod public_parameters;
pub use public_parameters::PublicParameters;

mod eval_vmv_re;
pub(crate) use eval_vmv_re::{eval_vmv_re_prove, eval_vmv_re_verify};

#[cfg(test)]
mod eval_vmv_re_test;

mod vmv_state;
#[cfg(test)]
use vmv_state::VMV;
pub(crate) use vmv_state::{VMVProverState, VMVVerifierState};

#[cfg(test)]
mod vmv_state_test;

mod dory_public_setup;
pub use dory_public_setup::{DoryProverPublicSetup, DoryVerifierPublicSetup};

mod dory_commitment;
#[cfg(test)]
mod dory_commitment_test;

#[cfg(not(feature = "blitzar"))]
mod dory_commitment_helper_cpu;
#[cfg(not(feature = "blitzar"))]
use dory_commitment_helper_cpu::compute_dory_commitments;
#[cfg(feature = "blitzar")]
mod dory_commitment_helper_gpu;
pub use dory_commitment::{DoryCommitment, DoryScalar};
#[cfg(feature = "blitzar")]
use dory_commitment_helper_gpu::compute_dory_commitments;
#[cfg(test)]
mod dory_compute_commitments_test;

mod dory_vmv_helper;
use dory_vmv_helper::{
    compute_L_R_vec, compute_T_vec_prime, compute_l_r_tensors, compute_nu, compute_v_vec,
};
mod build_vmv_state;
use build_vmv_state::{build_vmv_prover_state, build_vmv_verifier_state};

mod dory_commitment_evaluation_proof;
pub use dory_commitment_evaluation_proof::DoryEvaluationProof;
#[cfg(test)]
mod dory_commitment_evaluation_proof_test;

mod deferred_msm;
type DeferredGT = deferred_msm::DeferredMSM<GT, F>;
type DeferredG1 = deferred_msm::DeferredMSM<G1Affine, F>;
type DeferredG2 = deferred_msm::DeferredMSM<G2Affine, F>;

mod blitzar_metadata_table;
mod offset_to_bytes;
mod pack_scalars;
mod pairings;
mod transpose;

mod dynamic_build_vmv_state;
#[cfg(not(feature = "blitzar"))]
mod dynamic_dory_commitment_helper_cpu;
#[cfg(feature = "blitzar")]
mod dynamic_dory_commitment_helper_gpu;
mod dynamic_dory_helper;
mod dynamic_dory_standard_basis_helper;
mod dynamic_dory_structure;
#[cfg(not(feature = "blitzar"))]
use dynamic_dory_commitment_helper_cpu::compute_dynamic_dory_commitments;
#[cfg(feature = "blitzar")]
use dynamic_dory_commitment_helper_gpu::compute_dynamic_dory_commitments;
mod dynamic_dory_commitment;
mod dynamic_dory_commitment_evaluation_proof;
#[cfg(test)]
mod dynamic_dory_compute_commitments_test;
pub use dynamic_dory_commitment::DynamicDoryCommitment;
#[cfg(test)]
mod dynamic_dory_commitment_evaluation_proof_test;
pub use dynamic_dory_commitment_evaluation_proof::DynamicDoryEvaluationProof;
