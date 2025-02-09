use super::{
    hyrax_configuration::HyraxConfiguration, hyrax_public_setup::HyraxPublicSetup,
    hyrax_scalar::HyraxScalarWrapper,
};
use crate::{
    base::{
        commitment::{Commitment, CommittableColumn},
        if_rayon,
    },
    proof_primitive::dynamic_matrix_utils::matrix_structure::row_and_column_from_index,
};
use alloc::{vec, vec::Vec};
use ark_std::iterable::Iterable;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
use itertools::{EitherOrBoth, Itertools};
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

/// The Hyrax commitment scheme:
/// A column of data is converted to a matrix, which is crossed with a vector of generators to produce a vector of commits, which is the commitment.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Default)]
pub struct HyraxCommitment<C: HyraxConfiguration> {
    /// The result of matrix cross the generators
    pub row_commits: Vec<C::CompressedGroup>,
}

impl<'a, C: HyraxConfiguration> HyraxCommitment<C>
where
    for<'b> C::OperableGroup: 'b,
{
    /// The algorithm for computing a hyrax commitment is as follows:
    /// Transform the committable column into a dynamic matrix (similar to dynamic dory)
    /// Multiply the dynamic matrix by the generators from the setup.
    /// The resulting vector of group elements is the commitment.
    /// There is a need (for sp1 development) for commitments to be deserialized to a decompressed type rather than a compressed type,
    /// so the vector is converted to a vector of compressed group elements.
    fn compute_hyrax_commitment(
        committable_column: &CommittableColumn,
        offset: usize,
        setup: &HyraxPublicSetup<'a, C::OperableGroup>,
    ) -> Self {
        let scalar_column: Vec<C::OperableScalar> = match committable_column {
            CommittableColumn::Boolean(vec) => vec.iter().map(Into::into).collect(),
            CommittableColumn::TinyInt(vec) => vec.iter().map(Into::into).collect(),
            CommittableColumn::SmallInt(vec) => vec.iter().map(Into::into).collect(),
            CommittableColumn::Int(vec) => vec.iter().map(Into::into).collect(),
            CommittableColumn::BigInt(vec) | CommittableColumn::TimestampTZ(_, _, vec) => {
                vec.iter().map(Into::into).collect()
            }
            CommittableColumn::Int128(vec) => vec.iter().map(Into::into).collect(),
            CommittableColumn::Decimal75(_, _, vec) => vec.iter().map(Into::into).collect(),
            CommittableColumn::Scalar(vec) | CommittableColumn::VarChar(vec) => {
                vec.iter().map(Into::into).collect()
            }
            CommittableColumn::RangeCheckWord(vec) => vec.iter().map(Into::into).collect(),
        };
        let table_size = offset + committable_column.len();
        let empty_row_commits =
            vec![C::OperableGroup::default(); row_and_column_from_index(table_size - 1).0 + 1];
        let decompressed_row_commits = if_rayon!(
            (offset..table_size).into_par_iter(),
            (offset..table_size).into_iter()
        )
        .map(|index| {
            let (row, column) = row_and_column_from_index(index);
            (
                row,
                setup.generators[column] * scalar_column[index - offset],
            )
        })
        .collect::<Vec<_>>()
        .into_iter()
        .fold(empty_row_commits, |mut acc, (row, ep)| {
            acc[row] += ep;
            acc
        });
        let row_commits: Vec<C::CompressedGroup> = if_rayon!(
            decompressed_row_commits.par_iter(),
            decompressed_row_commits.iter()
        )
        .map(C::from_operable_to_compressed)
        .collect();
        Self { row_commits }
    }

    /// # Panics
    ///
    /// Will panic if decompression is unsuccessful.
    fn add_row_commits(
        row_commits_a: &[C::CompressedGroup],
        row_commits_b: &[C::CompressedGroup],
    ) -> Vec<C::CompressedGroup> {
        row_commits_a
            .iter()
            .zip_longest(row_commits_b)
            .map(|both| match both {
                EitherOrBoth::Both(a, b) => C::from_operable_to_compressed(
                    &(C::from_compressed_to_operable(a) + C::from_compressed_to_operable(b)),
                ),
                EitherOrBoth::Left(a) => *a,
                EitherOrBoth::Right(b) => *b,
            })
            .collect()
    }

    fn sub_row_commits(
        row_commits_a: &[C::CompressedGroup],
        row_commits_b: &[C::CompressedGroup],
    ) -> Vec<C::CompressedGroup> {
        row_commits_a
            .iter()
            .zip_longest(row_commits_b)
            .map(|both| match both {
                EitherOrBoth::Both(a, b) => C::from_operable_to_compressed(
                    &(C::from_compressed_to_operable(a) - C::from_compressed_to_operable(b)),
                ),
                EitherOrBoth::Left(a) => *a,
                EitherOrBoth::Right(b) => {
                    C::from_operable_to_compressed(&-C::from_compressed_to_operable(b))
                }
            })
            .collect()
    }
}

