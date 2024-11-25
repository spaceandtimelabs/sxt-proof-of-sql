/// A column of data, with type included. This is simply a wrapper around `Vec<T>` for enumerated `T`.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow Array.
use super::{Column, ColumnType, OwnedColumnError, OwnedColumnResult};
use crate::base::{
    math::{
        decimal::Precision,
        permutation::{Permutation, PermutationError},
    },
    scalar::Scalar,
    slice_ops::inner_product_ref_cast,
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
#[non_exhaustive]
/// Supported types for [`OwnedColumn`]
pub enum OwnedColumn<S: Scalar> {
    /// Boolean columns
    Boolean(Vec<bool>),
    /// i8 columns
    TinyInt(Vec<i8>),
    /// i16 columns
    SmallInt(Vec<i16>),
    /// i32 columns
    Int(Vec<i32>),
    /// i64 columns
    BigInt(Vec<i64>),
    /// String columns
    VarChar(Vec<String>),
    /// i128 columns
    Int128(Vec<i128>),
    /// Decimal columns
    Decimal75(Precision, i8, Vec<S>),
    /// Scalar columns
    Scalar(Vec<S>),
    /// Timestamp columns
    TimestampTZ(PoSQLTimeUnit, PoSQLTimeZone, Vec<i64>),
}

impl<S: Scalar> OwnedColumn<S> {
    /// Compute the inner product of the column with a vector of scalars.
    pub(crate) fn inner_product(&self, vec: &[S]) -> S {
        match self {
            OwnedColumn::Boolean(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::TinyInt(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::SmallInt(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::Int(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => {
                inner_product_ref_cast(col, vec)
            }
            OwnedColumn::VarChar(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::Int128(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => {
                inner_product_ref_cast(col, vec)
            }
        }
    }

    /// Returns the length of the column.
    #[must_use]
    pub fn len(&self) -> usize {
        match self {
            OwnedColumn::Boolean(col) => col.len(),
            OwnedColumn::TinyInt(col) => col.len(),
            OwnedColumn::SmallInt(col) => col.len(),
            OwnedColumn::Int(col) => col.len(),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => col.len(),
            OwnedColumn::VarChar(col) => col.len(),
            OwnedColumn::Int128(col) => col.len(),
            OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => col.len(),
        }
    }

    /// Returns the column with its entries permutated
    pub fn try_permute(&self, permutation: &Permutation) -> Result<Self, PermutationError> {
        Ok(match self {
            OwnedColumn::Boolean(col) => OwnedColumn::Boolean(permutation.try_apply(col)?),
            OwnedColumn::TinyInt(col) => OwnedColumn::TinyInt(permutation.try_apply(col)?),
            OwnedColumn::SmallInt(col) => OwnedColumn::SmallInt(permutation.try_apply(col)?),
            OwnedColumn::Int(col) => OwnedColumn::Int(permutation.try_apply(col)?),
            OwnedColumn::BigInt(col) => OwnedColumn::BigInt(permutation.try_apply(col)?),
            OwnedColumn::VarChar(col) => OwnedColumn::VarChar(permutation.try_apply(col)?),
            OwnedColumn::Int128(col) => OwnedColumn::Int128(permutation.try_apply(col)?),
            OwnedColumn::Decimal75(precision, scale, col) => {
                OwnedColumn::Decimal75(*precision, *scale, permutation.try_apply(col)?)
            }
            OwnedColumn::Scalar(col) => OwnedColumn::Scalar(permutation.try_apply(col)?),
            OwnedColumn::TimestampTZ(tu, tz, col) => {
                OwnedColumn::TimestampTZ(*tu, *tz, permutation.try_apply(col)?)
            }
        })
    }

    /// Returns the sliced column.
    #[must_use]
    pub fn slice(&self, start: usize, end: usize) -> Self {
        match self {
            OwnedColumn::Boolean(col) => OwnedColumn::Boolean(col[start..end].to_vec()),
            OwnedColumn::TinyInt(col) => OwnedColumn::TinyInt(col[start..end].to_vec()),
            OwnedColumn::SmallInt(col) => OwnedColumn::SmallInt(col[start..end].to_vec()),
            OwnedColumn::Int(col) => OwnedColumn::Int(col[start..end].to_vec()),
            OwnedColumn::BigInt(col) => OwnedColumn::BigInt(col[start..end].to_vec()),
            OwnedColumn::VarChar(col) => OwnedColumn::VarChar(col[start..end].to_vec()),
            OwnedColumn::Int128(col) => OwnedColumn::Int128(col[start..end].to_vec()),
            OwnedColumn::Decimal75(precision, scale, col) => {
                OwnedColumn::Decimal75(*precision, *scale, col[start..end].to_vec())
            }
            OwnedColumn::Scalar(col) => OwnedColumn::Scalar(col[start..end].to_vec()),
            OwnedColumn::TimestampTZ(tu, tz, col) => {
                OwnedColumn::TimestampTZ(*tu, *tz, col[start..end].to_vec())
            }
        }
    }

    /// Returns true if the column is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            OwnedColumn::Boolean(col) => col.is_empty(),
            OwnedColumn::TinyInt(col) => col.is_empty(),
            OwnedColumn::SmallInt(col) => col.is_empty(),
            OwnedColumn::Int(col) => col.is_empty(),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => col.is_empty(),
            OwnedColumn::VarChar(col) => col.is_empty(),
            OwnedColumn::Int128(col) => col.is_empty(),
            OwnedColumn::Scalar(col) | OwnedColumn::Decimal75(_, _, col) => col.is_empty(),
        }
    }
    /// Returns the type of the column.
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        match self {
            OwnedColumn::Boolean(_) => ColumnType::Boolean,
            OwnedColumn::TinyInt(_) => ColumnType::TinyInt,
            OwnedColumn::SmallInt(_) => ColumnType::SmallInt,
            OwnedColumn::Int(_) => ColumnType::Int,
            OwnedColumn::BigInt(_) => ColumnType::BigInt,
            OwnedColumn::VarChar(_) => ColumnType::VarChar,
            OwnedColumn::Int128(_) => ColumnType::Int128,
            OwnedColumn::Scalar(_) => ColumnType::Scalar,
            OwnedColumn::Decimal75(precision, scale, _) => {
                ColumnType::Decimal75(*precision, *scale)
            }
            OwnedColumn::TimestampTZ(tu, tz, _) => ColumnType::TimestampTZ(*tu, *tz),
        }
    }

    /// Convert a slice of scalars to a vec of owned columns
    pub fn try_from_scalars(scalars: &[S], column_type: ColumnType) -> OwnedColumnResult<Self> {
        match column_type {
            ColumnType::Boolean => Ok(OwnedColumn::Boolean(
                scalars
                    .iter()
                    .map(|s| -> Result<bool, _> { TryInto::<bool>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::TinyInt => Ok(OwnedColumn::TinyInt(
                scalars
                    .iter()
                    .map(|s| -> Result<i8, _> { TryInto::<i8>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::SmallInt => Ok(OwnedColumn::SmallInt(
                scalars
                    .iter()
                    .map(|s| -> Result<i16, _> { TryInto::<i16>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::Int => Ok(OwnedColumn::Int(
                scalars
                    .iter()
                    .map(|s| -> Result<i32, _> { TryInto::<i32>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::BigInt => Ok(OwnedColumn::BigInt(
                scalars
                    .iter()
                    .map(|s| -> Result<i64, _> { TryInto::<i64>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::Int128 => Ok(OwnedColumn::Int128(
                scalars
                    .iter()
                    .map(|s| -> Result<i128, _> { TryInto::<i128>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?,
            )),
            ColumnType::Scalar => Ok(OwnedColumn::Scalar(scalars.to_vec())),
            ColumnType::Decimal75(precision, scale) => {
                Ok(OwnedColumn::Decimal75(precision, scale, scalars.to_vec()))
            }
            ColumnType::TimestampTZ(tu, tz) => {
                let raw_values: Vec<i64> = scalars
                    .iter()
                    .map(|s| -> Result<i64, _> { TryInto::<i64>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?;
                Ok(OwnedColumn::TimestampTZ(tu, tz, raw_values))
            }
            // Can not convert scalars to VarChar
            ColumnType::VarChar => Err(OwnedColumnError::TypeCastError {
                from_type: ColumnType::Scalar,
                to_type: ColumnType::VarChar,
            }),
        }
    }

    /// Convert a slice of option scalars to a vec of owned columns
    pub fn try_from_option_scalars(
        option_scalars: &[Option<S>],
        column_type: ColumnType,
    ) -> OwnedColumnResult<Self> {
        let scalars = option_scalars
            .iter()
            .copied()
            .collect::<Option<Vec<_>>>()
            .ok_or(OwnedColumnError::Unsupported {
                error: "NULL is not supported yet".to_string(),
            })?;
        Self::try_from_scalars(&scalars, column_type)
    }

    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i8], panicking if it is not.
    pub fn i8_iter(&self) -> impl Iterator<Item = &i8> {
        match self {
            OwnedColumn::TinyInt(col) => col.iter(),
            _ => panic!("Expected TinyInt column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i16], panicking if it is not.
    pub fn i16_iter(&self) -> impl Iterator<Item = &i16> {
        match self {
            OwnedColumn::SmallInt(col) => col.iter(),
            _ => panic!("Expected SmallInt column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i32], panicking if it is not.
    pub fn i32_iter(&self) -> impl Iterator<Item = &i32> {
        match self {
            OwnedColumn::Int(col) => col.iter(),
            _ => panic!("Expected Int column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i64], panicking if it is not.
    pub fn i64_iter(&self) -> impl Iterator<Item = &i64> {
        match self {
            OwnedColumn::TimestampTZ(_, _, col) | OwnedColumn::BigInt(col) => col.iter(),
            _ => panic!("Expected TimestampTZ or BigInt column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [i128], panicking if it is not.
    pub fn i128_iter(&self) -> impl Iterator<Item = &i128> {
        match self {
            OwnedColumn::Int128(col) => col.iter(),
            _ => panic!("Expected Int128 column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [bool], panicking if it is not.
    pub fn bool_iter(&self) -> impl Iterator<Item = &bool> {
        match self {
            OwnedColumn::Boolean(col) => col.iter(),
            _ => panic!("Expected Boolean column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is a [Scalar], panicking if it is not.
    pub fn scalar_iter(&self) -> impl Iterator<Item = &S> {
        match self {
            OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => col.iter(),
            _ => panic!("Expected Scalar or Decimal75 column"),
        }
    }
    #[cfg(test)]
    /// Returns an iterator over the raw data of the column
    /// assuming the underlying type is [String], panicking if it is not.
    pub fn string_iter(&self) -> impl Iterator<Item = &String> {
        match self {
            OwnedColumn::VarChar(col) => col.iter(),
            _ => panic!("Expected VarChar column"),
        }
    }
}

impl<'a, S: Scalar> From<&Column<'a, S>> for OwnedColumn<S> {
    fn from(col: &Column<'a, S>) -> Self {
        match col {
            Column::Boolean(col) => OwnedColumn::Boolean(col.to_vec()),
            Column::TinyInt(col) => OwnedColumn::TinyInt(col.to_vec()),
            Column::SmallInt(col) => OwnedColumn::SmallInt(col.to_vec()),
            Column::Int(col) => OwnedColumn::Int(col.to_vec()),
            Column::BigInt(col) => OwnedColumn::BigInt(col.to_vec()),
            Column::VarChar((col, _)) => {
                OwnedColumn::VarChar(col.iter().map(ToString::to_string).collect())
            }
            Column::Int128(col) => OwnedColumn::Int128(col.to_vec()),
            Column::Decimal75(precision, scale, col) => {
                OwnedColumn::Decimal75(*precision, *scale, col.to_vec())
            }
            Column::Scalar(col) => OwnedColumn::Scalar(col.to_vec()),
            Column::TimestampTZ(tu, tz, col) => OwnedColumn::TimestampTZ(*tu, *tz, col.to_vec()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::{math::decimal::Precision, scalar::test_scalar::TestScalar};
    use alloc::vec;
    use bumpalo::Bump;

    #[test]
    fn we_can_slice_a_column() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1, 2, 3, 4, 5]);
        assert_eq!(col.slice(1, 4), OwnedColumn::Int128(vec![2, 3, 4]));
    }

    #[test]
    fn we_can_permute_a_column() {
        let col: OwnedColumn<TestScalar> = OwnedColumn::Int128(vec![1, 2, 3, 4, 5]);
        let permutation = Permutation::try_new(vec![1, 3, 4, 0, 2]).unwrap();
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::Int128(vec![2, 4, 5, 1, 3])
        );
    }

    #[test]
    fn we_can_convert_columns_to_owned_columns_round_trip() {
        let alloc = Bump::new();
        // Integers
        let col: Column<'_, TestScalar> = Column::Int128(&[1, 2, 3, 4, 5]);
        let owned_col: OwnedColumn<TestScalar> = (&col).into();
        assert_eq!(owned_col, OwnedColumn::Int128(vec![1, 2, 3, 4, 5]));
        let new_col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, new_col);

        // Booleans
        let col: Column<'_, TestScalar> = Column::Boolean(&[true, false, true, false, true]);
        let owned_col: OwnedColumn<TestScalar> = (&col).into();
        assert_eq!(
            owned_col,
            OwnedColumn::Boolean(vec![true, false, true, false, true])
        );
        let new_col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, new_col);

        // Strings
        let strs = [
            "Space and Time",
            "מרחב וזמן",
            "Χώρος και Χρόνος",
            "Տարածություն և ժամանակ",
            "ቦታ እና ጊዜ",
            "სივრცე და დრო",
        ];
        let scalars = strs.iter().map(TestScalar::from).collect::<Vec<_>>();
        let col: Column<'_, TestScalar> = Column::VarChar((&strs, &scalars));
        let owned_col: OwnedColumn<TestScalar> = (&col).into();
        assert_eq!(
            owned_col,
            OwnedColumn::VarChar(
                strs.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
            )
        );
        let new_col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, new_col);

        // Decimals
        let scalars: Vec<TestScalar> = [1, 2, 3, 4, 5].iter().map(TestScalar::from).collect();
        let col: Column<'_, TestScalar> =
            Column::Decimal75(Precision::new(75).unwrap(), -128, &scalars);
        let owned_col: OwnedColumn<TestScalar> = (&col).into();
        assert_eq!(
            owned_col,
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), -128, scalars.clone())
        );
        let new_col = Column::<TestScalar>::from_owned_column(&owned_col, &alloc);
        assert_eq!(col, new_col);
    }

    #[test]
    fn we_can_convert_scalars_to_owned_columns() {
        // Int
        let scalars = [1, 2, 3, 4, 5]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::Int128;
        let owned_col = OwnedColumn::try_from_scalars(&scalars, column_type).unwrap();
        assert_eq!(owned_col, OwnedColumn::Int128(vec![1, 2, 3, 4, 5]));

        // Boolean
        let scalars = [true, false, true, false, true]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::Boolean;
        let owned_col = OwnedColumn::try_from_scalars(&scalars, column_type).unwrap();
        assert_eq!(
            owned_col,
            OwnedColumn::Boolean(vec![true, false, true, false, true])
        );

        // Decimal
        let scalars = [1, 2, 3, 4, 5]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::Decimal75(Precision::new(75).unwrap(), -128);
        let owned_col = OwnedColumn::try_from_scalars(&scalars, column_type).unwrap();
        assert_eq!(
            owned_col,
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), -128, scalars)
        );
    }

    #[test]
    fn we_cannot_convert_scalars_to_owned_columns_if_varchar() {
        let scalars = ["a", "b", "c", "d", "e"]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::VarChar;
        let res = OwnedColumn::try_from_scalars(&scalars, column_type);
        assert!(matches!(res, Err(OwnedColumnError::TypeCastError { .. })));
    }

    #[test]
    fn we_cannot_convert_scalars_to_owned_columns_if_overflow() {
        // Int
        let scalars = [i128::MAX, i128::MAX, i128::MAX, i128::MAX, i128::MAX]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::BigInt;
        let res = OwnedColumn::try_from_scalars(&scalars, column_type);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));

        // Boolean
        let scalars = [i128::MAX, i128::MAX, i128::MAX, i128::MAX, i128::MAX]
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let column_type = ColumnType::Boolean;
        let res = OwnedColumn::try_from_scalars(&scalars, column_type);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));
    }

    #[test]
    fn we_can_convert_option_scalars_to_owned_columns() {
        // Int
        let option_scalars = [Some(1), Some(2), Some(3), Some(4), Some(5)]
            .iter()
            .map(|s| s.map(TestScalar::from))
            .collect::<Vec<_>>();
        let column_type = ColumnType::Int128;
        let owned_col = OwnedColumn::try_from_option_scalars(&option_scalars, column_type).unwrap();
        assert_eq!(owned_col, OwnedColumn::Int128(vec![1, 2, 3, 4, 5]));

        // Boolean
        let option_scalars = [Some(true), Some(false), Some(true), Some(false), Some(true)]
            .iter()
            .map(|s| s.map(TestScalar::from))
            .collect::<Vec<_>>();
        let column_type = ColumnType::Boolean;
        let owned_col = OwnedColumn::try_from_option_scalars(&option_scalars, column_type).unwrap();
        assert_eq!(
            owned_col,
            OwnedColumn::Boolean(vec![true, false, true, false, true])
        );

        // Decimal
        let option_scalars = [Some(1), Some(2), Some(3), Some(4), Some(5)]
            .iter()
            .map(|s| s.map(TestScalar::from))
            .collect::<Vec<_>>();
        let scalars = [1, 2, 3, 4, 5]
            .iter()
            .map(|&i| TestScalar::from(i))
            .collect::<Vec<_>>();
        let column_type = ColumnType::Decimal75(Precision::new(75).unwrap(), 127);
        let owned_col = OwnedColumn::try_from_option_scalars(&option_scalars, column_type).unwrap();
        assert_eq!(
            owned_col,
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), 127, scalars)
        );
    }

    #[test]
    fn we_cannot_convert_option_scalars_to_owned_columns_if_varchar() {
        let option_scalars = ["a", "b", "c", "d", "e"]
            .iter()
            .map(|s| Some(TestScalar::from(*s)))
            .collect::<Vec<_>>();
        let column_type = ColumnType::VarChar;
        let res = OwnedColumn::try_from_option_scalars(&option_scalars, column_type);
        assert!(matches!(res, Err(OwnedColumnError::TypeCastError { .. })));
    }

    #[test]
    fn we_cannot_convert_option_scalars_to_owned_columns_if_overflow() {
        // Int
        let option_scalars = [
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
        ]
        .iter()
        .map(|s| s.map(TestScalar::from))
        .collect::<Vec<_>>();
        let column_type = ColumnType::BigInt;
        let res = OwnedColumn::try_from_option_scalars(&option_scalars, column_type);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));

        // Boolean
        let option_scalars = [
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
            Some(i128::MAX),
        ]
        .iter()
        .map(|s| s.map(TestScalar::from))
        .collect::<Vec<_>>();
        let column_type = ColumnType::Boolean;
        let res = OwnedColumn::try_from_option_scalars(&option_scalars, column_type);
        assert!(matches!(
            res,
            Err(OwnedColumnError::ScalarConversionError { .. })
        ));
    }

    #[test]
    fn we_cannot_convert_option_scalars_to_owned_columns_if_none() {
        // Int
        let option_scalars = [Some(1), Some(2), None, Some(4), Some(5)]
            .iter()
            .map(|s| s.map(TestScalar::from))
            .collect::<Vec<_>>();
        let column_type = ColumnType::Int128;
        let res = OwnedColumn::try_from_option_scalars(&option_scalars, column_type);
        assert!(matches!(res, Err(OwnedColumnError::Unsupported { .. })));

        // Boolean
        let option_scalars = [Some(true), Some(false), None, Some(false), Some(true)]
            .iter()
            .map(|s| s.map(TestScalar::from))
            .collect::<Vec<_>>();
        let column_type = ColumnType::Boolean;
        let res = OwnedColumn::try_from_option_scalars(&option_scalars, column_type);
        assert!(matches!(res, Err(OwnedColumnError::Unsupported { .. })));
    }
}
