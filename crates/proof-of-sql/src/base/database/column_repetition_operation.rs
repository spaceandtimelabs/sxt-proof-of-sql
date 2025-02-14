use super::{
    slice_operation::{repeat_elementwise, repeat_slice},
    Column, ColumnType,
};
use crate::base::scalar::Scalar;
use alloc::vec::Vec;
use bumpalo::Bump;
use core::{iter, iter::Iterator};

pub trait RepetitionOp {
    fn op<T: Clone>(column: &[T], n: usize) -> impl Iterator<Item = T>;

    // Special case for fixed-size binary columns.
    fn op_fixed_size_binary(col_bytes: &[u8], width: usize, n: usize) -> Vec<u8>;

    /// Run a column repetition operation on a `Column`.
    #[allow(clippy::too_many_lines)]
    fn column_op<'a, S>(column: &Column<'a, S>, alloc: &'a Bump, n: usize) -> Column<'a, S>
    where
        S: Scalar,
    {
        let len = n * column.len();
        match column.column_type() {
            ColumnType::Boolean => {
                let mut iter = Self::op(column.as_boolean().expect("Column types should match"), n);
                Column::Boolean(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::Uint8 => {
                let mut iter = Self::op(column.as_uint8().expect("Column types should match"), n);
                Column::Uint8(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::TinyInt => {
                let mut iter = Self::op(column.as_tinyint().expect("Column types should match"), n);
                Column::TinyInt(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::SmallInt => {
                let mut iter =
                    Self::op(column.as_smallint().expect("Column types should match"), n);
                Column::SmallInt(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::Int => {
                let mut iter = Self::op(column.as_int().expect("Column types should match"), n);
                Column::Int(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::BigInt => {
                let mut iter = Self::op(column.as_bigint().expect("Column types should match"), n);
                Column::BigInt(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::Int128 => {
                let mut iter = Self::op(column.as_int128().expect("Column types should match"), n);
                Column::Int128(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::Scalar => {
                let mut iter = Self::op(column.as_scalar().expect("Column types should match"), n);
                Column::Scalar(alloc.alloc_slice_fill_with(len, |_| {
                    iter.next().expect("Iterator should have enough elements")
                }) as &[_])
            }
            ColumnType::Decimal75(precision, scale) => {
                let mut iter =
                    Self::op(column.as_decimal75().expect("Column types should match"), n);
                Column::Decimal75(
                    precision,
                    scale,
                    alloc.alloc_slice_fill_with(len, |_| {
                        iter.next().expect("Iterator should have enough elements")
                    }) as &[_],
                )
            }
            ColumnType::VarChar => {
                let (raw_result, raw_scalars) =
                    column.as_varchar().expect("Column types should match");

                // Create iterators for both the result and scalars
                let mut result_iter = Self::op(raw_result, n);
                let mut scalar_iter = Self::op(raw_scalars, n);

                Column::VarChar((
                    alloc.alloc_slice_fill_with(len, |_| {
                        result_iter
                            .next()
                            .expect("Iterator should have enough elements")
                    }) as &[_],
                    alloc.alloc_slice_fill_with(len, |_| {
                        scalar_iter
                            .next()
                            .expect("Iterator should have enough elements")
                    }) as &[_],
                ))
            }
            ColumnType::TimestampTZ(tu, tz) => {
                let mut iter = Self::op(
                    column.as_timestamptz().expect("Column types should match"),
                    n,
                );
                Column::TimestampTZ(
                    tu,
                    tz,
                    alloc.alloc_slice_fill_with(len, |_| {
                        iter.next().expect("Iterator should have enough elements")
                    }) as &[_],
                )
            }
            ColumnType::FixedSizeBinary(width) => {
                // get the existing bytes
                let col_bytes = column.as_fixed_size_binary().expect("Column types match").1;
                let bw = width.width_as_usize();

                // call the new trait method, which is specialized
                let repeated_bytes = Self::op_fixed_size_binary(col_bytes, bw, n);

                // copy the repeated result into Bump
                let allocated = alloc.alloc_slice_copy(&repeated_bytes);
                Column::FixedSizeBinary(width, allocated)
            }
        }
    }
}

pub struct ColumnRepeatOp {}
impl RepetitionOp for ColumnRepeatOp {
    fn op<T: Clone>(column: &[T], n: usize) -> impl Iterator<Item = T> {
        repeat_slice(column, n)
    }

    fn op_fixed_size_binary(col_bytes: &[u8], width: usize, n: usize) -> Vec<u8> {
        let rows: Vec<_> = col_bytes.chunks_exact(width).collect();
        let mut out = Vec::with_capacity(rows.len() * width * n);
        for row in rows.iter().cycle().take(rows.len() * n) {
            out.extend_from_slice(row);
        }
        out
    }
}

pub struct ElementwiseRepeatOp {}
impl RepetitionOp for ElementwiseRepeatOp {
    fn op<T: Clone>(column: &[T], n: usize) -> impl Iterator<Item = T> {
        repeat_elementwise(column, n)
    }

    fn op_fixed_size_binary(col_bytes: &[u8], width: usize, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(col_bytes.len() * n);
        out.extend(
            col_bytes
                .chunks_exact(width)
                .flat_map(|row| iter::repeat(row).take(n))
                .flatten()
                .copied(),
        );
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{math::non_negative_i32::NonNegativeI32, scalar::test_scalar::TestScalar};

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

    #[test]
    fn test_column_repetition_op_fixedsizebinary() {
        let bump = Bump::new();

        // define a 3-row column: row0 => i32=1, row1 => i32=2, row2 => i32=3
        // in little-endian, each row is 4 bytes
        let row0 = 1_i32.to_le_bytes();
        let row1 = 2_i32.to_le_bytes();
        let row2 = 3_i32.to_le_bytes();

        // concatenate into a single buffer
        let mut bytes = Vec::with_capacity(3 * 4);
        bytes.extend_from_slice(&row0);
        bytes.extend_from_slice(&row1);
        bytes.extend_from_slice(&row2);

        // construct the column
        let width = NonNegativeI32::new(4).unwrap();
        let column: Column<TestScalar> = Column::FixedSizeBinary(width, &bytes);

        // apply ColumnRepeatOp with n=2 => we repeat the entire column in sequence:
        // result => row0, row1, row2, row0, row1, row2
        let repeated = ColumnRepeatOp::column_op::<TestScalar>(&column, &bump, 2);

        // build the expected 6-row sequence
        let mut expected_bytes = Vec::with_capacity(6 * 4);
        // original 3 rows
        expected_bytes.extend_from_slice(&row0);
        expected_bytes.extend_from_slice(&row1);
        expected_bytes.extend_from_slice(&row2);
        // repeated again
        expected_bytes.extend_from_slice(&row0);
        expected_bytes.extend_from_slice(&row1);
        expected_bytes.extend_from_slice(&row2);

        let expected = Column::FixedSizeBinary(width, &expected_bytes);
        assert_eq!(repeated, expected);
    }

    #[test]
    fn test_elementwise_repetition_op_fixedsizebinary() {
        let bump = Bump::new();

        // define 3 rows, each 4 bytes in little-endian i32 format.
        //   row0 => i32=1
        //   row1 => i32=2
        //   row2 => i32=3
        let row0 = 1_i32.to_le_bytes();
        let row1 = 2_i32.to_le_bytes();
        let row2 = 3_i32.to_le_bytes();

        // concatenate into a single buffer of 12 bytes (3 rows × 4 bytes each).
        let mut bytes = Vec::with_capacity(3 * 4);
        bytes.extend_from_slice(&row0);
        bytes.extend_from_slice(&row1);
        bytes.extend_from_slice(&row2);

        let width = NonNegativeI32::new(4).unwrap();
        let column: Column<TestScalar> = Column::FixedSizeBinary(width, &bytes);

        // call "ElementwiseRepeatOp" with n=2 => each row duplicated in place
        // so we expect row0,row0, row1,row1, row2,row2.
        let repeated = ElementwiseRepeatOp::column_op::<TestScalar>(&column, &bump, 2);

        // build the expected 6-row buffer (6 × 4 = 24 bytes)
        let mut expected_bytes = Vec::with_capacity(6 * 4);
        expected_bytes.extend_from_slice(&row0); // row0 repeated
        expected_bytes.extend_from_slice(&row0);
        expected_bytes.extend_from_slice(&row1); // row1 repeated
        expected_bytes.extend_from_slice(&row1);
        expected_bytes.extend_from_slice(&row2); // row2 repeated
        expected_bytes.extend_from_slice(&row2);

        let expected = Column::FixedSizeBinary(width, &expected_bytes);
        assert_eq!(repeated, expected);
    }
}
