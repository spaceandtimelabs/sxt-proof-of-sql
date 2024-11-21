use super::{
    slice_operation::{repeat_elementwise, repeat_slice},
    Column, ColumnType,
};
use crate::base::scalar::Scalar;
use alloc::vec::Vec;
use bumpalo::Bump;

#[allow(dead_code)]
pub trait RepetitionOp {
    fn op<T: Clone>(column: &[T], n: usize) -> Vec<T>;

    /// Run a column repetition operation on a `Column`.
    fn column_op<'a, S>(column: &Column<'a, S>, alloc: &'a Bump, n: usize) -> Column<'a, S>
    where
        S: Scalar,
    {
        let len = n * column.len();
        match column.column_type() {
            ColumnType::Boolean => Column::Boolean(
                alloc.alloc_slice_fill_with(
                    len,
                    Self::op(column.as_boolean().expect("Column types should match"), n)
                        .next()
                        .expect("The element should exist"),
                ) as &[_],
            ),
            ColumnType::TinyInt => Column::TinyInt(
                alloc.alloc_slice_fill_with(
                    len,
                    Self::op(column.as_tinyint().expect("Column types should match"), n)
                        .next()
                        .expect("The element should exist"),
                ) as &[_],
            ),
            ColumnType::SmallInt => Column::SmallInt(
                alloc.alloc_slice_fill_with(
                    len,
                    Self::op(column.as_smallint().expect("Column types should match"), n)
                        .next()
                        .expect("The element should exist"),
                ) as &[_],
            ),
            ColumnType::Int => Column::Int(
                alloc.alloc_slice_fill_with(
                    len,
                    Self::op(column.as_int().expect("Column types should match"), n)
                        .next()
                        .expect("The element should exist"),
                ) as &[_],
            ),
            ColumnType::BigInt => Column::BigInt(
                alloc.alloc_slice_fill_with(
                    len,
                    Self::op(column.as_bigint().expect("Column types should match"), n)
                        .next()
                        .expect("The element should exist"),
                ) as &[_],
            ),
            ColumnType::Int128 => Column::Int128(
                alloc.alloc_slice_fill_with(
                    len,
                    Self::op(column.as_int128().expect("Column types should match"), n)
                        .next()
                        .expect("The element should exist"),
                ) as &[_],
            ),
            ColumnType::Scalar => Column::Scalar(
                alloc.alloc_slice_fill_with(
                    len,
                    Self::op(column.as_scalar().expect("Column types should match"), n)
                        .next()
                        .expect("The element should exist"),
                ) as &[_],
            ),
            ColumnType::Decimal75(precision, scale) => Column::Decimal75(
                precision,
                scale,
                alloc.alloc_slice_fill_with(
                    len,
                    Self::op(column.as_decimal75().expect("Column types should match"), n)
                        .next()
                        .expect("The element should exist"),
                ) as &[_],
            ),
            ColumnType::VarChar => {
                let (raw_result, raw_scalars) =
                    column.as_varchar().expect("Column types should match");
                Column::VarChar((
                    alloc.alloc_slice_fill_with(
                        len,
                        Self::op(raw_result, n)
                            .next()
                            .expect("The element should exist"),
                    ) as &[_],
                    alloc.alloc_slice_fill_with(
                        len,
                        Self::op(raw_scalars, n)
                            .next()
                            .expect("The element should exist"),
                    ) as &[_],
                ))
            }
            ColumnType::TimestampTZ(tu, tz) => Column::TimestampTZ(
                tu,
                tz,
                alloc.alloc_slice_fill_with(
                    len,
                    Self::op(
                        column.as_timestamptz().expect("Column types should match"),
                        n,
                    )
                    .next()
                    .expect("The element should exist"),
                ) as &[_],
            ),
        }
    }
}

pub struct ColumnRepeatOp {}
impl RepetitionOp for ColumnRepeatOp {
    fn op<T: Clone>(column: &[T], n: usize) -> Vec<T> {
        repeat_slice(column, n)
    }
}

pub struct ElementwiseRepeatOp {}
impl RepetitionOp for ElementwiseRepeatOp {
    fn op<T: Clone>(column: &[T], n: usize) -> Vec<T> {
        repeat_elementwise(column, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn test_column_repetition_op() {
        let bump = Bump::new();

        let column: Column<TestScalar> = Column::Int(&[1, 2, 3]);
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_int().unwrap(), &[1, 2, 3, 1, 2, 3]);

        // Varchar
        let strings = vec!["a", "b", "c"];
        let scalars = strings.iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::VarChar((&strings, &scalars));
        let result = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        let doubled_strings = vec!["a", "b", "c", "a", "b", "c"];
        let doubled_scalars = doubled_strings
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(
            result,
            Column::VarChar((&doubled_strings, &doubled_scalars))
        );
    }

    #[test]
    fn test_elementwise_repetition_op() {
        let bump = Bump::new();

        let column: Column<TestScalar> = Column::Int(&[1, 2, 3]);
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        assert_eq!(result.as_int().unwrap(), &[1, 1, 2, 2, 3, 3]);

        // Varchar
        let strings = vec!["a", "b", "c"];
        let scalars = strings.iter().map(TestScalar::from).collect::<Vec<_>>();
        let column: Column<TestScalar> = Column::VarChar((&strings, &scalars));
        let result = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);
        let doubled_strings = vec!["a", "a", "b", "b", "c", "c"];
        let doubled_scalars = doubled_strings
            .iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        assert_eq!(
            result,
            Column::VarChar((&doubled_strings, &doubled_scalars))
        );
    }
}
