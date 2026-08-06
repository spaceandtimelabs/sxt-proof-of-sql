use super::{ColumnType, LiteralValue, OwnedColumn};
use crate::base::{
    math::decimal::Precision,
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::{Scalar, ScalarExt},
    slice_ops::slice_cast_with,
};
use alloc::vec::Vec;
use bumpalo::Bump;

/// Represents a read-only view of a column in an in-memory,
/// column-oriented database.
///
/// Note: The types here should correspond to native SQL database types.
/// See `<https://ignite.apache.org/docs/latest/sql-reference/data-types>` for
/// a description of the native types used by Apache Ignite.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[non_exhaustive]
pub enum Column<'a, S: Scalar> {
    /// Boolean columns
    Boolean(&'a [bool]),
    /// u8 columns
    Uint8(&'a [u8]),
    /// i8 columns
    TinyInt(&'a [i8]),
    /// i16 columns
    SmallInt(&'a [i16]),
    /// i32 columns
    Int(&'a [i32]),
    /// i64 columns
    BigInt(&'a [i64]),
    /// i128 columns
    Int128(&'a [i128]),
    /// String columns
    ///  - the first element maps to the str values.
    ///  - the second element maps to the str hashes (see [`crate::base::scalar::Scalar`]).
    VarChar((&'a [&'a str], &'a [S])),
    /// Decimal columns with a max width of 252 bits
    ///  - the backing store maps to the type `S`
    Decimal75(Precision, i8, &'a [S]),
    /// Timestamp columns with timezone
    /// - the first element maps to the stored `TimeUnit`
    /// - the second element maps to a timezone
    /// - the third element maps to columns of timeunits since unix epoch
    TimestampTZ(PoSQLTimeUnit, PoSQLTimeZone, &'a [i64]),
    /// Scalar columns
    Scalar(&'a [S]),
    /// Variable length binary columns
    VarBinary((&'a [&'a [u8]], &'a [S])),
}

impl<'a, S: Scalar> Column<'a, S> {
    /// Provides the column type associated with the column
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Boolean(_) => ColumnType::Boolean,
            Self::Uint8(_) => ColumnType::Uint8,
            Self::TinyInt(_) => ColumnType::TinyInt,
            Self::SmallInt(_) => ColumnType::SmallInt,
            Self::Int(_) => ColumnType::Int,
            Self::BigInt(_) => ColumnType::BigInt,
            Self::VarChar(_) => ColumnType::VarChar,
            Self::Int128(_) => ColumnType::Int128,
            Self::Scalar(_) => ColumnType::Scalar,
            Self::Decimal75(precision, scale, _) => ColumnType::Decimal75(*precision, *scale),
            Self::TimestampTZ(time_unit, timezone, _) => {
                ColumnType::TimestampTZ(*time_unit, *timezone)
            }
            Self::VarBinary(..) => ColumnType::VarBinary,
        }
    }
    /// Returns the length of the column.
    /// # Panics
    /// this function requires that `col` and `scals` have the same length.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            Self::Boolean(col) => col.len(),
            Self::Uint8(col) => col.len(),
            Self::TinyInt(col) => col.len(),
            Self::SmallInt(col) => col.len(),
            Self::Int(col) => col.len(),
            Self::BigInt(col) | Self::TimestampTZ(_, _, col) => col.len(),
            Self::VarChar((col, scals)) => {
                assert_eq!(col.len(), scals.len());
                col.len()
            }
            Self::VarBinary((col, scals)) => {
                assert_eq!(col.len(), scals.len());
                col.len()
            }
            Self::Int128(col) => col.len(),
            Self::Scalar(col) | Self::Decimal75(_, _, col) => col.len(),
        }
    }
    /// Returns `true` if the column has no elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Generate a constant column from a literal value with a given length
    pub fn from_literal_with_length(
        literal: &LiteralValue,
        length: usize,
        alloc: &'a Bump,
    ) -> Self {
        match literal {
            LiteralValue::Boolean(value) => {
                Column::Boolean(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Uint8(value) => {
                Column::Uint8(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::TinyInt(value) => {
                Column::TinyInt(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::SmallInt(value) => {
                Column::SmallInt(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Int(value) => Column::Int(alloc.alloc_slice_fill_copy(length, *value)),
            LiteralValue::BigInt(value) => {
                Column::BigInt(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Int128(value) => {
                Column::Int128(alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::Scalar(value) => {
                Column::Scalar(alloc.alloc_slice_fill_copy(length, (*value).into()))
            }
            LiteralValue::Decimal75(precision, scale, value) => Column::Decimal75(
                *precision,
                *scale,
                alloc.alloc_slice_fill_copy(length, value.into_scalar()),
            ),
            LiteralValue::TimeStampTZ(tu, tz, value) => {
                Column::TimestampTZ(*tu, *tz, alloc.alloc_slice_fill_copy(length, *value))
            }
            LiteralValue::VarChar(string) => Column::VarChar((
                alloc.alloc_slice_fill_with(length, |_| alloc.alloc_str(string) as &str),
                alloc.alloc_slice_fill_copy(length, S::from_str_via_hash(string)),
            )),
            LiteralValue::VarBinary(bytes) => {
                // Convert the bytes to a slice of bytes references
                let bytes_slice = alloc
                    .alloc_slice_fill_with(length, |_| alloc.alloc_slice_copy(bytes) as &[_]);

                // Convert the bytes to scalars using from_byte_slice_via_hash
                let scalars =
                    alloc.alloc_slice_fill_copy(length, S::from_byte_slice_via_hash(bytes));

                Column::VarBinary((bytes_slice, scalars))
            }
        }
    }

    /// Generate a `Int128` `rho` column [0, 1, 2, ..., length - 1]
    pub fn rho(length: usize, alloc: &'a Bump) -> Self {
        let raw_rho = (0..length as i128).collect::<Vec<_>>();
        let rho = alloc.alloc_slice_copy(raw_rho.as_slice());
        Column::<S>::Int128(rho as &[_])
    }

    /// Convert an `OwnedColumn` to a `Column`
    pub fn from_owned_column(owned_column: &'a OwnedColumn<S>, alloc: &'a Bump) -> Self {
        match owned_column {
            OwnedColumn::Boolean(col) => Column::Boolean(col.as_slice()),
            OwnedColumn::Uint8(col) => Column::Uint8(col.as_slice()),
            OwnedColumn::TinyInt(col) => Column::TinyInt(col.as_slice()),
            OwnedColumn::SmallInt(col) => Column::SmallInt(col.as_slice()),
            OwnedColumn::Int(col) => Column::Int(col.as_slice()),
            OwnedColumn::BigInt(col) => Column::BigInt(col.as_slice()),
            OwnedColumn::Int128(col) => Column::Int128(col.as_slice()),
            OwnedColumn::Decimal75(precision, scale, col) => {
                Column::Decimal75(*precision, *scale, col.as_slice())
            }
            OwnedColumn::Scalar(col) => Column::Scalar(col.as_slice()),
            OwnedColumn::VarChar(col) => {
                let scalars = col
                    .iter()
                    .map(|s| S::from_str_via_hash(s))
                    .collect::<Vec<_>>();
                let strs = col
                    .iter()
                    .map(|s| s.as_str() as &'a str)
                    .collect::<Vec<_>>();
                Column::VarChar((
                    alloc.alloc_slice_clone(strs.as_slice()),
                    alloc.alloc_slice_copy(scalars.as_slice()),
                ))
            }
            OwnedColumn::VarBinary(col) => {
                let scalars = col
                    .iter()
                    .map(|b| S::from_byte_slice_via_hash(b))
                    .collect::<Vec<_>>();
                let bytes = col.iter().map(|s| s as &'a [u8]).collect::<Vec<_>>();
                Column::VarBinary((
                    alloc.alloc_slice_clone(&bytes),
                    alloc.alloc_slice_copy(scalars.as_slice()),
                ))
            }
            OwnedColumn::TimestampTZ(tu, tz, col) => Column::TimestampTZ(*tu, *tz, col.as_slice()),
        }
    }

    /// Returns the column as a slice of booleans if it is a boolean column. Otherwise, returns None.
    pub(crate) fn as_boolean(&self) -> Option<&'a [bool]> {
        match self {
            Self::Boolean(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of u8 if it is a uint8 column. Otherwise, returns None.
    pub(crate) fn as_uint8(&self) -> Option<&'a [u8]> {
        match self {
            Self::Uint8(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of i8 if it is a tinyint column. Otherwise, returns None.
    pub(crate) fn as_tinyint(&self) -> Option<&'a [i8]> {
        match self {
            Self::TinyInt(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of i16 if it is a smallint column. Otherwise, returns None.
    pub(crate) fn as_smallint(&self) -> Option<&'a [i16]> {
        match self {
            Self::SmallInt(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of i32 if it is an int column. Otherwise, returns None.
    pub(crate) fn as_int(&self) -> Option<&'a [i32]> {
        match self {
            Self::Int(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of i64 if it is a bigint column. Otherwise, returns None.
    pub(crate) fn as_bigint(&self) -> Option<&'a [i64]> {
        match self {
            Self::BigInt(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of i128 if it is an int128 column. Otherwise, returns None.
    pub(crate) fn as_int128(&self) -> Option<&'a [i128]> {
        match self {
            Self::Int128(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of scalars if it is a scalar column. Otherwise, returns None.
    pub(crate) fn as_scalar(&self) -> Option<&'a [S]> {
        match self {
            Self::Scalar(col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of scalars if it is a decimal75 column. Otherwise, returns None.
    pub(crate) fn as_decimal75(&self) -> Option<&'a [S]> {
        match self {
            Self::Decimal75(_, _, col) => Some(col),
            _ => None,
        }
    }

    /// Returns the column as a slice of strings and a slice of scalars if it is a varchar column. Otherwise, returns None.
    pub(crate) fn as_varchar(&self) -> Option<(&'a [&'a str], &'a [S])> {
        match self {
            Self::VarChar((col, scals)) => Some((col, scals)),
            _ => None,
        }
    }

    /// Returns the column as a slice of strings and a slice of scalars if it is a varchar column. Otherwise, returns None.
    pub(crate) fn as_varbinary(&self) -> Option<(&'a [&'a [u8]], &'a [S])> {
        match self {
            Self::VarBinary((col, scals)) => Some((col, scals)),
            _ => None,
        }
    }

    /// Returns the column as a slice of i64 if it is a timestamp column. Otherwise, returns None.
    pub(crate) fn as_timestamptz(&self) -> Option<&'a [i64]> {
        match self {
            Self::TimestampTZ(_, _, col) => Some(col),
            _ => None,
        }
    }

    /// Returns element at index as scalar
    ///
    /// Note that if index is out of bounds, this function will return None
    pub(crate) fn scalar_at(&self, index: usize) -> Option<S> {
        (index < self.len()).then_some(match self {
            Self::Boolean(col) => S::from(col[index]),
            Self::Uint8(col) => S::from(col[index]),
            Self::TinyInt(col) => S::from(col[index]),
            Self::SmallInt(col) => S::from(col[index]),
            Self::Int(col) => S::from(col[index]),
            Self::BigInt(col) | Self::TimestampTZ(_, _, col) => S::from(col[index]),
            Self::Int128(col) => S::from(col[index]),
            Self::Scalar(col) | Self::Decimal75(_, _, col) => col[index],
            Self::VarChar((_, scals)) | Self::VarBinary((_, scals)) => scals[index],
        })
    }

    /// Convert a column to a vector of Scalar values
    #[tracing::instrument(name = "Column::to_scalar", level = "debug", skip_all)]
    pub(crate) fn to_scalar(self) -> Vec<S> {
        match self {
            Self::Boolean(col) => slice_cast_with(col, |b| S::from(b)),
            Self::Decimal75(_, _, col) => slice_cast_with(col, |s| *s),
            Self::VarChar((_, values)) => slice_cast_with(values, |s| *s),
            Self::VarBinary((_, values)) => slice_cast_with(values, |s| *s),
            Self::Uint8(col) => slice_cast_with(col, |i| S::from(i)),
            Self::TinyInt(col) => slice_cast_with(col, |i| S::from(i)),
            Self::SmallInt(col) => slice_cast_with(col, |i| S::from(i)),
            Self::Int(col) => slice_cast_with(col, |i| S::from(i)),
            Self::BigInt(col) => slice_cast_with(col, |i| S::from(i)),
            Self::Int128(col) => slice_cast_with(col, |i| S::from(i)),
            Self::Scalar(col) => slice_cast_with(col, |i| S::from(i)),
            Self::TimestampTZ(_, _, col) => slice_cast_with(col, |i| S::from(i)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{base::scalar::test_scalar::TestScalar, proof_primitive::dory::DoryScalar};
    use alloc::{string::String, vec};

    #[test]
    fn we_can_get_the_len_of_a_column() {
        let precision = 10;
        let scale = 2;

        let scalar_values = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];

        // Test non-empty columns
        let column = Column::<DoryScalar>::Boolean(&[true, false, true]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<DoryScalar>::TinyInt(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<TestScalar>::SmallInt(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<TestScalar>::Int(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<TestScalar>::BigInt(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::VarChar((&["a", "b", "c"], &scalar_values));
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::<DoryScalar>::Int128(&[1, 2, 3]);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let column = Column::Scalar(&scalar_values);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        let decimal_data = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];

        let precision = Precision::new(precision).unwrap();
        let column = Column::Decimal75(precision, scale, &decimal_data);
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());

        // Test empty columns
        let column = Column::<DoryScalar>::Boolean(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::TinyInt(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<TestScalar>::SmallInt(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<TestScalar>::Int(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<TestScalar>::BigInt(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::VarChar((&[], &[]));
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<TestScalar>::Int128(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column = Column::<DoryScalar>::Scalar(&[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());

        let column: Column<'_, TestScalar> = Column::Decimal75(precision, scale, &[]);
        assert_eq!(column.len(), 0);
        assert!(column.is_empty());
    }

    #[test]
    fn we_can_convert_owned_columns_to_columns_round_trip() {
        let alloc = Bump::new();
        // Integers
        let owned_col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1, 2, 3, 4, 5]);
        let col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, Column::Int128(&[1, 2, 3, 4, 5]));
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);

        // Booleans
        let owned_col: OwnedColumn<TestScalar> =
            OwnedColumn::Boolean(vec![true, false, true, false, true]);
        let col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, Column::Boolean(&[true, false, true, false, true]));
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);

        // Strings
        let strs = [
            "Space and Time",
            "Tér és Idő",
            "Пространство и время",
            "Spațiu și Timp",
            "Spazju u Ħin",
        ];
        let scalars = strs.iter().map(TestScalar::from).collect::<Vec<_>>();
        let owned_col = OwnedColumn::VarChar(
            strs.iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        );
        let col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, Column::VarChar((&strs, &scalars)));
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);

        // Decimals
        let scalars: Vec<TestScalar> = [1, 2, 3, 4, 5].iter().map(TestScalar::from).collect();
        let owned_col: OwnedColumn<TestScalar> =
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), 127, scalars.clone());
        let col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(
            col,
            Column::Decimal75(Precision::new(75).unwrap(), 127, &scalars)
        );
        let new_owned_col = (&col).into();
        assert_eq!(owned_col, new_owned_col);
    }

    #[test]
    fn we_can_get_the_data_size_of_a_column() {
        let column = Column::<DoryScalar>::Boolean(&[true, false, true]);
        assert_eq!(column.column_type().byte_size(), 1);
        assert_eq!(column.column_type().bit_size(), 8);

        let column = Column::<TestScalar>::TinyInt(&[1, 2, 3, 4]);
        assert_eq!(column.column_type().byte_size(), 1);
        assert_eq!(column.column_type().bit_size(), 8);

        let column = Column::<TestScalar>::SmallInt(&[1, 2, 3, 4]);
        assert_eq!(column.column_type().byte_size(), 2);
        assert_eq!(column.column_type().bit_size(), 16);

        let column = Column::<TestScalar>::Int(&[1, 2, 3]);
        assert_eq!(column.column_type().byte_size(), 4);
        assert_eq!(column.column_type().bit_size(), 32);

        let column = Column::<TestScalar>::BigInt(&[1]);
        assert_eq!(column.column_type().byte_size(), 8);
        assert_eq!(column.column_type().bit_size(), 64);

        let column = Column::<DoryScalar>::Int128(&[1, 2]);
        assert_eq!(column.column_type().byte_size(), 16);
        assert_eq!(column.column_type().bit_size(), 128);

        let scalar_values = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];

        let column = Column::VarChar((&["a", "b", "c", "d", "e"], &scalar_values));
        assert_eq!(column.column_type().byte_size(), 32);
        assert_eq!(column.column_type().bit_size(), 256);

        let column = Column::Scalar(&scalar_values);
        assert_eq!(column.column_type().byte_size(), 32);
        assert_eq!(column.column_type().bit_size(), 256);

        let precision = 10;
        let scale = 2;
        let decimal_data = [
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];

        let precision = Precision::new(precision).unwrap();
        let column = Column::Decimal75(precision, scale, &decimal_data);
        assert_eq!(column.column_type().byte_size(), 32);
        assert_eq!(column.column_type().bit_size(), 256);

        let column: Column<'_, DoryScalar> =
            Column::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[1, 2, 3]);
        assert_eq!(column.column_type().byte_size(), 8);
        assert_eq!(column.column_type().bit_size(), 64);
    }

    #[test]
    fn we_can_get_length_of_varbinary_column() {
        let raw_bytes: &[&[u8]] = &[b"foo", b"bar", b""];
        let scalars: Vec<TestScalar> = raw_bytes
            .iter()
            .map(|b| TestScalar::from_le_bytes_mod_order(b))
            .collect();

        let column = Column::VarBinary((raw_bytes, &scalars));
        assert_eq!(column.len(), 3);
        assert!(!column.is_empty());
        assert_eq!(column.column_type(), ColumnType::VarBinary);
    }

    #[test]
    fn we_can_convert_varbinary_owned_column_to_column_and_back() {
        use bumpalo::Bump;
        let alloc = Bump::new();

        let owned_varbinary = OwnedColumn::VarBinary(vec![b"abc".to_vec(), b"xyz".to_vec()]);

        let column = Column::<TestScalar>::from_owned_column(&owned_varbinary, &alloc);
        match column {
            Column::VarBinary((bytes, scalars)) => {
                assert_eq!(bytes.len(), 2);
                assert_eq!(scalars.len(), 2);
                assert_eq!(bytes[0], b"abc");
                assert_eq!(bytes[1], b"xyz");
            }
            _ => panic!("Expected VarBinary column"),
        }

        let round_trip_owned: OwnedColumn<TestScalar> = (&column).into();
        assert_eq!(owned_varbinary, round_trip_owned);
    }
}
