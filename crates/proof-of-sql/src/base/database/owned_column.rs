use super::{
    Column, ColumnCoercionError, ColumnType, NullableColumn, OwnedColumnError, OwnedColumnResult,
};
use crate::base::{
    database::table::TableError,
    math::{
        decimal::Precision,
        permutation::{Permutation, PermutationError},
    },
    posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    scalar::Scalar,
    slice_ops::{inner_product_ref_cast, inner_product_with_bytes},
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::fmt::Debug;
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// A column of data, with type included. This is simply a wrapper around `Vec<T>` for enumerated `T`.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow Array.
#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
/// Supported types for [`OwnedColumn`]
pub enum OwnedColumn<S: Scalar> {
    /// Boolean columns
    Boolean(Vec<bool>),
    /// u8 columns
    Uint8(Vec<u8>),
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
    /// Variable length binary columns
    VarBinary(Vec<Vec<u8>>),
    /// i128 columns
    Int128(Vec<i128>),
    /// Decimal columns
    #[cfg_attr(test, proptest(skip))]
    Decimal75(Precision, i8, Vec<S>),
    /// Scalar columns
    #[cfg_attr(test, proptest(skip))]
    Scalar(Vec<S>),
    /// Timestamp columns
    #[cfg_attr(test, proptest(skip))]
    TimestampTZ(PoSQLTimeUnit, PoSQLTimeZone, Vec<i64>),
}

/// A nullable column that contains a values `OwnedColumn`,
/// and an optional boolean presence vector.
///
/// When `presence` is `None`, the column is not nullable (all values are present).
/// When `presence` contains a boolean vector, its length must match the length of the values column,
/// and a `true` value indicates the presence of a value at the corresponding index,
/// while a `false` value indicates NULL.
///
/// This implementation follows the `PostgreSQL` approach to NULL values by using
/// a separate boolean array to track presence.
#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub struct OwnedNullableColumn<S: Scalar> {
    /// The actual values in the column
    pub values: OwnedColumn<S>,
    /// Optional presence vector. `true` means value is present, `false` means NULL
    /// If `None`, all values are present (non-NULL)
    pub presence: Option<Vec<bool>>,
}

