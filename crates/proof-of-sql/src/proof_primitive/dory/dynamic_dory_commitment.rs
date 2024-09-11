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

use super::{DoryProverPublicSetup, GT};
use crate::base::{
    commitment::{Commitment, CommittableColumn},
    impl_serde_for_ark_serde_checked,
    scalar::{MontScalar, Scalar},
};
use ark_ec::pairing::PairingOutput;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use core::ops::Mul;
use derive_more::{AddAssign, Neg, Sub, SubAssign};
use num_traits::One;

/// The Dory scalar type. (alias for `MontScalar<ark_bls12_381::FrConfig>`)
pub type DoryScalar = MontScalar<ark_bls12_381::FrConfig>;

impl Scalar for DoryScalar {
    const MAX_SIGNED: Self = Self(ark_ff::MontFp!(
        "26217937587563095239723870254092982918845276250263818911301829349969290592256"
    ));
    const ZERO: Self = Self(ark_ff::MontFp!("0"));
    const ONE: Self = Self(ark_ff::MontFp!("1"));
    const TWO: Self = Self(ark_ff::MontFp!("2"));
}

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
pub struct DoryCommitment(pub(super) GT);

/// The default for GT is the the additive identity, but should be the multiplicative identity.
impl Default for DoryCommitment {
    fn default() -> Self {
        Self(PairingOutput(One::one()))
    }
}

// Traits required for `DoryCommitment` to impl `Commitment`.
impl_serde_for_ark_serde_checked!(DoryCommitment);
impl Mul<DoryCommitment> for DoryScalar {
    type Output = DoryCommitment;
    fn mul(self, rhs: DoryCommitment) -> Self::Output {
        DoryCommitment(rhs.0 * self.0)
    }
}
impl<'a> Mul<&'a DoryCommitment> for DoryScalar {
    type Output = DoryCommitment;
    fn mul(self, rhs: &'a DoryCommitment) -> Self::Output {
        DoryCommitment(rhs.0 * self.0)
    }
}
impl Commitment for DoryCommitment {
    type Scalar = DoryScalar;
    type PublicSetup<'a> = DoryProverPublicSetup<'a>;

    fn compute_commitments(
        commitments: &mut [Self],
        committable_columns: &[CommittableColumn],
        offset: usize,
        setup: &Self::PublicSetup<'_>,
    ) {
        assert_eq!(commitments.len(), committable_columns.len());
        let c = super::compute_dory_commitments(committable_columns, offset, setup);
        commitments.copy_from_slice(&c);
    }
}
