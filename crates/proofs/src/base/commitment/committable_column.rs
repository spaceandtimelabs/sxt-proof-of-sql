use crate::base::{
    database::{Column, ColumnType, OwnedColumn},
    math::decimal::Precision,
    ref_into::RefInto,
    scalar::Scalar,
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
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommittableColumn<'a> {
    /// Borrowed BigInt column, mapped to `i64`.
    BigInt(&'a [i64]),
    /// Borrowed Int128 column, mapped to `i128`.
    Int128(&'a [i128]),
    /// Borrowed Decimal75(precion, scale, column), mapped to 'i256'
    Decimal75(Precision, i8, Vec<[u64; 4]>),
    /// Column of big ints for committing to, hashed from a VarChar column.
    VarChar(Vec<[u64; 4]>),
    /// Column of big ints for committing to, montgomery-reduced from a Scalar column.
    Scalar(Vec<[u64; 4]>),
    /// Borrowed Bool column, mapped to `bool`.
    Boolean(&'a [bool]),
}

impl<'a> CommittableColumn<'a> {
    /// Returns the length of the column.
    pub fn len(&self) -> usize {
        match self {
            CommittableColumn::BigInt(col) => col.len(),
            CommittableColumn::VarChar(col) => col.len(),
            CommittableColumn::Int128(col) => col.len(),
            CommittableColumn::Scalar(col) => col.len(),
            CommittableColumn::Decimal75(_, _, col) => col.len(),
            CommittableColumn::Boolean(col) => col.len(),
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
            CommittableColumn::Decimal75(precision, scale, _) => {
                ColumnType::Decimal75(*precision, *scale)
            }
            CommittableColumn::VarChar(_) => ColumnType::VarChar,
            CommittableColumn::Scalar(_) => ColumnType::Scalar,
            CommittableColumn::Boolean(_) => {
                unimplemented!("Boolean columns are not supported yet")
            }
        }
    }
}

impl<'a, S: Scalar> From<&Column<'a, S>> for CommittableColumn<'a> {
    fn from(value: &Column<'a, S>) -> Self {
        match value {
            Column::BigInt(ints) => CommittableColumn::BigInt(ints),
            Column::Int128(ints) => CommittableColumn::Int128(ints),

            Column::VarChar((_, scalars)) => {
                let as_limbs: Vec<_> = scalars.iter().map(RefInto::<[u64; 4]>::ref_into).collect();
                CommittableColumn::VarChar(as_limbs)
            }
            Column::Scalar(scalars) => (scalars as &[_]).into(),
            Column::Decimal75(precision, scale, decimals) => {
                let as_limbs: Vec<_> = decimals.iter().map(RefInto::<[u64; 4]>::ref_into).collect();
                CommittableColumn::Decimal75(*precision, *scale, as_limbs)
            }
        }
    }
}

impl<'a, S: Scalar> From<&'a OwnedColumn<S>> for CommittableColumn<'a> {
    fn from(value: &'a OwnedColumn<S>) -> Self {
        match value {
            OwnedColumn::BigInt(ints) => (ints as &[_]).into(),
            OwnedColumn::Int128(ints) => (ints as &[_]).into(),
            OwnedColumn::Decimal75(precision, scale, decimals) => CommittableColumn::Decimal75(
                *precision,
                *scale,
                decimals
                    .iter()
                    .map(Into::<S>::into)
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
            OwnedColumn::VarChar(strings) => CommittableColumn::VarChar(
                strings
                    .iter()
                    .map(Into::<S>::into)
                    .map(Into::<[u64; 4]>::into)
                    .collect(),
            ),
            OwnedColumn::Scalar(scalars) => (scalars as &[_]).into(),
        }
    }
}

impl<'a> From<&'a [i64]> for CommittableColumn<'a> {
    fn from(value: &'a [i64]) -> Self {
        CommittableColumn::BigInt(value)
    }
}
impl<'a> From<&'a [i128]> for CommittableColumn<'a> {
    fn from(value: &'a [i128]) -> Self {
        CommittableColumn::Int128(value)
    }
}
impl<'a, S: Scalar> From<&'a [S]> for CommittableColumn<'a> {
    fn from(value: &'a [S]) -> Self {
        CommittableColumn::Scalar(value.iter().map(RefInto::<[u64; 4]>::ref_into).collect())
    }
}
impl<'a> From<&'a [bool]> for CommittableColumn<'a> {
    fn from(value: &'a [bool]) -> Self {
        CommittableColumn::Boolean(value)
    }
}

