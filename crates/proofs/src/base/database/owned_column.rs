/// A column of data, with type included. This is simply a wrapper around `Vec<T>` for enumerated `T`.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow Array.
use super::ColumnType;
use crate::base::{math::decimal::Precision, scalar::Scalar};
#[derive(Debug, PartialEq, Clone, Eq)]
#[non_exhaustive]
/// Supported types for OwnedColumn
pub enum OwnedColumn<S: Scalar> {
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
}

impl<S: Scalar> OwnedColumn<S> {
    /// Returns the length of the column.
    pub fn len(&self) -> usize {
        match self {
            OwnedColumn::BigInt(col) => col.len(),
            OwnedColumn::VarChar(col) => col.len(),
            OwnedColumn::Int128(col) => col.len(),
            OwnedColumn::Decimal75(_, _, col) => col.len(),
            OwnedColumn::Scalar(col) => col.len(),
        }
    }
    /// Returns true if the column is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            OwnedColumn::BigInt(col) => col.is_empty(),
            OwnedColumn::VarChar(col) => col.is_empty(),
            OwnedColumn::Int128(col) => col.is_empty(),
            OwnedColumn::Scalar(col) => col.is_empty(),
            OwnedColumn::Decimal75(_, _, col) => col.is_empty(),
        }
    }
    /// Returns the type of the column.
    pub fn column_type(&self) -> ColumnType {
        match self {
            OwnedColumn::BigInt(_) => ColumnType::BigInt,
            OwnedColumn::VarChar(_) => ColumnType::VarChar,
            OwnedColumn::Int128(_) => ColumnType::Int128,
            OwnedColumn::Scalar(_) => ColumnType::Scalar,
            OwnedColumn::Decimal75(precision, scale, _) => {
                ColumnType::Decimal75(*precision, *scale)
            }
        }
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
