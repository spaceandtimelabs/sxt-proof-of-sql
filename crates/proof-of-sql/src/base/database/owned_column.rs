/// A column of data, with type included. This is simply a wrapper around `Vec<T>` for enumerated `T`.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow Array.
use super::ColumnType;
use crate::base::{
    math::decimal::Precision,
    scalar::Scalar,
    time::timestamp::{ProofsTimeUnit, ProofsTimeZone},
};
#[derive(Debug, PartialEq, Clone, Eq)]
#[non_exhaustive]
/// Supported types for OwnedColumn
pub enum OwnedColumn<S: Scalar> {
    /// Boolean columns
    Boolean(Vec<bool>),
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
    Timestamp(ProofsTimeUnit, ProofsTimeZone, Vec<i64>),
}

impl<S: Scalar> OwnedColumn<S> {
    /// Returns the length of the column.
    pub fn len(&self) -> usize {
        match self {
            OwnedColumn::Boolean(col) => col.len(),
            OwnedColumn::SmallInt(col) => col.len(),
            OwnedColumn::Int(col) => col.len(),
            OwnedColumn::BigInt(col) => col.len(),
            OwnedColumn::VarChar(col) => col.len(),
            OwnedColumn::Int128(col) => col.len(),
            OwnedColumn::Decimal75(_, _, col) => col.len(),
            OwnedColumn::Scalar(col) => col.len(),
            OwnedColumn::Timestamp(_, _, col) => col.len(),
        }
    }
    /// Returns true if the column is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            OwnedColumn::Boolean(col) => col.is_empty(),
            OwnedColumn::SmallInt(col) => col.is_empty(),
            OwnedColumn::Int(col) => col.is_empty(),
            OwnedColumn::BigInt(col) => col.is_empty(),
            OwnedColumn::VarChar(col) => col.is_empty(),
            OwnedColumn::Int128(col) => col.is_empty(),
            OwnedColumn::Scalar(col) => col.is_empty(),
            OwnedColumn::Decimal75(_, _, col) => col.is_empty(),
            OwnedColumn::Timestamp(_, _, col) => col.is_empty(),
        }
    }
    /// Returns the type of the column.
    pub fn column_type(&self) -> ColumnType {
        match self {
            OwnedColumn::Boolean(_) => ColumnType::Boolean,
            OwnedColumn::SmallInt(_) => ColumnType::SmallInt,
            OwnedColumn::Int(_) => ColumnType::Int,
            OwnedColumn::BigInt(_) => ColumnType::BigInt,
            OwnedColumn::VarChar(_) => ColumnType::VarChar,
            OwnedColumn::Int128(_) => ColumnType::Int128,
            OwnedColumn::Scalar(_) => ColumnType::Scalar,
            OwnedColumn::Decimal75(precision, scale, _) => {
                ColumnType::Decimal75(*precision, *scale)
            }
            OwnedColumn::Timestamp(tu, tz, _) => ColumnType::Timestamp(*tu, *tz),
        }
    }
}

impl<S: Scalar> FromIterator<bool> for OwnedColumn<S> {
    fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
        Self::Boolean(Vec::from_iter(iter))
    }
}
impl<S: Scalar> FromIterator<i16> for OwnedColumn<S> {
    fn from_iter<T: IntoIterator<Item = i16>>(iter: T) -> Self {
        Self::SmallInt(Vec::from_iter(iter))
    }
}
impl<S: Scalar> FromIterator<i32> for OwnedColumn<S> {
    fn from_iter<T: IntoIterator<Item = i32>>(iter: T) -> Self {
        Self::Int(Vec::from_iter(iter))
    }
}
impl<S: Scalar> FromIterator<i64> for OwnedColumn<S> {
    fn from_iter<T: IntoIterator<Item = i64>>(iter: T) -> Self {
        Self::BigInt(Vec::from_iter(iter))
    }
}
impl<S: Scalar> FromIterator<i128> for OwnedColumn<S> {
    fn from_iter<T: IntoIterator<Item = i128>>(iter: T) -> Self {
        Self::Int128(Vec::from_iter(iter))
    }
}
impl<S: Scalar> FromIterator<String> for OwnedColumn<S> {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self::VarChar(Vec::from_iter(iter))
    }
}
impl<S: Scalar> FromIterator<S> for OwnedColumn<S> {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        Self::Scalar(Vec::from_iter(iter))
    }
}
impl<'a, S: Scalar> FromIterator<&'a str> for OwnedColumn<S> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter().map(|s| s.to_string()))
    }
}
