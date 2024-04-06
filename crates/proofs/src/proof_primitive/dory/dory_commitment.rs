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
    commitment::{Commitment, CommittableColumn, NumColumnsMismatch, VecCommitmentExt},
    impl_serde_for_ark_serde,
    scalar::{MontScalar, Scalar},
};
use ark_ec::pairing::PairingOutput;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use core::ops::Mul;
use derive_more::{AddAssign, Neg, Sub};
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
    Debug, Sub, Eq, PartialEq, Neg, Copy, Clone, AddAssign, CanonicalSerialize, CanonicalDeserialize,
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
impl_serde_for_ark_serde!(DoryCommitment);
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
}

impl VecCommitmentExt for Vec<DoryCommitment> {
    type CommitmentPublicSetup = DoryProverPublicSetup;
    fn from_commitable_columns_with_offset(
        committable_columns: &[CommittableColumn],
        offset: usize,
        setup: &Self::CommitmentPublicSetup,
    ) -> Self {
        super::dory_commitment_helper::compute_dory_commitments(committable_columns, offset, setup)
    }

    fn try_append_rows_with_offset<'a, C>(
        &mut self,
        columns: impl IntoIterator<Item = C>,
        offset: usize,
        setup: &Self::CommitmentPublicSetup,
    ) -> Result<(), NumColumnsMismatch>
    where
        C: Into<CommittableColumn<'a>>,
    {
        let committable_columns: Vec<CommittableColumn<'a>> =
            columns.into_iter().map(Into::into).collect::<Vec<_>>();
        if self.len() != committable_columns.len() {
            return Err(NumColumnsMismatch);
        }
        let other = Self::from_commitable_columns_with_offset(&committable_columns, offset, setup);
        for (commitment_a, commitment_b) in self.iter_mut().zip(other) {
            commitment_a.0 += commitment_b.0
        }
        Ok(())
    }

    fn from_columns_with_offset<'a, C>(
        columns: impl IntoIterator<Item = C>,
        offset: usize,
        setup: &Self::CommitmentPublicSetup,
    ) -> Self
    where
        C: Into<CommittableColumn<'a>>,
    {
        let committable_columns: Vec<CommittableColumn<'a>> =
            columns.into_iter().map(Into::into).collect::<Vec<_>>();
        Self::from_commitable_columns_with_offset(&committable_columns, offset, setup)
    }
    fn extend_columns_with_offset<'a, C>(
        &mut self,
        columns: impl IntoIterator<Item = C>,
        offset: usize,
        setup: &Self::CommitmentPublicSetup,
    ) where
        C: Into<CommittableColumn<'a>>,
    {
        self.extend(Self::from_columns_with_offset(columns, offset, setup))
    }
    fn try_add(self, other: Self) -> Result<Self, NumColumnsMismatch>
    where
        Self: Sized,
    {
        if self.len() != other.len() {
            return Err(NumColumnsMismatch);
        }
        let commitments = self
            .into_iter()
            .zip(other)
            .map(|(commitment_a, commitment_b)| DoryCommitment(commitment_a.0 + commitment_b.0))
            .collect();
        Ok(commitments)
    }
    fn try_sub(self, other: Self) -> Result<Self, NumColumnsMismatch>
    where
        Self: Sized,
    {
        if self.len() != other.len() {
            return Err(NumColumnsMismatch);
        }
        let commitments = self
            .into_iter()
            .zip(other)
            .map(|(commitment_a, commitment_b)| DoryCommitment(commitment_a.0 - commitment_b.0))
            .collect();
        Ok(commitments)
    }
    fn num_commitments(&self) -> usize {
        self.len()
    }
    type DecompressedCommitment = DoryCommitment;
    fn to_decompressed(&self) -> Option<Vec<Self::DecompressedCommitment>> {
        Some(self.to_vec())
    }

    fn get_decompressed_commitment(&self, i: usize) -> Option<Self::DecompressedCommitment> {
        Some(self[i])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::database::{Column, OwnedColumn},
        proof_primitive::dory::rand_util::test_rng,
    };
    use ark_ec::pairing::Pairing;

    #[test]
    fn we_can_convert_from_columns() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());

        // empty case
        let commitments = Vec::<DoryCommitment>::from_columns_with_offset(
            &Vec::<Column<DoryScalar>>::new(),
            0,
            &setup,
        );

        assert!(commitments.is_empty());

        // nonempty case
        let column_a = [12i64, 34, 56];
        let column_b = ["Lorem", "ipsum", "dolor"].map(String::from);

        let columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a.to_vec()),
            OwnedColumn::VarChar(column_b.to_vec()),
        ];

        let commitments = Vec::<DoryCommitment>::from_columns_with_offset(&columns, 0, &setup);

        let mut expected_commitments = vec![DoryCommitment::default(); 2];
        expected_commitments[0] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[0],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_a[0]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[1],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_a[1]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[2],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_a[2]).0,
        );
        expected_commitments[1] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[0],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_b[0].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[1],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_b[1].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[2],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_b[2].clone()).0,
        );

        assert_eq!(commitments, expected_commitments);
    }

    #[test]
    fn we_can_append_rows() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());

        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let mut commitments = Vec::<DoryCommitment>::from_columns_with_offset(&columns, 0, &setup);

        let new_columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a[3..].to_vec()),
            OwnedColumn::VarChar(column_b[3..].to_vec()),
        ];

        commitments
            .try_append_rows_with_offset(&new_columns, 3, &setup)
            .unwrap();

        let mut expected_commitments = vec![DoryCommitment::default(); 2];
        expected_commitments[0] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[0],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_a[0]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[1],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_a[1]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[2],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_a[2]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[3],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_a[3]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[0],
                    setup.public_parameters().Gamma_2[1],
                ) * DoryScalar::from(column_a[4]).0,
        );
        expected_commitments[1] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[0],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_b[0].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[1],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_b[1].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[2],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_b[2].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[3],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_b[3].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[0],
                    setup.public_parameters().Gamma_2[1],
                ) * DoryScalar::from(column_b[4].clone()).0,
        );

        assert_eq!(commitments, expected_commitments);
    }

    #[test]
    fn we_cannot_append_rows_with_different_column_count() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());

        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let mut commitments = Vec::<DoryCommitment>::from_columns_with_offset(&columns, 0, &setup);

        let new_columns = Vec::<Column<DoryScalar>>::new();
        assert!(matches!(
            commitments.try_append_rows_with_offset(&new_columns, 3, &setup),
            Err(NumColumnsMismatch)
        ));

        let new_columns = vec![OwnedColumn::<DoryScalar>::BigInt(column_a[3..].to_vec())];
        assert!(matches!(
            commitments.try_append_rows_with_offset(&new_columns, 3, &setup),
            Err(NumColumnsMismatch)
        ));

        let new_columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a[3..].to_vec()),
            OwnedColumn::VarChar(column_b[3..].to_vec()),
            OwnedColumn::BigInt(column_a[3..].to_vec()),
        ];
        assert!(matches!(
            commitments.try_append_rows_with_offset(&new_columns, 3, &setup),
            Err(NumColumnsMismatch)
        ));
    }

    #[test]
    fn we_can_extend_columns() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());

        let column_a = [12i64, 34, 56];
        let column_b = ["Lorem", "ipsum", "dolor"].map(String::from);
        let column_c = ["sit", "amet", "consectetur"].map(String::from);
        let column_d = [78i64, 90, 1112];

        let columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a.to_vec()),
            OwnedColumn::VarChar(column_b.to_vec()),
        ];

        let mut commitments = Vec::<DoryCommitment>::from_columns_with_offset(&columns, 0, &setup);

        let new_columns = vec![
            OwnedColumn::<DoryScalar>::VarChar(column_c.to_vec()),
            OwnedColumn::BigInt(column_d.to_vec()),
        ];

        commitments.extend_columns_with_offset(&new_columns, 0, &setup);

        let mut expected_commitments = vec![DoryCommitment::default(); 4];

        expected_commitments[0] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[0],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_a[0]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[1],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_a[1]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[2],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_a[2]).0,
        );
        expected_commitments[1] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[0],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_b[0].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[1],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_b[1].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[2],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_b[2].clone()).0,
        );
        expected_commitments[2] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[0],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_c[0].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[1],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_c[1].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[2],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_c[2].clone()).0,
        );
        expected_commitments[3] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[0],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_d[0]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[1],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_d[1]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[2],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_d[2]).0,
        );

        assert_eq!(commitments, expected_commitments);
    }

    #[test]
    fn we_can_add_commitment_collections() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());

        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let commitments_a = Vec::<DoryCommitment>::from_columns_with_offset(&columns, 0, &setup);

        let new_columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a[3..].to_vec()),
            OwnedColumn::VarChar(column_b[3..].to_vec()),
        ];

        let commitments_b =
            Vec::<DoryCommitment>::from_columns_with_offset(&new_columns, 3, &setup);

        let commitments = commitments_a.try_add(commitments_b).unwrap();

        let mut expected_commitments = vec![DoryCommitment::default(); 2];
        expected_commitments[0] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[0],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_a[0]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[1],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_a[1]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[2],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_a[2]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[3],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_a[3]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[0],
                    setup.public_parameters().Gamma_2[1],
                ) * DoryScalar::from(column_a[4]).0,
        );
        expected_commitments[1] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[0],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_b[0].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[1],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_b[1].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[2],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_b[2].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[3],
                    setup.public_parameters().Gamma_2[0],
                ) * DoryScalar::from(column_b[3].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[0],
                    setup.public_parameters().Gamma_2[1],
                ) * DoryScalar::from(column_b[4].clone()).0,
        );

        assert_eq!(commitments, expected_commitments);
    }

    #[test]
    fn we_cannot_add_commitment_collections_of_mixed_column_counts() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());

        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let commitments = Vec::<DoryCommitment>::from_columns_with_offset(&columns, 0, &setup);

        let new_columns = Vec::<Column<DoryScalar>>::new();
        let new_commitments =
            Vec::<DoryCommitment>::from_columns_with_offset(&new_columns, 3, &setup);
        assert!(matches!(
            commitments.clone().try_add(new_commitments),
            Err(NumColumnsMismatch)
        ));

        let new_columns = vec![OwnedColumn::<DoryScalar>::BigInt(column_a[3..].to_vec())];
        let new_commitments =
            Vec::<DoryCommitment>::from_columns_with_offset(&new_columns, 3, &setup);
        assert!(matches!(
            commitments.clone().try_add(new_commitments),
            Err(NumColumnsMismatch)
        ));

        let new_columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a[3..].to_vec()),
            OwnedColumn::VarChar(column_b[3..].to_vec()),
            OwnedColumn::BigInt(column_a[3..].to_vec()),
        ];
        let new_commitments =
            Vec::<DoryCommitment>::from_columns_with_offset(&new_columns, 3, &setup);
        assert!(matches!(
            commitments.try_add(new_commitments),
            Err(NumColumnsMismatch)
        ));
    }

    #[test]
    fn we_can_sub_commitment_collections() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());

        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let commitments_a = Vec::<DoryCommitment>::from_columns_with_offset(&columns, 0, &setup);

        let full_columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a.to_vec()),
            OwnedColumn::VarChar(column_b.to_vec()),
        ];

        let commitments_b =
            Vec::<DoryCommitment>::from_columns_with_offset(&full_columns, 0, &setup);

        let commitments = commitments_b.try_sub(commitments_a).unwrap();

        let mut expected_commitments = vec![DoryCommitment::default(); 2];

        expected_commitments[0] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[3],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_a[3]).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[0],
                    setup.public_parameters().Gamma_2[1],
                ) * DoryScalar::from(column_a[4]).0,
        );
        expected_commitments[1] = DoryCommitment(
            Pairing::pairing(
                setup.public_parameters().Gamma_1[3],
                setup.public_parameters().Gamma_2[0],
            ) * DoryScalar::from(column_b[3].clone()).0
                + Pairing::pairing(
                    setup.public_parameters().Gamma_1[0],
                    setup.public_parameters().Gamma_2[1],
                ) * DoryScalar::from(column_b[4].clone()).0,
        );

        assert_eq!(commitments, expected_commitments);
    }

    #[test]
    fn we_cannot_sub_commitment_collections_of_mixed_column_counts() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());

        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let commitments = Vec::<DoryCommitment>::from_columns_with_offset(&columns, 0, &setup);

        let full_columns = Vec::<Column<DoryScalar>>::new();
        let full_commitments =
            Vec::<DoryCommitment>::from_columns_with_offset(&full_columns, 0, &setup);
        assert!(matches!(
            full_commitments.clone().try_sub(commitments.clone()),
            Err(NumColumnsMismatch)
        ));

        let full_columns = vec![OwnedColumn::<DoryScalar>::BigInt(column_a.to_vec())];
        let full_commitments =
            Vec::<DoryCommitment>::from_columns_with_offset(&full_columns, 0, &setup);
        assert!(matches!(
            full_commitments.try_sub(commitments.clone()),
            Err(NumColumnsMismatch)
        ));

        let full_columns = vec![
            OwnedColumn::<DoryScalar>::BigInt(column_a.to_vec()),
            OwnedColumn::VarChar(column_b.to_vec()),
            OwnedColumn::BigInt(column_a.to_vec()),
        ];
        let full_commitments =
            Vec::<DoryCommitment>::from_columns_with_offset(&full_columns, 0, &setup);
        assert!(matches!(
            full_commitments.try_sub(commitments),
            Err(NumColumnsMismatch)
        ));
    }
}
