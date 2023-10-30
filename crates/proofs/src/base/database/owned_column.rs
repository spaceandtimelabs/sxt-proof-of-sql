/// A column of data, with type included. This is simply a wrapper around `Vec<T>` for enumerated `T`.
/// This is primarily used as an internal result that is used before
/// converting to the final result in either Arrow format or JSON.
/// This is the analog of an arrow Array.
#[derive(Debug, PartialEq, Clone, Eq)]
pub enum OwnedColumn {
    /// i64 columns
    BigInt(Vec<i64>),
    /// String columns
    VarChar(Vec<String>),
    /// i128 columns
    Int128(Vec<i128>),
}

impl OwnedColumn {
    /// Returns the length of the column.
    pub fn len(&self) -> usize {
        match self {
            OwnedColumn::BigInt(col) => col.len(),
            OwnedColumn::VarChar(col) => col.len(),
            OwnedColumn::Int128(col) => col.len(),
        }
    }
    /// Returns true if the column is empty.
    pub fn is_empty(&self) -> bool {
        match self {
            OwnedColumn::BigInt(col) => col.is_empty(),
            OwnedColumn::VarChar(col) => col.is_empty(),
            OwnedColumn::Int128(col) => col.is_empty(),
        }
    }
}

impl FromIterator<i64> for OwnedColumn {
    fn from_iter<T: IntoIterator<Item = i64>>(iter: T) -> Self {
        Self::BigInt(Vec::from_iter(iter))
    }
}
impl FromIterator<i128> for OwnedColumn {
    fn from_iter<T: IntoIterator<Item = i128>>(iter: T) -> Self {
        Self::Int128(Vec::from_iter(iter))
    }
}
impl FromIterator<String> for OwnedColumn {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self::VarChar(Vec::from_iter(iter))
    }
}
impl<'a> FromIterator<&'a str> for OwnedColumn {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter().map(|s| s.to_string()))
    }
}
