/// Definitions of Column, GeneralColumn, Table and conversions
/// from Arrow Arrays, RecordBatches as well as Datafusion ColumnarValues into them.
use crate::base::{
    proof::{Commit, Commitment, ProofError, ProofResult},
    scalar::{IntoScalar, SafeIntColumn},
};
use curve25519_dalek::scalar::Scalar;
use derive_more::{Deref, DerefMut, TryInto};
use std::ops::{Add, Mul, Neg, Sub};

/// Definition of Column, GeneralColumn and Table

/// New-type representing a database column.
#[derive(Clone, Default, Debug, Eq, PartialEq, Deref, DerefMut)]
pub struct Column<T> {
    pub data: Vec<T>,
}

impl<T> Column<T>
where
    T: IntoScalar + Clone,
{
    pub fn into_scalar_column(self) -> Column<Scalar> {
        Column::from(
            self.iter()
                .map(|d| d.clone().into_scalar())
                .collect::<Vec<Scalar>>(),
        )
    }
}

impl<T> Commit for Column<T>
where
    T: IntoScalar + Clone,
{
    type Commitment = Commitment;

    fn commit(&self) -> Self::Commitment {
        Commitment::from(
            self.iter()
                .map(|d| d.clone().into_scalar())
                .collect::<Vec<Scalar>>()
                .as_slice(),
        )
    }
}

impl<T> From<Vec<T>> for Column<T> {
    fn from(data: Vec<T>) -> Self {
        Column { data }
    }
}

impl<T> IntoIterator for Column<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

// Implement Add for Column<Scalar>
impl Add for &Column<Scalar> {
    type Output = Column<Scalar>;

    fn add(self, other: Self) -> Self::Output {
        assert_eq!(self.len(), other.len());
        let sum: Vec<_> = self.iter().zip(other.iter()).map(|(a, b)| a + b).collect();
        Column::from(sum)
    }
}

// Implement Mul for Column<Scalar>
impl Mul for &Column<Scalar> {
    type Output = Column<Scalar>;

    fn mul(self, other: Self) -> Self::Output {
        assert_eq!(self.len(), other.len());
        let product: Vec<_> = self.iter().zip(other.iter()).map(|(a, b)| a * b).collect();
        Column::from(product)
    }
}

// Implement Sub for Column<Scalar>
impl Sub for &Column<Scalar> {
    type Output = Column<Scalar>;

    fn sub(self, other: Self) -> Self::Output {
        assert_eq!(self.len(), other.len());
        let difference: Vec<_> = self.iter().zip(other.iter()).map(|(a, b)| a - b).collect();
        Column::from(difference)
    }
}

// Implement Neg for Column<Scalar>
impl Neg for &Column<Scalar> {
    type Output = Column<Scalar>;

    fn neg(self) -> Self::Output {
        let negated: Vec<_> = self.iter().map(|a| -a).collect();
        Column::from(negated)
    }
}

impl<X> FromIterator<X> for Column<X> {
    fn from_iter<I: IntoIterator<Item = X>>(iter: I) -> Self {
        Column {
            data: iter.into_iter().collect(),
        }
    }
}

// Enum of columns of all the supported types
#[derive(Clone, Debug, Eq, PartialEq, TryInto)]
#[try_into(owned, ref, ref_mut)]
pub enum GeneralColumn {
    BooleanColumn(Column<bool>),
    SafeIntColumn(SafeIntColumn),
}

impl GeneralColumn {
    pub fn len(&self) -> usize {
        match self {
            GeneralColumn::BooleanColumn(c) => c.data.len(),
            GeneralColumn::SafeIntColumn(c) => c.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Commit for GeneralColumn {
    type Commitment = Commitment;

    fn commit(&self) -> Self::Commitment {
        match self {
            GeneralColumn::BooleanColumn(c) => c.commit(),
            GeneralColumn::SafeIntColumn(c) => c.commit(),
        }
    }
}

impl From<GeneralColumn> for Column<Scalar> {
    fn from(general_column: GeneralColumn) -> Self {
        match general_column {
            GeneralColumn::BooleanColumn(col) => col
                .iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
            GeneralColumn::SafeIntColumn(col) => col
                .into_iter()
                .map(|ci| ci.into_scalar())
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

/// The proof version of RecordBatch
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Table {
    pub data: Vec<GeneralColumn>,
    /// Num of rows in any GeneralColumn in data
    pub num_rows: usize,
}

impl Table {
    pub fn try_new(data: Vec<GeneralColumn>, num_rows: usize) -> ProofResult<Self> {
        for col in data.iter() {
            if col.len() != num_rows {
                return Err(ProofError::TableColumnLengthError);
            }
        }
        Ok(Table { data, num_rows })
    }
}

impl Commit for Table {
    type Commitment = Vec<Commitment>;

    fn commit(&self) -> Self::Commitment {
        self.data.iter().map(|c| c.commit()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generalcolumn_length() {
        let general_column =
            GeneralColumn::SafeIntColumn(SafeIntColumn::from(vec![1i16, 2i16, 3i16]));
        assert_eq!(general_column.len(), 3);
    }

    #[test]
    fn test_generalcolumn_length_empty() {
        let general_column = GeneralColumn::SafeIntColumn(SafeIntColumn::from(Vec::<i8>::new()));
        assert_eq!(general_column.len(), 0);
    }

    #[test]
    fn test_generalcolumn_is_empty_true() {
        let general_column = GeneralColumn::SafeIntColumn(SafeIntColumn::from(Vec::<i64>::new()));
        assert_eq!(general_column.is_empty(), true);
    }

    #[test]
    fn test_generalcolumn_is_empty_false() {
        let general_column =
            GeneralColumn::SafeIntColumn(SafeIntColumn::from(vec![-1i16, -2i16, -3i16]));
        assert_eq!(general_column.is_empty(), false);
    }

    #[test]
    fn test_table_try_new() {
        let general_column0 =
            GeneralColumn::SafeIntColumn(SafeIntColumn::from(vec![-1i16, -2i16, -3i16]));
        let general_column1 = GeneralColumn::SafeIntColumn(SafeIntColumn::from(vec![1, 2, 3]));
        let general_columns = vec![general_column0, general_column1];
        let actual = Table::try_new(general_columns, 3).unwrap();
        let expected = Table {
            data: vec![
                GeneralColumn::SafeIntColumn(SafeIntColumn::from(vec![-1i16, -2i16, -3i16])),
                GeneralColumn::SafeIntColumn(SafeIntColumn::from(vec![1, 2, 3])),
            ],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_empty_table_try_new() {
        let general_columns: Vec<GeneralColumn> = vec![];
        let actual = Table::try_new(general_columns, 3).unwrap();
        let expected = Table {
            data: vec![],
            num_rows: 3,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    #[should_panic]
    fn test_table_try_new_failed_incompatible_lengths() {
        let general_column0 = GeneralColumn::SafeIntColumn(SafeIntColumn::from(vec![-1i16, -2i16]));
        let general_column1 = GeneralColumn::SafeIntColumn(SafeIntColumn::from(vec![1, 2, 3]));
        let general_columns = vec![general_column0, general_column1];
        Table::try_new(general_columns, 3).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_table_try_new_failed_wrong_num_rows() {
        let general_column0 =
            GeneralColumn::SafeIntColumn(SafeIntColumn::from(vec![-1i16, -2i16, 3i16]));
        let general_column1 = GeneralColumn::SafeIntColumn(SafeIntColumn::from(vec![1, 2, 3]));
        let general_columns = vec![general_column0, general_column1];
        Table::try_new(general_columns, 2).unwrap();
    }
}
