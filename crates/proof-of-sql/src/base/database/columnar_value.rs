use crate::base::{
    database::{Column, ColumnType, LiteralValue},
    scalar::Scalar,
};
use bumpalo::Bump;

/// The result of evaluating an expression.
///
/// Inspired by [`datafusion_expr_common::ColumnarValue`]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ColumnarValue<'a, S: Scalar> {
    /// A [ `ColumnarValue::Column` ] is a list of values.
    Column(Column<'a, S>),
    /// A [ `ColumnarValue::Literal` ] is a single value with indeterminate size.
    Literal(LiteralValue<S>),
}

impl<'a, S: Scalar> ColumnarValue<'a, S> {
    /// Provides the column type associated with the column
    pub fn column_type(&self) -> ColumnType {
        match self {
            Self::Column(column) => column.column_type(),
            Self::Literal(literal) => literal.column_type(),
        }
    }

    /// Converts the [`ColumnarValue`] to a [`Column`]
    pub fn into_column(&self, num_rows: usize, alloc: &'a Bump) -> Column<'a, S> {
        match self {
            Self::Column(column) => {
                assert_eq!(column.len(), num_rows);
                *column
            }
            Self::Literal(literal) => Column::from_literal_with_length(literal, num_rows, alloc),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_get_column_type_of_columnar_values() {
        let column = ColumnarValue::Column(Column::Int(&[1, 2, 3]));
        assert_eq!(column.column_type(), ColumnType::Int);

        let column = ColumnarValue::Literal(LiteralValue::Boolean(true));
        assert_eq!(column.column_type(), ColumnType::Boolean);
    }

    #[test]
    fn we_can_transform_columnar_values_into_columns() {
        let bump = Bump::new();

        let columnar_value = ColumnarValue::Column(Column::Int(&[1, 2, 3]));
        let column = column.into_column(3, &bump);
        assert_eq!(column, Column::Int(&[1, 2, 3]));

        let columnar_value = ColumnarValue::Literal(LiteralValue::Boolean(false));
        let column = column.into_column(5, &bump);
        assert_eq!(column, Column::Boolean(&[false; 5]));

        // Check whether it works if `num_rows` is 0
        let columnar_value = ColumnarValue::Literal(LiteralValue::TinyInt(2));
        let column = column.into_column(0, &bump);
        assert_eq!(column, Column::TinyInt(&[]));
    }
}