impl<S: Scalar> OwnedColumn<S> {
    /// Compute the inner product of the column with a vector of scalars.
    pub(crate) fn inner_product(&self, vec: &[S]) -> S {
        match self {
            OwnedColumn::Boolean(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::Uint8(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::TinyInt(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::SmallInt(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::Int(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => {
                inner_product_ref_cast(col, vec)
            }
            OwnedColumn::VarChar(col) => inner_product_ref_cast(col, vec),
            OwnedColumn::VarBinary(col) => inner_product_with_bytes(col, vec),
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
            OwnedColumn::Uint8(col) => col.len(),
            OwnedColumn::SmallInt(col) => col.len(),
            OwnedColumn::Int(col) => col.len(),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => col.len(),
            OwnedColumn::VarChar(col) => col.len(),
            OwnedColumn::VarBinary(col) => col.len(),
            OwnedColumn::Int128(col) => col.len(),
            OwnedColumn::Decimal75(_, _, col) | OwnedColumn::Scalar(col) => col.len(),
        }
    }

    /// Returns the column with its entries permutated
    pub fn try_permute(&self, permutation: &Permutation) -> Result<Self, PermutationError> {
        Ok(match self {
            OwnedColumn::Boolean(col) => OwnedColumn::Boolean(permutation.try_apply(col)?),
            OwnedColumn::TinyInt(col) => OwnedColumn::TinyInt(permutation.try_apply(col)?),
            OwnedColumn::Uint8(col) => OwnedColumn::Uint8(permutation.try_apply(col)?),
            OwnedColumn::SmallInt(col) => OwnedColumn::SmallInt(permutation.try_apply(col)?),
            OwnedColumn::Int(col) => OwnedColumn::Int(permutation.try_apply(col)?),
            OwnedColumn::BigInt(col) => OwnedColumn::BigInt(permutation.try_apply(col)?),
            OwnedColumn::VarChar(col) => OwnedColumn::VarChar(permutation.try_apply(col)?),
            OwnedColumn::VarBinary(col) => OwnedColumn::VarBinary(permutation.try_apply(col)?),
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
            OwnedColumn::Uint8(col) => OwnedColumn::Uint8(col[start..end].to_vec()),
            OwnedColumn::SmallInt(col) => OwnedColumn::SmallInt(col[start..end].to_vec()),
            OwnedColumn::Int(col) => OwnedColumn::Int(col[start..end].to_vec()),
            OwnedColumn::BigInt(col) => OwnedColumn::BigInt(col[start..end].to_vec()),
            OwnedColumn::VarChar(col) => OwnedColumn::VarChar(col[start..end].to_vec()),
            OwnedColumn::VarBinary(col) => OwnedColumn::VarBinary(col[start..end].to_vec()),
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
            OwnedColumn::Uint8(col) => col.is_empty(),
            OwnedColumn::SmallInt(col) => col.is_empty(),
            OwnedColumn::Int(col) => col.is_empty(),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => col.is_empty(),
            OwnedColumn::VarChar(col) => col.is_empty(),
            OwnedColumn::VarBinary(col) => col.is_empty(),
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
            OwnedColumn::Uint8(_) => ColumnType::Uint8,
            OwnedColumn::SmallInt(_) => ColumnType::SmallInt,
            OwnedColumn::Int(_) => ColumnType::Int,
            OwnedColumn::BigInt(_) => ColumnType::BigInt,
            OwnedColumn::VarChar(_) => ColumnType::VarChar,
            OwnedColumn::VarBinary(_) => ColumnType::VarBinary,
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
            ColumnType::Boolean => {
                let result = scalars
                    .iter()
                    .map(|s| -> Result<bool, _> { TryInto::<bool>::try_into(*s) })
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })?;
                Ok(OwnedColumn::Boolean(result))
            }
            ColumnType::Uint8 => {
                let result: Result<Vec<u8>, _> = scalars
                    .iter()
                    .map(|s| -> Result<u8, _> { TryInto::<u8>::try_into(*s) })
                    .collect();
                result
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })
                    .map(OwnedColumn::Uint8)
            }
            ColumnType::TinyInt => {
                let result: Result<Vec<i8>, _> = scalars
                    .iter()
                    .map(|s| -> Result<i8, _> { TryInto::<i8>::try_into(*s) })
                    .collect();
                result
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })
                    .map(OwnedColumn::TinyInt)
            }
            ColumnType::SmallInt => {
                let result: Result<Vec<i16>, _> = scalars
                    .iter()
                    .map(|s| -> Result<i16, _> { TryInto::<i16>::try_into(*s) })
                    .collect();
                result
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })
                    .map(OwnedColumn::SmallInt)
            }
            ColumnType::Int => {
                let result: Result<Vec<i32>, _> = scalars
                    .iter()
                    .map(|s| -> Result<i32, _> { TryInto::<i32>::try_into(*s) })
                    .collect();
                result
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })
                    .map(OwnedColumn::Int)
            }
            ColumnType::BigInt => {
                let result: Result<Vec<i64>, _> = scalars
                    .iter()
                    .map(|s| -> Result<i64, _> { TryInto::<i64>::try_into(*s) })
                    .collect();
                result
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })
                    .map(OwnedColumn::BigInt)
            }
            ColumnType::Int128 => {
                let result: Result<Vec<i128>, _> = scalars
                    .iter()
                    .map(|s| -> Result<i128, _> { TryInto::<i128>::try_into(*s) })
                    .collect();
                result
                    .map_err(|_| OwnedColumnError::ScalarConversionError {
                        error: "Overflow in scalar conversions".to_string(),
                    })
                    .map(OwnedColumn::Int128)
            }
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
            ColumnType::VarChar | ColumnType::VarBinary => Err(OwnedColumnError::TypeCastError {
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
    /// assuming the underlying type is [u8], panicking if it is not.
    pub fn u8_iter(&self) -> impl Iterator<Item = &u8> {
        match self {
            OwnedColumn::Uint8(col) => col.iter(),
            _ => panic!("Expected Uint8 column"),
        }
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
            Column::Uint8(col) => OwnedColumn::Uint8(col.to_vec()),
            Column::SmallInt(col) => OwnedColumn::SmallInt(col.to_vec()),
            Column::Int(col) => OwnedColumn::Int(col.to_vec()),
            Column::BigInt(col) => OwnedColumn::BigInt(col.to_vec()),
            Column::VarChar((col, _)) => {
                OwnedColumn::VarChar(col.iter().map(ToString::to_string).collect())
            }
            Column::VarBinary((col, _)) => {
                OwnedColumn::VarBinary(col.iter().map(|slice| slice.to_vec()).collect())
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

impl<S: Scalar> OwnedColumn<S> {
    /// Attempts to coerce a column of scalars to a numeric column of the specified type.
    /// If the specified type is the same as the current column type, the function will return the column as is.
    ///
    /// # Arguments
    ///
    /// * `to_type` - The target numeric column type to coerce to.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - If the coercion is successful.
    /// * `Err(ColumnCoercionError)` - If the coercion fails due to type mismatch or overflow.
    ///
    /// # Errors
    ///
    /// If the specified type is the same as the current column type, the function will not error.
    ///
    /// Otherwise, this function will return an error if:
    /// * The column type is not `Scalar`.
    /// * The target type is not a numeric type.
    /// * There is an overflow during the coercion.
    pub(crate) fn try_coerce_scalar_to_numeric(
        self,
        to_type: ColumnType,
    ) -> Result<Self, ColumnCoercionError> {
        if self.column_type() == to_type {
            Ok(self)
        } else if let OwnedColumn::Scalar(vec) = self {
            match to_type {
                ColumnType::Uint8 => {
                    let result: Result<Vec<u8>, _> =
                        vec.into_iter().map(TryInto::try_into).collect();
                    result
                        .map_err(|_| ColumnCoercionError::Overflow)
                        .map(OwnedColumn::Uint8)
                }
                ColumnType::TinyInt => {
                    let result: Result<Vec<i8>, _> =
                        vec.into_iter().map(TryInto::try_into).collect();
                    result
                        .map_err(|_| ColumnCoercionError::Overflow)
                        .map(OwnedColumn::TinyInt)
                }
                ColumnType::SmallInt => {
                    let result: Result<Vec<i16>, _> =
                        vec.into_iter().map(TryInto::try_into).collect();
                    result
                        .map_err(|_| ColumnCoercionError::Overflow)
                        .map(OwnedColumn::SmallInt)
                }
                ColumnType::Int => {
                    let result: Result<Vec<i32>, _> =
                        vec.into_iter().map(TryInto::try_into).collect();
                    result
                        .map_err(|_| ColumnCoercionError::Overflow)
                        .map(OwnedColumn::Int)
                }
                ColumnType::BigInt => {
                    let result: Result<Vec<i64>, _> =
                        vec.into_iter().map(TryInto::try_into).collect();
                    result
                        .map_err(|_| ColumnCoercionError::Overflow)
                        .map(OwnedColumn::BigInt)
                }
                ColumnType::Int128 => {
                    let result: Result<Vec<i128>, _> =
                        vec.into_iter().map(TryInto::try_into).collect();
                    result
                        .map_err(|_| ColumnCoercionError::Overflow)
                        .map(OwnedColumn::Int128)
                }
                ColumnType::Decimal75(precision, scale) => {
                    Ok(OwnedColumn::Decimal75(precision, scale, vec))
                }
                _ => Err(ColumnCoercionError::InvalidTypeCoercion),
            }
        } else {
            Err(ColumnCoercionError::InvalidTypeCoercion)
        }
    }
}

impl<S: Scalar> OwnedNullableColumn<S> {
    /// Creates a new `OwnedNullableColumn` without any NULL values
    /// (all values are present)
    #[must_use]
    pub fn new(values: OwnedColumn<S>) -> Self {
        Self {
            values,
            presence: None,
        }
    }

    /// Creates a new `OwnedNullableColumn` with the given values and presence vector
    ///
    /// Returns an error if the presence vector is `Some` and its length does not match the values length
    pub fn with_presence(
        values: OwnedColumn<S>,
        presence: Option<Vec<bool>>,
    ) -> Result<Self, TableError> {
        if let Some(presence_vec) = &presence {
            if values.len() != presence_vec.len() {
                return Err(TableError::PresenceLengthMismatch);
            }
        }
        Ok(Self { values, presence })
    }

    /// Returns the length of the column
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Returns `true` if the column has no elements
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Returns `true` if the column is nullable
    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.presence.is_some()
    }

    /// Returns the column type
    #[must_use]
    pub fn column_type(&self) -> ColumnType {
        self.values.column_type()
    }

    /// Checks if the value at the given index is NULL
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds
    #[must_use]
    pub fn is_null(&self, index: usize) -> bool {
        assert!(index < self.len(), "Index out of bounds");
        match &self.presence {
            Some(presence) => !presence[index],
            None => false, // When presence is None, no values are NULL
        }
    }

    /// Returns the sliced column
    #[must_use]
    pub fn slice(&self, start: usize, end: usize) -> Self {
        let values = self.values.slice(start, end);
        let presence = self.presence.as_ref().map(|p| p[start..end].to_vec());
        Self { values, presence }
    }

    /// Returns the column with its entries permutated
    pub fn try_permute(&self, permutation: &Permutation) -> Result<Self, PermutationError> {
        let values = self.values.try_permute(permutation)?;
        let presence = match &self.presence {
            Some(p) => Some(permutation.try_apply(p)?),
            None => None,
        };
        Ok(Self { values, presence })
    }

    /// Returns the scalar value at the specified index, or None if the value is NULL
    ///
    /// # Returns
    ///
    /// - `Some(Some(value))` if the value is present at the index
    /// - `Some(None)` if the value is NULL at the index
    /// - `None` if scalar conversion is not possible for the column type
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds
    #[must_use]
    pub fn scalar_at(&self, index: usize) -> Option<Option<S>> {
        assert!(index < self.len(), "Index out of bounds");

        // Check if the value is NULL
        if let Some(presence) = &self.presence {
            if !presence[index] {
                return Some(None); // This position contains a NULL
            }
        }

        // Get the non-NULL value
        match &self.values {
            OwnedColumn::Boolean(col) => Some(Some(col[index].into())),
            OwnedColumn::Uint8(col) => Some(Some(col[index].into())),
            OwnedColumn::TinyInt(col) => Some(Some(col[index].into())),
            OwnedColumn::SmallInt(col) => Some(Some(col[index].into())),
            OwnedColumn::Int(col) => Some(Some(col[index].into())),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => {
                Some(Some(col[index].into()))
            }
            OwnedColumn::Int128(col) => Some(Some(col[index].into())),
            OwnedColumn::Scalar(col) | OwnedColumn::Decimal75(_, _, col) => Some(Some(col[index])),
            // VarChar and VarBinary would need hash values
            _ => None,
        }
    }

    /// Returns the scalar value at the specified index if it exists, or None if the index is out of bounds or the value is NULL
    ///
    /// This is a non-panicking alternative to `scalar_at`.
    ///
    /// # Returns
    ///
    /// - `Some(Some(value))` if the value is present at the index
    /// - `Some(None)` if the value is NULL at the index
    /// - `None` if the index is out of bounds or scalar conversion is not possible for the column type
    #[must_use]
    pub fn get_scalar_at(&self, index: usize) -> Option<Option<S>> {
        // Check if index is out of bounds
        if index >= self.len() {
            return None;
        }

        // Check if the value is NULL
        if let Some(presence) = &self.presence {
            if !presence[index] {
                return Some(None); // This position contains a NULL
            }
        }

        // Get the non-NULL value
        match &self.values {
            OwnedColumn::Boolean(col) => Some(Some(col[index].into())),
            OwnedColumn::Uint8(col) => Some(Some(col[index].into())),
            OwnedColumn::TinyInt(col) => Some(Some(col[index].into())),
            OwnedColumn::SmallInt(col) => Some(Some(col[index].into())),
            OwnedColumn::Int(col) => Some(Some(col[index].into())),
            OwnedColumn::BigInt(col) | OwnedColumn::TimestampTZ(_, _, col) => {
                Some(Some(col[index].into()))
            }
            OwnedColumn::Int128(col) => Some(Some(col[index].into())),
            OwnedColumn::Scalar(col) | OwnedColumn::Decimal75(_, _, col) => Some(Some(col[index])),
            // VarChar and VarBinary would need hash values
            _ => None,
        }
    }

    /// Creates a nullable column from a vector of optional scalars
    ///
    /// This is a convenience method to create a nullable column from a vector where
    /// Some values are treated as present and None values are treated as NULL.
    ///
    /// # Errors
    ///
    /// Returns an error if the column type cannot be created from the provided scalars
    pub fn try_from_option_scalars(
        option_scalars: &[Option<S>],
        column_type: ColumnType,
    ) -> OwnedColumnResult<Self> {
        if option_scalars.is_empty() {
            // Handle empty case
            let empty_column = OwnedColumn::try_from_scalars(&[], column_type)?;
            return Ok(Self::new(empty_column));
        }

        // Extract non-null values and build presence vector
        let mut values = Vec::with_capacity(option_scalars.len());
        let mut presence = Vec::with_capacity(option_scalars.len());
        let mut has_nulls = false;

        for opt_scalar in option_scalars {
            if let Some(val) = opt_scalar {
                values.push(*val);
                presence.push(true);
            } else {
                // Push a default value for NULL entries
                // This value won't be used since the presence vector marks it as NULL
                values.push(S::default());
                presence.push(false);
                has_nulls = true;
            }
        }

        // Create the OwnedColumn from the extracted values
        let owned_column = OwnedColumn::try_from_scalars(&values, column_type)?;

        // Only use the presence vector if there are actual nulls
        let presence = if has_nulls { Some(presence) } else { None };

        // Convert the TableError to OwnedColumnError (now possible due to our From implementation)
        match Self::with_presence(owned_column, presence) {
            Ok(result) => Ok(result),
            Err(err) => Err(OwnedColumnError::from(err)),
        }
    }

    /// Collects an iterator of optional scalars into a nullable column
    ///
    /// This is similar to `try_from_option_scalars` but works with iterators.
    ///
    /// ## `ExactSizeIterator` Requirement
    ///
    /// This method requires an `ExactSizeIterator` to enable efficient pre-allocation of
    /// the internal vectors. Knowing the exact size upfront allows us to allocate the right
    /// amount of memory for both the values and presence vectors, avoiding costly reallocations.
    ///
    /// If you have an iterator that doesn't implement `ExactSizeIterator`, consider collecting
    /// it into a `Vec` first and then using `try_from_option_scalars` instead.
    ///
    /// ## Data Consistency
    ///
    /// This method maintains the same consistency guarantees as other operations in `OwnedNullableColumn`,
    /// ensuring that both the values and presence vectors are updated synchronously.
    ///
    /// # Errors
    ///
    /// Returns an error if the column type cannot be created from the provided scalars
    pub fn try_collect_option_scalars<I>(
        iter: I,
        column_type: ColumnType,
    ) -> OwnedColumnResult<Self>
    where
        I: Iterator<Item = Option<S>> + ExactSizeIterator,
    {
        let len = iter.len();
        if len == 0 {
            // Handle empty case
            let empty_column = OwnedColumn::try_from_scalars(&[], column_type)?;
            return Ok(Self::new(empty_column));
        }

        // Extract non-null values and build presence vector
        let mut values = Vec::with_capacity(len);
        let mut presence = Vec::with_capacity(len);
        let mut has_nulls = false;

        for opt_scalar in iter {
            if let Some(val) = opt_scalar {
                values.push(val);
                presence.push(true);
            } else {
                // Push a default value for NULL entries
                values.push(S::default());
                presence.push(false);
                has_nulls = true;
            }
        }

        // Create the OwnedColumn from the extracted values
        let owned_column = OwnedColumn::try_from_scalars(&values, column_type)?;

        // Only use the presence vector if there are actual nulls
        let presence = if has_nulls { Some(presence) } else { None };

        // Convert the TableError to OwnedColumnError (now possible due to our From implementation)
        match Self::with_presence(owned_column, presence) {
            Ok(result) => Ok(result),
            Err(err) => Err(OwnedColumnError::from(err)),
        }
    }
}

impl<'a, S: Scalar> From<&NullableColumn<'a, S>> for OwnedNullableColumn<S> {
    fn from(nullable_column: &NullableColumn<'a, S>) -> Self {
        let values = OwnedColumn::from(&nullable_column.values);
        let presence = nullable_column.presence.map(<[bool]>::to_vec);
        // This should never fail since we're directly converting from a valid NullableColumn
        Self::with_presence(values.clone(), presence).unwrap_or_else(|_| {
            // Fallback to creating a non-nullable column if there's any issue
            Self::new(values)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::{
        math::decimal::Precision,
        scalar::{test_scalar::TestScalar, ScalarExt},
    };
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
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), -128, scalars.clone())
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

    #[test]
    fn we_can_coerce_scalar_to_numeric() {
        let scalars = vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];
        let col = OwnedColumn::Scalar(scalars.clone());

        // Coerce to TinyInt
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::TinyInt)
            .unwrap();
        assert_eq!(coerced_col, OwnedColumn::TinyInt(vec![1, 2, 3]));

        // Coerce to SmallInt
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::SmallInt)
            .unwrap();
        assert_eq!(coerced_col, OwnedColumn::SmallInt(vec![1, 2, 3]));

        // Coerce to Int
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::Int)
            .unwrap();
        assert_eq!(coerced_col, OwnedColumn::Int(vec![1, 2, 3]));

        // Coerce to BigInt
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::BigInt)
            .unwrap();
        assert_eq!(coerced_col, OwnedColumn::BigInt(vec![1, 2, 3]));

        // Coerce to Int128
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::Int128)
            .unwrap();
        assert_eq!(coerced_col, OwnedColumn::Int128(vec![1, 2, 3]));

        // Coerce to Decimal75
        let coerced_col = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::Decimal75(Precision::new(75).unwrap(), 0))
            .unwrap();
        assert_eq!(
            coerced_col,
            OwnedColumn::Decimal75(Precision::new(75).unwrap(), 0, scalars)
        );
    }

    #[test]
    fn we_cannot_coerce_scalar_to_invalid_type() {
        let scalars = vec![
            TestScalar::from(1),
            TestScalar::from(2),
            TestScalar::from(3),
        ];
        let col = OwnedColumn::Scalar(scalars);

        // Attempt to coerce to VarChar
        let res = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::VarChar);
        assert!(matches!(res, Err(ColumnCoercionError::InvalidTypeCoercion)));

        // Attempt to coerce non-scalar column
        let col = OwnedColumn::<TestScalar>::Int(vec![1, 2, 3]);
        let res = col.try_coerce_scalar_to_numeric(ColumnType::BigInt);
        assert!(matches!(res, Err(ColumnCoercionError::InvalidTypeCoercion)));
    }

    #[test]
    fn we_cannot_coerce_scalar_to_numeric_if_overflow() {
        let scalars = vec![TestScalar::from(i128::MAX), -TestScalar::from(i128::MIN)];
        let col = OwnedColumn::Scalar(scalars);

        // Attempt to coerce to TinyInt
        let res = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::TinyInt);
        assert!(matches!(res, Err(ColumnCoercionError::Overflow)));

        // Attempt to coerce to SmallInt
        let res = col
            .clone()
            .try_coerce_scalar_to_numeric(ColumnType::SmallInt);
        assert!(matches!(res, Err(ColumnCoercionError::Overflow)));

        // Attempt to coerce to Int
        let res = col.clone().try_coerce_scalar_to_numeric(ColumnType::Int);
        assert!(matches!(res, Err(ColumnCoercionError::Overflow)));

        // Attempt to coerce to BigInt
        let res = col.clone().try_coerce_scalar_to_numeric(ColumnType::BigInt);
        assert!(matches!(res, Err(ColumnCoercionError::Overflow)));

        // Attempt to coerce to Int128
        let res = col.try_coerce_scalar_to_numeric(ColumnType::Int128);
        assert!(matches!(res, Err(ColumnCoercionError::Overflow)));
    }

    #[test]
    fn we_can_slice_and_permute_varbinary_columns() {
        let col = OwnedColumn::<TestScalar>::VarBinary(vec![
            b"foo".to_vec(),
            b"bar".to_vec(),
            b"baz".to_vec(),
            b"qux".to_vec(),
        ]);
        assert_eq!(
            col.slice(1, 3),
            OwnedColumn::VarBinary(vec![b"bar".to_vec(), b"baz".to_vec()])
        );
        let permutation = Permutation::try_new(vec![2, 0, 3, 1]).unwrap();
        assert_eq!(
            col.try_permute(&permutation).unwrap(),
            OwnedColumn::VarBinary(vec![
                b"baz".to_vec(),
                b"foo".to_vec(),
                b"qux".to_vec(),
                b"bar".to_vec()
            ])
        );
    }

    #[test]
    fn we_can_convert_varbinary_column_round_trip_using_hash() {
        let raw_bytes = [b"abc".as_ref(), b"xyz".as_ref()];

        let scalars: Vec<TestScalar> = raw_bytes
            .iter()
            .map(|&b| TestScalar::from_byte_slice_via_hash(b))
            .collect();

        let col: Column<'_, TestScalar> =
            Column::VarBinary((raw_bytes.as_slice(), scalars.as_slice()));

        let owned_col: OwnedColumn<TestScalar> = (&col).into();

        assert_eq!(
            owned_col,
            OwnedColumn::VarBinary(vec![b"abc".to_vec(), b"xyz".to_vec()])
        );

        let bump = bumpalo::Bump::new();
        let new_col = Column::<TestScalar>::from_owned_column(&owned_col, &bump);

        assert_eq!(col, new_col);
    }

    #[test]
    fn we_can_compute_inner_product_with_varbinary_columns_using_hash() {
        let lhs = OwnedColumn::<TestScalar>::VarBinary(vec![
            b"foo".to_vec(),
            b"bar".to_vec(),
            b"baz".to_vec(),
        ]);

        let scalars = vec![
            TestScalar::from(10),
            TestScalar::from(20),
            TestScalar::from(30),
        ];

        let product = lhs.inner_product(&scalars);

        let lhs_hashes: Vec<TestScalar> = [b"foo".as_ref(), b"bar".as_ref(), b"baz".as_ref()]
            .iter()
            .map(|&bytes| TestScalar::from_byte_slice_via_hash(bytes))
            .collect();

        let expected =
            lhs_hashes[0] * scalars[0] + lhs_hashes[1] * scalars[1] + lhs_hashes[2] * scalars[2];

        assert_eq!(product, expected);
    }

    #[test]
    fn we_can_create_owned_nullable_column() {
        let bool_values = vec![true, false, true];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Boolean(bool_values);

        let nullable_column = OwnedNullableColumn::new(owned_column.clone());
        assert_eq!(nullable_column.len(), 3);
        assert!(!nullable_column.is_empty());
        assert!(!nullable_column.is_nullable());

        for i in 0..3 {
            assert!(!nullable_column.is_null(i));
        }

        let presence = Some(vec![true, true, true]);
        let nullable_column =
            OwnedNullableColumn::with_presence(owned_column.clone(), presence).unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());

        for i in 0..3 {
            assert!(!nullable_column.is_null(i));
        }

        let presence = Some(vec![true, false, true]);
        let nullable_column = OwnedNullableColumn::with_presence(owned_column, presence).unwrap();
        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());

        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));
    }

    #[test]
    fn owned_nullable_column_returns_error_if_presence_length_mismatch() {
        let bool_values = vec![true, false, true];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Boolean(bool_values);

        let presence = Some(vec![true, false]);
        let result = OwnedNullableColumn::with_presence(owned_column, presence);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TableError::PresenceLengthMismatch
        ));
    }

    #[test]
    fn we_can_slice_owned_nullable_column() {
        let bool_values = vec![true, false, true, false, true];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Boolean(bool_values);
        let presence = Some(vec![true, false, true, true, false]);
        let nullable_column = OwnedNullableColumn::with_presence(owned_column, presence).unwrap();

        let sliced = nullable_column.slice(1, 4);

        assert_eq!(sliced.len(), 3);
        assert!(sliced.is_nullable());

        assert!(sliced.is_null(0));
        assert!(!sliced.is_null(1));
        assert!(!sliced.is_null(2));
    }

    #[test]
    fn we_can_permute_owned_nullable_column() {
        let bool_values = vec![true, false, true];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Boolean(bool_values);
        let presence = Some(vec![true, false, true]);
        let nullable_column = OwnedNullableColumn::with_presence(owned_column, presence).unwrap();

        let permutation = Permutation::try_new(vec![2, 0, 1]).unwrap();

        let permuted = nullable_column.try_permute(&permutation).unwrap();

        assert_eq!(permuted.len(), 3);
        assert!(permuted.is_nullable());

        assert!(!permuted.is_null(0));
        assert!(!permuted.is_null(1));
        assert!(permuted.is_null(2));
    }

    #[test]
    fn we_can_convert_nullable_column_to_owned_nullable_column() {
        let alloc = Bump::new();

        let bool_values = [true, false, true];
        let column: Column<'_, TestScalar> = Column::Boolean(&bool_values);

        let presence = Some(alloc.alloc_slice_copy(&[true, false, true]));
        let nullable_column = NullableColumn::with_presence(column, presence.as_deref()).unwrap();

        let owned_nullable_column = OwnedNullableColumn::from(&nullable_column);

        assert_eq!(owned_nullable_column.len(), 3);
        assert!(owned_nullable_column.is_nullable());
        assert!(!owned_nullable_column.is_null(0));
        assert!(owned_nullable_column.is_null(1));
        assert!(!owned_nullable_column.is_null(2));
    }

    #[test]
    fn we_can_get_scalar_from_owned_nullable_column() {
        let int_values = vec![10, 20, 30];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Int(int_values);

        let nullable_column = OwnedNullableColumn::new(owned_column.clone());
        assert_eq!(nullable_column.scalar_at(0), Some(Some(10i32.into())));
        assert_eq!(nullable_column.scalar_at(1), Some(Some(20i32.into())));
        assert_eq!(nullable_column.scalar_at(2), Some(Some(30i32.into())));

        let presence = Some(vec![true, true, true]);
        let nullable_column =
            OwnedNullableColumn::with_presence(owned_column.clone(), presence).unwrap();
        assert_eq!(nullable_column.scalar_at(0), Some(Some(10i32.into())));
        assert_eq!(nullable_column.scalar_at(1), Some(Some(20i32.into())));
        assert_eq!(nullable_column.scalar_at(2), Some(Some(30i32.into())));

        let presence = Some(vec![true, false, true]);
        let nullable_column = OwnedNullableColumn::with_presence(owned_column, presence).unwrap();
        assert_eq!(nullable_column.scalar_at(0), Some(Some(10i32.into())));
        assert_eq!(nullable_column.scalar_at(1), Some(None));
        assert_eq!(nullable_column.scalar_at(2), Some(Some(30i32.into())));
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn owned_nullable_column_scalar_at_panics_if_index_out_of_bounds() {
        let int_values = vec![10, 20, 30];
        let owned_column: OwnedColumn<TestScalar> = OwnedColumn::Int(int_values);
        let nullable_column = OwnedNullableColumn::new(owned_column);

        let _ = nullable_column.scalar_at(3);
    }

    #[test]
    fn we_can_create_owned_nullable_column_from_option_scalars() {
        let option_values: Vec<Option<TestScalar>> =
            vec![Some(10i32.into()), None, Some(30i32.into())];

        let nullable_column =
            OwnedNullableColumn::try_from_option_scalars(&option_values, ColumnType::Int).unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());

        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        assert_eq!(nullable_column.scalar_at(0), Some(Some(10i32.into())));
        assert_eq!(nullable_column.scalar_at(1), Some(None));
        assert_eq!(nullable_column.scalar_at(2), Some(Some(30i32.into())));
    }

    #[test]
    fn we_can_create_owned_nullable_column_from_option_scalars_no_nulls() {
        let option_values: Vec<Option<TestScalar>> =
            vec![Some(10i32.into()), Some(20i32.into()), Some(30i32.into())];

        let nullable_column =
            OwnedNullableColumn::try_from_option_scalars(&option_values, ColumnType::Int).unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(!nullable_column.is_nullable());

        assert!(!nullable_column.is_null(0));
        assert!(!nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        assert_eq!(nullable_column.scalar_at(0), Some(Some(10i32.into())));
        assert_eq!(nullable_column.scalar_at(1), Some(Some(20i32.into())));
        assert_eq!(nullable_column.scalar_at(2), Some(Some(30i32.into())));
    }

    #[test]
    fn we_can_create_owned_nullable_column_from_option_scalars_all_nulls() {
        let option_values: Vec<Option<TestScalar>> = vec![None, None, None];

        let nullable_column =
            OwnedNullableColumn::try_from_option_scalars(&option_values, ColumnType::Int).unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());

        assert!(nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(nullable_column.is_null(2));

        assert_eq!(nullable_column.scalar_at(0), Some(None));
        assert_eq!(nullable_column.scalar_at(1), Some(None));
        assert_eq!(nullable_column.scalar_at(2), Some(None));
    }

    #[test]
    fn we_can_create_owned_nullable_column_from_option_scalars_empty() {
        let option_values: Vec<Option<TestScalar>> = vec![];

        let nullable_column =
            OwnedNullableColumn::try_from_option_scalars(&option_values, ColumnType::Int).unwrap();

        assert_eq!(nullable_column.len(), 0);
        assert!(!nullable_column.is_nullable());
        assert!(nullable_column.is_empty());
    }

    #[test]
    fn we_can_collect_option_scalars_into_owned_nullable_column() {
        let option_values: Vec<Option<TestScalar>> =
            vec![Some(10i32.into()), None, Some(30i32.into())];

        let nullable_column = OwnedNullableColumn::try_collect_option_scalars(
            option_values.into_iter(),
            ColumnType::Int,
        )
        .unwrap();

        assert_eq!(nullable_column.len(), 3);
        assert!(nullable_column.is_nullable());

        assert!(!nullable_column.is_null(0));
        assert!(nullable_column.is_null(1));
        assert!(!nullable_column.is_null(2));

        assert_eq!(nullable_column.scalar_at(0), Some(Some(10i32.into())));
        assert_eq!(nullable_column.scalar_at(1), Some(None));
        assert_eq!(nullable_column.scalar_at(2), Some(Some(30i32.into())));
    }
}