impl<C: HyraxConfiguration> Sub for HyraxCommitment<C>
where
    for<'a> C::OperableGroup: 'a,
{
    type Output = HyraxCommitment<C>;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            row_commits: Self::sub_row_commits(&self.row_commits, &rhs.row_commits),
        }
    }
}

impl<C: HyraxConfiguration> SubAssign for HyraxCommitment<C>
where
    for<'a> C::OperableGroup: 'a,
{
    fn sub_assign(&mut self, rhs: Self) {
        self.row_commits = Self::sub_row_commits(&self.row_commits, &rhs.row_commits);
    }
}

impl<C: HyraxConfiguration> Neg for HyraxCommitment<C> {
    type Output = HyraxCommitment<C>;

    fn neg(self) -> Self::Output {
        HyraxCommitment {
            row_commits: self
                .row_commits
                .iter()
                .map(|rc| C::from_operable_to_compressed(&-C::from_compressed_to_operable(rc)))
                .collect(),
        }
    }
}

impl<C: HyraxConfiguration> AddAssign for HyraxCommitment<C>
where
    for<'a> C::OperableGroup: 'a,
{
    fn add_assign(&mut self, rhs: Self) {
        self.row_commits = Self::add_row_commits(&self.row_commits, &rhs.row_commits);
    }
}

impl<C: HyraxConfiguration> Add for HyraxCommitment<C>
where
    for<'a> C::OperableGroup: 'a,
{
    type Output = HyraxCommitment<C>;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            row_commits: Self::add_row_commits(&self.row_commits, &rhs.row_commits),
        }
    }
}

impl<C: HyraxConfiguration> Mul<HyraxCommitment<C>> for HyraxScalarWrapper<C::OperableScalar> {
    type Output = HyraxCommitment<C>;

    fn mul(self, rhs: HyraxCommitment<C>) -> Self::Output {
        self * &rhs
    }
}

impl<C: HyraxConfiguration> Mul<&HyraxCommitment<C>> for HyraxScalarWrapper<C::OperableScalar> {
    type Output = HyraxCommitment<C>;

    fn mul(self, rhs: &HyraxCommitment<C>) -> Self::Output {
        HyraxCommitment {
            row_commits: rhs
                .row_commits
                .iter()
                .map(|rc| {
                    C::from_operable_to_compressed(&(C::from_compressed_to_operable(rc) * self.0))
                })
                .collect(),
        }
    }
}

impl<C: HyraxConfiguration> Commitment for HyraxCommitment<C>
where
    for<'a> C::OperableGroup: 'a,
{
    type Scalar = HyraxScalarWrapper<C::OperableScalar>;

    type PublicSetup<'a> = HyraxPublicSetup<'a, C::OperableGroup>;
    fn compute_commitments(
        committable_columns: &[CommittableColumn],
        offset: usize,
        setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self> {
        committable_columns
            .iter()
            .map(|cc| Self::compute_hyrax_commitment(cc, offset, setup))
            .collect()
    }

    fn append_to_transcript(&self, transcript: &mut impl crate::base::proof::Transcript) {
        transcript.extend_as_le(self.row_commits.iter().map(|rc| C::compressed_to_bytes(rc)));
    }
}
