use crate::base::{
    database::{Column, ColumnType, OwnedColumn},
    scalar::ArkScalar,
};
use blitzar::sequence::Sequence;

/// Column data in "committable form".
///
/// For some column types, transformations need to be applied before commitments are created.
/// These transformations require allocating new memory.
/// This is a problem since blitzar only borrows slices of data to commit to.
/// Normal column types don't store their data in "committable" form, so they cannot interface with
/// blitzar directly.
///
/// This type acts as an intermediate column type that *can* be used with blitzar directly.
/// For column types that need to be transformed, their "committable form" is owned here.
/// For column types that don't need to allocate new memory, their data is only borrowed here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommittableColumn<'a> {
    /// Borrowed BigInt column, mapped to `i64`.
    BigInt(&'a [i64]),
    /// Borrowed Int128 column, mapped to `i128`.
    Int128(&'a [i128]),
    /// Column of big ints for committing to, hashed from a VarChar column.
    VarChar(Vec<[u64; 4]>),
    #[cfg(test)]
    /// Column of big ints for committing to, montgomery-reduced from a Scalar column.
    Scalar(Vec<[u64; 4]>),
}

impl<'a> CommittableColumn<'a> {
    /// Returns the length of the column.
    pub fn len(&self) -> usize {
        match self {
            CommittableColumn::BigInt(col) => col.len(),
            CommittableColumn::VarChar(col) => col.len(),
            CommittableColumn::Int128(col) => col.len(),
            #[cfg(test)]
            CommittableColumn::Scalar(col) => col.len(),
        }
    }

    /// Returns true if the column is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the type of the column.
    pub fn column_type(&self) -> ColumnType {
        self.into()
    }
}

impl<'a> From<&CommittableColumn<'a>> for ColumnType {
    fn from(value: &CommittableColumn<'a>) -> Self {
        match value {
            CommittableColumn::BigInt(_) => ColumnType::BigInt,
            CommittableColumn::Int128(_) => ColumnType::Int128,
            CommittableColumn::VarChar(_) => ColumnType::VarChar,
            #[cfg(test)]
            CommittableColumn::Scalar(_) => ColumnType::Scalar,
        }
    }
}

impl<'a> From<&Column<'a>> for CommittableColumn<'a> {
    fn from(value: &Column<'a>) -> Self {
        match value {
            Column::BigInt(ints) => CommittableColumn::BigInt(ints),
            Column::Int128(ints) => CommittableColumn::Int128(ints),
            Column::VarChar((_, scalars)) => {
                let as_limbs: Vec<_> = scalars.iter().map(Into::<[u64; 4]>::into).collect();
                CommittableColumn::VarChar(as_limbs)
            }
            #[cfg(test)]
            Column::Scalar(scalars) => {
                let as_limbs: Vec<_> = scalars.iter().map(Into::<[u64; 4]>::into).collect();
                CommittableColumn::Scalar(as_limbs)
            }
        }
    }
}

