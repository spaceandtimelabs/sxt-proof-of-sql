use super::Commitment;
use crate::base::commitment::committable_column::CommittableColumn;
use alloc::vec::Vec;
use snafu::Snafu;

/// Cannot update commitment collections with different column counts
#[derive(Snafu, Debug)]
#[snafu(display("cannot update commitment collections with different column counts"))]
pub struct NumColumnsMismatch;

/// Extension trait intended for collections of commitments.
///
/// Implemented for `Vec<CompressedRistretto>`.
pub trait VecCommitmentExt {
    /// The public setup parameters required to compute the commitments.
    /// This is simply precomputed data that is required to compute the commitments.
    type CommitmentPublicSetup<'a>;

    /// Returns a collection of commitments to the provided columns using the given generator offset.
    fn from_columns_with_offset<'a, C>(
        columns: impl IntoIterator<Item = C>,
        offset: usize,
        setup: &Self::CommitmentPublicSetup<'_>,
    ) -> Self
    where
        C: Into<CommittableColumn<'a>>;

    /// Returns a collection of commitments to the provided slice of `CommittableColumn`s using the given generator offset.
    fn from_commitable_columns_with_offset(
        committable_columns: &[CommittableColumn],
        offset: usize,
        setup: &Self::CommitmentPublicSetup<'_>,
    ) -> Self;

    /// Append rows of data from the provided columns to the existing commitments.
    ///
    /// The given generator offset will be used for committing to the new rows.
    /// You most likely want this to be equal to the 0-indexed row number of the first new row.
    ///
    /// The number of columns provided must match the number of columns already committed to.
    fn try_append_rows_with_offset<'a, C>(
        &mut self,
        columns: impl IntoIterator<Item = C>,
        offset: usize,
        setup: &Self::CommitmentPublicSetup<'_>,
    ) -> Result<(), NumColumnsMismatch>
    where
        C: Into<CommittableColumn<'a>>;

    /// Add commitments to new columns to this collection using the given generator offset.
    fn extend_columns_with_offset<'a, C>(
        &mut self,
        columns: impl IntoIterator<Item = C>,
        offset: usize,
        setup: &Self::CommitmentPublicSetup<'_>,
    ) where
        C: Into<CommittableColumn<'a>>;

    /// Add two collections of commitments if they have equal column counts.
    fn try_add(self, other: Self) -> Result<Self, NumColumnsMismatch>
    where
        Self: Sized;

    /// Subtract two collections of commitments if they have equal column counts.
    fn try_sub(self, other: Self) -> Result<Self, NumColumnsMismatch>
    where
        Self: Sized;

    /// Returns the number of commitments in the collection.
    fn num_commitments(&self) -> usize;
}

fn unsafe_add_assign<C: Commitment>(a: &mut [C], b: &[C]) {
    a.iter_mut().zip(b).for_each(|(c_a, c_b)| {
        *c_a += c_b.clone();
    });
}
fn unsafe_sub_assign<C: Commitment>(a: &mut [C], b: &[C]) {
    a.iter_mut().zip(b).for_each(|(c_a, c_b)| {
        *c_a -= c_b.clone();
    });
}

