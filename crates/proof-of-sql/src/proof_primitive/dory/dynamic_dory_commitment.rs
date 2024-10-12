//! Module containing the `DoryCommitment` type and its implementation.
//!
//! While this can be used as a black box, it can be helpful to understand the underlying structure of the commitment.
//! Ultimately, the commitment is a commitment to a Matrix. This matrix is filled out from a column in the following fashion.
//!
//! We let `sigma` be a parameter that specifies the number of non-zero columns in the matrix.
//! More specifically, the number of non-zero columns is `2^sigma`.
//!
//! For an example, we will set `sigma=2` and thus, the number of columns is 4.
//! The column `[100,101,102,103,104,105,106,107,108,109,110,111,112,113,114,115]` with offset 9 is converted to the following matrix:
//! ```ignore
//!  0   0   0   0
//!  0   0   0   0
//!  0  100 101 102
//! 103 104 105 106
//! 107 108 109 110
//! 111 112 113 114
//! 115  0   0   0
//! ```
//! This matrix is then committed to using a matrix commitment.
//!
//! Note: the `VecCommitmentExt` trait requires using this offset when computing commitments.
//! This is to allow for updateability of the commitments as well as to allow for smart indexing/partitioning.

use super::{DoryScalar, ProverSetup, GT};
use crate::base::{
    commitment::{Commitment, CommittableColumn},
    impl_serde_for_ark_serde_checked,
};
use alloc::vec::Vec;
use ark_ec::pairing::PairingOutput;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use core::ops::Mul;
use derive_more::{AddAssign, Neg, Sub, SubAssign};
use num_traits::One;

#[derive(
    Debug,
    Sub,
    Eq,
    PartialEq,
    Neg,
    Copy,
    Clone,
    AddAssign,
    SubAssign,
    CanonicalSerialize,
    CanonicalDeserialize,
)]
/// The Dory commitment type.
pub struct DynamicDoryCommitment(pub(super) GT);

/// The default for GT is the the additive identity, but should be the multiplicative identity.
impl Default for DynamicDoryCommitment {
    fn default() -> Self {
        Self(PairingOutput(One::one()))
    }
}

// Traits required for `DoryCommitment` to impl `Commitment`.
impl_serde_for_ark_serde_checked!(DynamicDoryCommitment);
impl Mul<DynamicDoryCommitment> for DoryScalar {
    type Output = DynamicDoryCommitment;
    fn mul(self, rhs: DynamicDoryCommitment) -> Self::Output {
        DynamicDoryCommitment(rhs.0 * self.0)
    }
}
impl<'a> Mul<&'a DynamicDoryCommitment> for DoryScalar {
    type Output = DynamicDoryCommitment;
    fn mul(self, rhs: &'a DynamicDoryCommitment) -> Self::Output {
        DynamicDoryCommitment(rhs.0 * self.0)
    }
}
impl Commitment for DynamicDoryCommitment {
    type Scalar = DoryScalar;
    type PublicSetup<'a> = &'a ProverSetup<'a>;

    fn compute_commitments(
        committable_columns: &[CommittableColumn],
        offset: usize,
        setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self> {
        super::compute_dynamic_dory_commitments(committable_columns, offset, setup)
    }
}