impl<'a, 'b> From<&'a CommittableColumn<'b>> for Sequence<'a> {
    fn from(value: &'a CommittableColumn<'b>) -> Self {
        match value {
            CommittableColumn::BigInt(ints) => Sequence::from(*ints),
            CommittableColumn::Int128(ints) => Sequence::from(*ints),
            CommittableColumn::Decimal75(_, _, limbs) => Sequence::from(limbs),
            CommittableColumn::VarChar(limbs) => Sequence::from(limbs),
            CommittableColumn::Scalar(limbs) => Sequence::from(limbs),
            CommittableColumn::Boolean(bools) => Sequence::from(*bools),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::ArkScalar;
    use blitzar::compute::compute_curve25519_commitments;
    use curve25519_dalek::ristretto::CompressedRistretto;

    #[test]
    fn we_can_convert_from_owned_decimal75_column_to_committable_column() {
        let decimals = vec![ArkScalar::from(-1), ArkScalar::from(1), ArkScalar::from(2)];
        let decimal_column = OwnedColumn::Decimal75(Precision::new(75).unwrap(), -1, decimals);

        let res_committable_column: CommittableColumn = (&decimal_column).into();
        let test_committable_column: CommittableColumn = CommittableColumn::Decimal75(
            Precision::new(75).unwrap(),
            -1,
            [-1, 1, 2]
                .map(<ArkScalar>::from)
                .map(<[u64; 4]>::from)
                .into(),
        );

        assert_eq!(res_committable_column, test_committable_column)
    }

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
    fn we_can_get_type_and_length_of_decimal_column() {
        // empty case
        let decimal_committable_column =
            CommittableColumn::Decimal75(Precision::new(1).unwrap(), 0, [].to_vec());
        assert_eq!(decimal_committable_column.len(), 0);
        assert!(decimal_committable_column.is_empty());
        assert_eq!(
            decimal_committable_column.column_type(),
            ColumnType::Decimal75(Precision::new(1).unwrap(), 0)
        );
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
    fn we_can_get_length_of_boolean_column() {
        // empty case
        let bool_committable_column = CommittableColumn::Boolean(&[]);
        assert_eq!(bool_committable_column.len(), 0);

        let bool_committable_column = CommittableColumn::Boolean(&[true, false, true]);
        assert_eq!(bool_committable_column.len(), 3);
        assert!(!bool_committable_column.is_empty());
    }

    #[test]
    #[should_panic]
    fn we_cannot_get_type_of_boolean_column() {
        // empty case
        let bool_committable_column = CommittableColumn::Boolean(&[]);
        let _ = bool_committable_column.column_type();
    }

    #[test]
    fn we_can_convert_from_borrowing_bigint_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::<ArkScalar>::BigInt(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::BigInt(&[]));

        let from_borrowed_column =
            CommittableColumn::from(&Column::<ArkScalar>::BigInt(&[12, 34, 56]));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::BigInt(&[12, 34, 56])
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_decimal_column() {
        // Define a non-empty array of ArkScalars
        let binding = vec![
            ArkScalar::from(-1),
            ArkScalar::from(34),
            ArkScalar::from(56),
        ];

        let precision = Precision::new(75).unwrap();
        let from_borrowed_column =
            CommittableColumn::from(&Column::Decimal75(precision, 0, &binding));

        let expected_decimals = binding
            .iter()
            .map(|&scalar| scalar.into())
            .collect::<Vec<[u64; 4]>>();

        assert_eq!(
            from_borrowed_column,
            CommittableColumn::Decimal75(Precision::new(75).unwrap(), 0, expected_decimals)
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_int128_column() {
        // empty case
        let from_borrowed_column = CommittableColumn::from(&Column::<ArkScalar>::Int128(&[]));
        assert_eq!(from_borrowed_column, CommittableColumn::Int128(&[]));

        let from_borrowed_column =
            CommittableColumn::from(&Column::<ArkScalar>::Int128(&[12, 34, 56]));
        assert_eq!(
            from_borrowed_column,
            CommittableColumn::Int128(&[12, 34, 56])
        );
    }

    #[test]
    fn we_can_convert_from_borrowing_varchar_column() {
        // empty case
        let from_borrowed_column =
            CommittableColumn::from(&Column::<ArkScalar>::VarChar((&[], &[])));
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
        let from_borrowed_column = CommittableColumn::from(&Column::<ArkScalar>::Scalar(&[]));
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
        let owned_column = OwnedColumn::<ArkScalar>::BigInt(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::BigInt(&[]));

        let owned_column = OwnedColumn::<ArkScalar>::BigInt(vec![12, 34, 56]);
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::BigInt(&[12, 34, 56]));
    }

    #[test]
    fn we_can_convert_from_owned_int128_column() {
        // empty case
        let owned_column = OwnedColumn::<ArkScalar>::Int128(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Int128(&[]));

        let owned_column = OwnedColumn::<ArkScalar>::Int128(vec![12, 34, 56]);
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::Int128(&[12, 34, 56]));
    }

    #[test]
    fn we_can_convert_from_owned_varchar_column() {
        // empty case
        let owned_column = OwnedColumn::<ArkScalar>::VarChar(Vec::new());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(from_owned_column, CommittableColumn::VarChar(Vec::new()));

        let strings = ["12", "34", "56"].map(String::from);
        let owned_column = OwnedColumn::<ArkScalar>::VarChar(strings.to_vec());
        let from_owned_column = CommittableColumn::from(&owned_column);
        assert_eq!(
            from_owned_column,
            CommittableColumn::VarChar(strings.map(ArkScalar::from).map(<[u64; 4]>::from).into())
        );
    }

    #[test]
    fn we_can_convert_from_owned_scalar_column() {
        // empty case
        let owned_column = OwnedColumn::<ArkScalar>::Scalar(Vec::new());
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
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56];
        let committable_column = CommittableColumn::BigInt(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_decimal_column_through_committable_column() {
        // empty case
        let committable_column =
            CommittableColumn::Decimal75(Precision::new(1).unwrap(), 0, [].to_vec());
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [
            ArkScalar::from(12),
            ArkScalar::from(34),
            ArkScalar::from(56),
        ]
        .map(<[u64; 4]>::from);
        let committable_column =
            CommittableColumn::Decimal75(Precision::new(1).unwrap(), 0, (values).to_vec());

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
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
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56];
        let committable_column = CommittableColumn::Int128(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
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
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = ["12", "34", "56"].map(String::from);
        let owned_column = OwnedColumn::<ArkScalar>::VarChar(values.to_vec());
        let committable_column = CommittableColumn::from(&owned_column);

        let sequence_actual = Sequence::from(&committable_column);
        let scalars = values.map(ArkScalar::from).map(<[u64; 4]>::from);
        let sequence_expected = Sequence::from(scalars.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
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
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [12, 34, 56].map(ArkScalar::from);
        let owned_column = OwnedColumn::Scalar(values.to_vec());
        let committable_column = CommittableColumn::from(&owned_column);

        let sequence_actual = Sequence::from(&committable_column);
        let scalars = values.map(ArkScalar::from).map(<[u64; 4]>::from);
        let sequence_expected = Sequence::from(scalars.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }

    #[test]
    fn we_can_commit_to_boolean_column_through_committable_column() {
        // empty case
        let committable_column = CommittableColumn::Boolean(&[]);
        let sequence = Sequence::from(&committable_column);
        let mut commitment_buffer = [CompressedRistretto::default()];
        compute_curve25519_commitments(&mut commitment_buffer, &[sequence], 0);
        assert_eq!(commitment_buffer[0], CompressedRistretto::default());

        // nonempty case
        let values = [true, false, true];
        let committable_column = CommittableColumn::Boolean(&values);

        let sequence_actual = Sequence::from(&committable_column);
        let sequence_expected = Sequence::from(values.as_slice());
        let mut commitment_buffer = [CompressedRistretto::default(); 2];
        compute_curve25519_commitments(
            &mut commitment_buffer,
            &[sequence_actual, sequence_expected],
            0,
        );
        assert_eq!(commitment_buffer[0], commitment_buffer[1]);
    }
}