impl<'a> From<&'a OwnedColumn> for CommittableColumn<'a> {
    fn from(value: &'a OwnedColumn) -> Self {
        match value {
            OwnedColumn::BigInt(ints) => CommittableColumn::BigInt(ints),
            OwnedColumn::Int128(ints) => CommittableColumn::Int128(ints),
            OwnedColumn::VarChar(strings) => {
                let as_limbs: Vec<_> = strings
                    .iter()
                    .map(ArkScalar::from)
                    .map(Into::<[u64; 4]>::into)
                    .collect();
                CommittableColumn::VarChar(as_limbs)
            }
            #[cfg(test)]
            OwnedColumn::Scalar(scalars) => {
                let as_limbs: Vec<_> = scalars.iter().map(Into::<[u64; 4]>::into).collect();
                CommittableColumn::Scalar(as_limbs)
            }
        }
    }
}
impl<'a, 'b> From<&'a CommittableColumn<'b>> for Sequence<'a> {
    fn from(value: &'a CommittableColumn<'b>) -> Self {
        match value {
            CommittableColumn::BigInt(ints) => Sequence::from(*ints),
            CommittableColumn::Int128(ints) => Sequence::from(*ints),
            CommittableColumn::VarChar(limbs) => Sequence::from(limbs),
            #[cfg(test)]
            CommittableColumn::Scalar(limbs) => Sequence::from(limbs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blitzar::compute::compute_commitments;
    use curve25519_dalek::ristretto::CompressedRistretto;

    #[test]
    fn we_can_get_type_and_length_of_bigint_column() {
        // empty case
        let bigint_committable_column = CommittableColumn::BigInt(&[]);
        assert_eq!(bigint_committable_column.len(), 0);
        assert!(bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::BigInt);

        let bigint_committable_column = CommittableColumn::BigInt(&[12, 34, 56]);
        assert_eq!(bigint_committable_column.len(), 3);
        assert!(!bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::BigInt);
    }

    #[test]
    fn we_can_get_type_and_length_of_int128_column() {
        // empty case
        let bigint_committable_column = CommittableColumn::Int128(&[]);
        assert_eq!(bigint_committable_column.len(), 0);
        assert!(bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::Int128);

        let bigint_committable_column = CommittableColumn::Int128(&[12, 34, 56]);
        assert_eq!(bigint_committable_column.len(), 3);
        assert!(!bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::Int128);
    }

    #[test]
    fn we_can_get_type_and_length_of_varchar_column() {
        // empty case
        let bigint_committable_column = CommittableColumn::VarChar(Vec::new());
        assert_eq!(bigint_committable_column.len(), 0);
        assert!(bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::VarChar);

        let bigint_committable_column = CommittableColumn::VarChar(
            ["12", "34", "56"]
                .map(Into::<String>::into)
                .map(Into::<ArkScalar>::into)
                .map(Into::<[u64; 4]>::into)
                .into(),
        );
        assert_eq!(bigint_committable_column.len(), 3);
        assert!(!bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::VarChar);
    }

    #[test]
    fn we_can_get_type_and_length_of_scalar_column() {
        // empty case
        let bigint_committable_column = CommittableColumn::Scalar(Vec::new());
        assert_eq!(bigint_committable_column.len(), 0);
        assert!(bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::Scalar);

        let bigint_committable_column = CommittableColumn::Scalar(
            [12, 34, 56]
                .map(<ArkScalar>::from)
                .map(<[u64; 4]>::from)
                .into(),
        );
        assert_eq!(bigint_committable_column.len(), 3);
        assert!(!bigint_committable_column.is_empty());
        assert_eq!(bigint_committable_column.column_type(), ColumnType::Scalar);
    }

    #[test]
    fn we_can_convert_from_borrowing_bigint_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::BigInt(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::BigInt(&[]));

        let from_borrowed_column = CommittableColumn::from(&Column::BigInt(&[12, 34, 56]));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::BigInt(&[12, 34, 56])
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_int128_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::Int128(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::Int128(&[]));

        let from_borrowed_column = CommittableColumn::from(&Column::Int128(&[12, 34, 56]));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::Int128(&[12, 34, 56])
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_varchar_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::VarChar((&[], &[])));
        assert_eq!(from_borrowed_column, CommittableColumn::VarChar(Vec::new()));

        let varchar_data = ["12", "34", "56"];
        let scalars = varchar_data.map(ArkScalar::from);
        let from_borrowed_column =
            CommittableColumn::from(&Column::VarChar((&varchar_data, &scalars)));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::VarChar(scalars.map(<[u64; 4]>::from).into())
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_scalar_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::Scalar(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::Scalar(Vec::new()));

        let scalars = [12, 34, 56].map(ArkScalar::from);
        let from_borrowed_column = CommittableColumn::from(&Column::Scalar(&scalars));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::Scalar(scalars.map(<[u64; 4]>::from).into())
        );
    }

    #[test]
    fn we_can_convert_from_owned_bigint_column() {
        // empty case
        let owned_column = OwnedColumn::BigInt(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::BigInt(&[]));

        let owned_column = OwnedColumn::BigInt(vec![12, 34, 56]);
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::BigInt(&[12, 34, 56]));
    }

    #[test]
    fn we_can_convert_from_owned_int128_column() {
        // empty case
        let owned_column = OwnedColumn::Int128(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Int128(&[]));

        let owned_column = OwnedColumn::Int128(vec![12, 34, 56]);
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Int128(&[12, 34, 56]));
    }

    #[test]
    fn we_can_convert_from_owned_varchar_column() {
        // empty case
        let owned_column = OwnedColumn::VarChar(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::VarChar(Vec::new()));

        let strings = ["12", "34", "56"].map(String::from);
        let owned_column = OwnedColumn::VarChar(strings.to_vec());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(
            from_owned_column,
            CommittableColumn::VarChar(strings.map(ArkScalar::from).map(<[u64; 4]>::from).into())
        );
    }

    #[test]
    fn we_can_convert_from_owned_scalar_column() {
        // empty case
        let owned_column = OwnedColumn::Scalar(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Scalar(Vec::new()));

        let scalars = [12, 34, 56].map(ArkScalar::from);
        let owned_column = OwnedColumn::Scalar(scalars.to_vec());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(
            from_owned_column,
            CommittableColumn::Scalar(scalars.map(<[u64; 4]>::from).into())
        );
    }

    #[test]
    fn we_can_commit_to_bigint_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::BigInt(&[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56];
        let committable_column = CommittableColumn::BigInt(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    // Committing to Int128 columns is blocked by PROOF-772 without a workaround
    #[test]
    #[ignore]
    fn we_can_commit_to_int128_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::Int128(&[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56];
        let committable_column = CommittableColumn::Int128(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_varchar_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::VarChar(vec![]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = ["12", "34", "56"].map(String::from);
        let owned_column = OwnedColumn::VarChar(values.to_vec());
        let committable_column = CommittableColumn::from(&owned_column);

        let sequence_actual = Sequence::from(&committable_column);
        let scalars = values.map(ArkScalar::from).map(<[u64; 4]>::from);
        let sequence_expected = Sequence::from(scalars.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_scalar_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::Scalar(vec![]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56].map(ArkScalar::from);
        let owned_column = OwnedColumn::Scalar(values.to_vec());
        let committable_column = CommittableColumn::from(&owned_column);

        let sequence_actual = Sequence::from(&committable_column);
        let scalars = values.map(ArkScalar::from).map(<[u64; 4]>::from);
        let sequence_expected = Sequence::from(scalars.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }
}