impl<C: Commitment> VecCommitmentExt for Vec<C> {
    type CommitmentPublicSetup<'a> = C::PublicSetup<'a>;
    fn from_columns_with_offset<'a, COL>(
        columns: impl IntoIterator<Item = COL>,
        offset: usize,
        setup: &Self::CommitmentPublicSetup<'_>,
    ) -> Self
    where
        COL: Into<CommittableColumn<'a>>,
    {
        let committable_columns: Vec<CommittableColumn<'a>> =
            columns.into_iter().map(Into::into).collect::<Vec<_>>();

        Self::from_commitable_columns_with_offset(&committable_columns, offset, setup)
    }

    fn from_commitable_columns_with_offset(
        committable_columns: &[CommittableColumn],
        offset: usize,
        setup: &Self::CommitmentPublicSetup<'_>,
    ) -> Self {
        C::compute_commitments(committable_columns, offset, setup)
    }

    fn try_append_rows_with_offset<'a, COL>(
        &mut self,
        columns: impl IntoIterator<Item = COL>,
        offset: usize,
        setup: &Self::CommitmentPublicSetup<'_>,
    ) -> Result<(), NumColumnsMismatch>
    where
        COL: Into<CommittableColumn<'a>>,
    {
        let committable_columns: Vec<CommittableColumn<'a>> =
            columns.into_iter().map(Into::into).collect::<Vec<_>>();

        if self.len() != committable_columns.len() {
            return Err(NumColumnsMismatch);
        }

        let partial_commitments = C::compute_commitments(&committable_columns, offset, setup);
        unsafe_add_assign(self, &partial_commitments);

        Ok(())
    }

    fn extend_columns_with_offset<'a, COL>(
        &mut self,
        columns: impl IntoIterator<Item = COL>,
        offset: usize,
        setup: &Self::CommitmentPublicSetup<'_>,
    ) where
        COL: Into<CommittableColumn<'a>>,
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

        let mut commitments = self;
        unsafe_add_assign(&mut commitments, &other);

        Ok(commitments)
    }

    fn try_sub(self, other: Self) -> Result<Self, NumColumnsMismatch>
    where
        Self: Sized,
    {
        if self.len() != other.len() {
            return Err(NumColumnsMismatch);
        }

        let mut commitments = self;
        unsafe_sub_assign(&mut commitments, &other);

        Ok(commitments)
    }

    fn num_commitments(&self) -> usize {
        self.len()
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::base::{
        commitment::naive_commitment::NaiveCommitment,
        database::{Column, OwnedColumn},
        scalar::test_scalar::TestScalar,
    };

    #[test]
    fn we_can_convert_from_columns() {
        // empty case
        let commitments = Vec::<NaiveCommitment>::from_columns_with_offset(
            &Vec::<Column<TestScalar>>::new(),
            0,
            &(),
        );

        assert!(commitments.is_empty());

        // nonempty case
        let column_a = [12i64, 34, 56];
        let column_b = ["Lorem", "ipsum", "dolor"].map(String::from);

        let columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a.to_vec()),
            OwnedColumn::VarChar(column_b.to_vec()),
        ];

        let commitments = Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());

        let committable_columns = [
            CommittableColumn::BigInt(&column_a),
            CommittableColumn::VarChar(
                column_b
                    .iter()
                    .map(TestScalar::from)
                    .map(<[u64; 4]>::from)
                    .collect(),
            ),
        ];
        let expected_commitments =
            NaiveCommitment::compute_commitments(&committable_columns, 0, &());

        assert_eq!(commitments, expected_commitments);
    }

    #[test]
    fn we_can_append_rows() {
        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let mut commitments = Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());

        let new_columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a[3..].to_vec()),
            OwnedColumn::VarChar(column_b[3..].to_vec()),
        ];

        commitments
            .try_append_rows_with_offset(&new_columns, 3, &())
            .unwrap();

        let committable_columns = [
            CommittableColumn::BigInt(&column_a),
            CommittableColumn::VarChar(
                column_b
                    .iter()
                    .map(TestScalar::from)
                    .map(<[u64; 4]>::from)
                    .collect(),
            ),
        ];
        let expected_commitments =
            NaiveCommitment::compute_commitments(&committable_columns, 0, &());

        assert_eq!(commitments, expected_commitments);
    }

    #[test]
    fn we_cannot_append_rows_with_different_column_count() {
        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let mut commitments = Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());

        let new_columns = Vec::<Column<TestScalar>>::new();
        assert!(matches!(
            commitments.try_append_rows_with_offset(&new_columns, 3, &()),
            Err(NumColumnsMismatch)
        ));

        let new_columns = vec![OwnedColumn::<TestScalar>::BigInt(column_a[3..].to_vec())];
        assert!(matches!(
            commitments.try_append_rows_with_offset(&new_columns, 3, &()),
            Err(NumColumnsMismatch)
        ));

        let new_columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a[3..].to_vec()),
            OwnedColumn::VarChar(column_b[3..].to_vec()),
            OwnedColumn::BigInt(column_a[3..].to_vec()),
        ];
        assert!(matches!(
            commitments.try_append_rows_with_offset(&new_columns, 3, &()),
            Err(NumColumnsMismatch)
        ));
    }

    #[test]
    fn we_can_extend_columns() {
        let column_a = [12i64, 34, 56];
        let column_b = ["Lorem", "ipsum", "dolor"].map(String::from);
        let column_c = ["sit", "amet", "consectetur"].map(String::from);
        let column_d = [78i64, 90, 1112];

        let columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a.to_vec()),
            OwnedColumn::VarChar(column_b.to_vec()),
        ];

        let mut commitments = Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());

        let new_columns = vec![
            OwnedColumn::<TestScalar>::VarChar(column_c.to_vec()),
            OwnedColumn::BigInt(column_d.to_vec()),
        ];

        commitments.extend_columns_with_offset(&new_columns, 0, &());

        let committable_columns = [
            CommittableColumn::VarChar(
                column_a
                    .iter()
                    .map(Into::<TestScalar>::into)
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
            CommittableColumn::VarChar(
                column_b
                    .iter()
                    .map(Into::<TestScalar>::into)
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
            CommittableColumn::VarChar(
                column_c
                    .iter()
                    .map(Into::<TestScalar>::into)
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
            CommittableColumn::VarChar(
                column_d
                    .iter()
                    .map(Into::<TestScalar>::into)
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
        ];
        let expected_commitments =
            NaiveCommitment::compute_commitments(&committable_columns, 0, &());

        assert_eq!(commitments, expected_commitments);
    }

    #[test]
    fn we_can_add_commitment_collections() {
        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let commitments_a = Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());

        let new_columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a[3..].to_vec()),
            OwnedColumn::VarChar(column_b[3..].to_vec()),
        ];

        let commitments_b = Vec::<NaiveCommitment>::from_columns_with_offset(&new_columns, 3, &());

        let commitments = commitments_a.try_add(commitments_b).unwrap();

        let committable_columns = [
            CommittableColumn::BigInt(&column_a),
            CommittableColumn::VarChar(
                column_b
                    .iter()
                    .map(Into::<TestScalar>::into)
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
        ];
        let expected_commitments =
            NaiveCommitment::compute_commitments(&committable_columns, 0, &());

        assert_eq!(commitments, expected_commitments);
    }

    #[test]
    fn we_cannot_add_commitment_collections_of_mixed_column_counts() {
        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let commitments = Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());

        let new_columns = Vec::<Column<TestScalar>>::new();
        let new_commitments =
            Vec::<NaiveCommitment>::from_columns_with_offset(&new_columns, 3, &());
        assert!(matches!(
            commitments.clone().try_add(new_commitments),
            Err(NumColumnsMismatch)
        ));

        let new_columns = vec![OwnedColumn::<TestScalar>::BigInt(column_a[3..].to_vec())];
        let new_commitments =
            Vec::<NaiveCommitment>::from_columns_with_offset(&new_columns, 3, &());
        assert!(matches!(
            commitments.clone().try_add(new_commitments),
            Err(NumColumnsMismatch)
        ));

        let new_columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a[3..].to_vec()),
            OwnedColumn::VarChar(column_b[3..].to_vec()),
            OwnedColumn::BigInt(column_a[3..].to_vec()),
        ];
        let new_commitments =
            Vec::<NaiveCommitment>::from_columns_with_offset(&new_columns, 3, &());
        assert!(matches!(
            commitments.try_add(new_commitments),
            Err(NumColumnsMismatch)
        ));
    }

    #[test]
    fn we_can_sub_commitment_collections() {
        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);
        let front_emptied_column_a = [0i64, 0, 0, 78, 90];
        let front_emptied_column_b = ["", "", "", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let commitments_a = Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());

        let full_columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a.to_vec()),
            OwnedColumn::VarChar(column_b.to_vec()),
        ];

        let commitments_b = Vec::<NaiveCommitment>::from_columns_with_offset(&full_columns, 0, &());

        let commitments = commitments_b.try_sub(commitments_a).unwrap();

        let committable_columns = [
            CommittableColumn::BigInt(&front_emptied_column_a),
            CommittableColumn::VarChar(
                front_emptied_column_b
                    .iter()
                    .map(Into::<TestScalar>::into)
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
        ];
        let expected_commitments =
            NaiveCommitment::compute_commitments(&committable_columns, 0, &());

        assert_eq!(commitments, expected_commitments);
    }

    #[test]
    fn we_cannot_sub_commitment_collections_of_mixed_column_counts() {
        let column_a = [12i64, 34, 56, 78, 90];
        let column_b = ["Lorem", "ipsum", "dolor", "sit", "amet"].map(String::from);

        let columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a[..3].to_vec()),
            OwnedColumn::VarChar(column_b[..3].to_vec()),
        ];

        let commitments = Vec::<NaiveCommitment>::from_columns_with_offset(&columns, 0, &());

        let full_columns = Vec::<Column<TestScalar>>::new();
        let full_commitments =
            Vec::<NaiveCommitment>::from_columns_with_offset(&full_columns, 0, &());
        assert!(matches!(
            full_commitments.clone().try_sub(commitments.clone()),
            Err(NumColumnsMismatch)
        ));

        let full_columns = vec![OwnedColumn::<TestScalar>::BigInt(column_a.to_vec())];
        let full_commitments =
            Vec::<NaiveCommitment>::from_columns_with_offset(&full_columns, 0, &());
        assert!(matches!(
            full_commitments.try_sub(commitments.clone()),
            Err(NumColumnsMismatch)
        ));

        let full_columns = vec![
            OwnedColumn::<TestScalar>::BigInt(column_a.to_vec()),
            OwnedColumn::VarChar(column_b.to_vec()),
            OwnedColumn::BigInt(column_a.to_vec()),
        ];
        let full_commitments =
            Vec::<NaiveCommitment>::from_columns_with_offset(&full_columns, 0, &());
        assert!(matches!(
            full_commitments.try_sub(commitments),
            Err(NumColumnsMismatch)
        ));
    }
}
