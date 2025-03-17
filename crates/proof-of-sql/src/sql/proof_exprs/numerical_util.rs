use crate::base::{
    database::{Column, ColumnarValue, LiteralValue},
    scalar::{Scalar, ScalarExt},
};
use bumpalo::Bump;
use core::{cmp::Ordering, ops::Neg};
use itertools::izip;
use num_traits::{NumCast, PrimInt};

#[expect(clippy::cast_sign_loss)]
/// Add or subtract two literals together.
pub(crate) fn add_subtract_literals<S: Scalar>(
    lhs: &LiteralValue,
    rhs: &LiteralValue,
    lhs_scale: i8,
    rhs_scale: i8,
    is_subtract: bool,
) -> S {
    let (lhs_scaled, rhs_scaled) = match lhs_scale.cmp(&rhs_scale) {
        Ordering::Less => {
            let scaling_factor = S::pow10((rhs_scale - lhs_scale) as u8);
            (lhs.to_scalar::<S>() * scaling_factor, rhs.to_scalar())
        }
        Ordering::Equal => (lhs.to_scalar(), rhs.to_scalar()),
        Ordering::Greater => {
            let scaling_factor = S::pow10((lhs_scale - rhs_scale) as u8);
            (lhs.to_scalar(), rhs.to_scalar::<S>() * scaling_factor)
        }
    };
    if is_subtract {
        lhs_scaled - rhs_scaled
    } else {
        lhs_scaled + rhs_scaled
    }
}

#[expect(
    clippy::missing_panics_doc,
    reason = "lhs and rhs are guaranteed to have the same length by design, ensuring no panic occurs"
)]
/// Add or subtract two columns together.
pub(crate) fn add_subtract_columns<'a, S: Scalar>(
    lhs: Column<'a, S>,
    rhs: Column<'a, S>,
    lhs_scale: i8,
    rhs_scale: i8,
    alloc: &'a Bump,
    is_subtract: bool,
) -> &'a [S] {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    assert!(
        lhs_len == rhs_len,
        "lhs and rhs should have the same length"
    );
    let max_scale = lhs_scale.max(rhs_scale);
    let lhs_scalar = lhs.to_scalar_with_scaling(max_scale - lhs_scale);
    let rhs_scalar = rhs.to_scalar_with_scaling(max_scale - rhs_scale);
    let result = alloc.alloc_slice_fill_with(lhs_len, |i| {
        if is_subtract {
            lhs_scalar[i] - rhs_scalar[i]
        } else {
            lhs_scalar[i] + rhs_scalar[i]
        }
    });
    result
}

/// Add or subtract two [`ColumnarValues`] together.
#[expect(dead_code)]
pub(crate) fn add_subtract_columnar_values<'a, S: Scalar>(
    lhs: ColumnarValue<'a, S>,
    rhs: ColumnarValue<'a, S>,
    lhs_scale: i8,
    rhs_scale: i8,
    alloc: &'a Bump,
    is_subtract: bool,
) -> ColumnarValue<'a, S> {
    match (lhs, rhs) {
        (ColumnarValue::Column(lhs), ColumnarValue::Column(rhs)) => {
            ColumnarValue::Column(Column::Scalar(add_subtract_columns(
                lhs,
                rhs,
                lhs_scale,
                rhs_scale,
                alloc,
                is_subtract,
            )))
        }
        (ColumnarValue::Literal(lhs), ColumnarValue::Column(rhs)) => {
            ColumnarValue::Column(Column::Scalar(add_subtract_columns(
                Column::from_literal_with_length(&lhs, rhs.len(), alloc),
                rhs,
                lhs_scale,
                rhs_scale,
                alloc,
                is_subtract,
            )))
        }
        (ColumnarValue::Column(lhs), ColumnarValue::Literal(rhs)) => {
            ColumnarValue::Column(Column::Scalar(add_subtract_columns(
                lhs,
                Column::from_literal_with_length(&rhs, lhs.len(), alloc),
                lhs_scale,
                rhs_scale,
                alloc,
                is_subtract,
            )))
        }
        (ColumnarValue::Literal(lhs), ColumnarValue::Literal(rhs)) => {
            ColumnarValue::Literal(LiteralValue::Scalar(
                add_subtract_literals::<S>(&lhs, &rhs, lhs_scale, rhs_scale, is_subtract).into(),
            ))
        }
    }
}

/// Multiply two columns together.
/// # Panics
/// Panics if: `lhs` and `rhs` are not of the same length.
pub(crate) fn multiply_columns<'a, S: Scalar>(
    lhs: &Column<'a, S>,
    rhs: &Column<'a, S>,
    alloc: &'a Bump,
) -> &'a [S] {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    assert!(
        lhs_len == rhs_len,
        "lhs and rhs should have the same length"
    );
    alloc.alloc_slice_fill_with(lhs_len, |i| {
        lhs.scalar_at(i).unwrap() * rhs.scalar_at(i).unwrap()
    })
}

#[expect(dead_code)]
/// Multiply two [`ColumnarValues`] together.
/// # Panics
/// Panics if: `lhs` and `rhs` are not of the same length.
pub(crate) fn multiply_columnar_values<'a, S: Scalar>(
    lhs: &ColumnarValue<'a, S>,
    rhs: &ColumnarValue<'a, S>,
    alloc: &'a Bump,
) -> ColumnarValue<'a, S> {
    match (lhs, rhs) {
        (ColumnarValue::Column(lhs), ColumnarValue::Column(rhs)) => {
            ColumnarValue::Column(Column::Scalar(multiply_columns(lhs, rhs, alloc)))
        }
        (ColumnarValue::Literal(lhs), ColumnarValue::Column(rhs)) => {
            let lhs_scalar = lhs.to_scalar::<S>();
            let result =
                alloc.alloc_slice_fill_with(rhs.len(), |i| lhs_scalar * rhs.scalar_at(i).unwrap());
            ColumnarValue::Column(Column::Scalar(result))
        }
        (ColumnarValue::Column(lhs), ColumnarValue::Literal(rhs)) => {
            let rhs_scalar = rhs.to_scalar();
            let result =
                alloc.alloc_slice_fill_with(lhs.len(), |i| lhs.scalar_at(i).unwrap() * rhs_scalar);
            ColumnarValue::Column(Column::Scalar(result))
        }
        (ColumnarValue::Literal(lhs), ColumnarValue::Literal(rhs)) => {
            let result = lhs.to_scalar::<S>() * rhs.to_scalar();
            ColumnarValue::Literal(LiteralValue::Scalar(result.into()))
        }
    }
}

/// The counterpart of `add_subtract_columns` for evaluating decimal expressions.
pub(crate) fn scale_and_add_subtract_eval<S: Scalar>(
    lhs_eval: S,
    rhs_eval: S,
    lhs_scale: i8,
    rhs_scale: i8,
    is_subtract: bool,
) -> S {
    let max_scale = lhs_scale.max(rhs_scale);
    let left_scaled_eval = lhs_eval * S::pow10(max_scale.abs_diff(lhs_scale));
    let right_scaled_eval = rhs_eval * S::pow10(max_scale.abs_diff(rhs_scale));
    if is_subtract {
        left_scaled_eval - right_scaled_eval
    } else {
        left_scaled_eval + right_scaled_eval
    }
}

/// Divides two columns of data, where the data types are some signed int type(s).
/// Note that `i128::MIN / -1`, for example, results in a value that is not contained by i128.
/// Therefore, this value wraps around to `i128::MIN`. Division by 0 returns 0.
/// The first slice in the tuple represents this wrapped value, whereas the second is the
/// proper value of the quotient. Note that it is a scalar because it can represent
///  a value (`-i128::MIN`, for example) that is out of range of the integer type.
///
/// # Panics
///
/// Panics when there is a casting issue. These errors should only happen if `is_right_bigger_int_type` is the wrong value.
fn divide_integer_columns<
    'a,
    L: NumCast + Copy + PrimInt + Neg<Output = L>,
    R: NumCast + Copy + PrimInt + Neg<Output = R>,
    S: Scalar + From<L>,
>(
    lhs: &[L],
    rhs: &[R],
    alloc: &'a Bump,
    is_right_bigger_int_type: bool,
) -> (&'a [L], &'a [S]) {
    let division_wrapped = alloc.alloc_slice_fill_with(lhs.len(), |_| L::zero());
    let division = alloc.alloc_slice_fill_with(lhs.len(), |_| S::ZERO);
    for (dw, d, &l, &r) in izip!(&mut *division_wrapped, &mut *division, lhs, rhs) {
        *dw = if l == L::min_value() && r == -R::one() {
            L::min_value()
        } else if r == R::zero() {
            L::zero()
        } else if is_right_bigger_int_type {
            NumCast::from(R::from(l).unwrap() / r).unwrap()
        } else {
            l / L::from(r).unwrap()
        };
        *d = S::from(*dw)
            * (if *dw == L::min_value() && r == -R::one() {
                -S::ONE
            } else {
                S::ONE
            });
    }
    (division_wrapped, division)
}

/// Modulo two columns of data, where the data types are some unsigned int type(s).
/// Note that `i128::MIN % -1`, for example, is unusual in that `i128::MIN / -1`
/// ordinarily returns a value that is not containe dby i128. Division wraps this operation,
/// but modulo still returns 0 here.
/// Division by 0 returns the numerator for modulo.
///
/// # Panics
///
/// Panics when there is a casting issue. These errors should only happen if `is_right_bigger_int_type` is the wrong value.
fn modulo_integer_columns<
    'a,
    L: NumCast + Copy + PrimInt + Neg<Output = L>,
    R: NumCast + Copy + PrimInt + Neg<Output = R>,
>(
    lhs: &[L],
    rhs: &[R],
    alloc: &'a Bump,
    is_right_bigger_int_type: bool,
) -> &'a [L] {
    let remainder = alloc.alloc_slice_fill_with(lhs.len(), |_| L::zero());
    remainder
        .iter_mut()
        .zip(lhs.iter().copied().zip(rhs.iter().copied()))
        .for_each(|(m, (l, r))| {
            *m = if l == L::min_value() && r == -R::one() {
                L::zero()
            } else if r == R::zero() {
                NumCast::from(l).unwrap()
            } else if is_right_bigger_int_type {
                NumCast::from(R::from(l).unwrap() % r).unwrap()
            } else {
                NumCast::from(l % L::from(r).unwrap()).unwrap()
            }
        });
    remainder
}

/// Divide one column by another. The first value in the tuple wraps `MIN / -1` back to `MIN`,
/// whereas the second returns `-MIN`, where `MIN` is the minimum value of a signed int.
/// For now, only signed integer types are supported.
/// # Panics
/// Panics if: `lhs` and `rhs` are not of the same length or column type division is unsupported.
#[expect(clippy::too_many_lines)]
pub(crate) fn divide_columns<'a, S: Scalar>(
    lhs: &Column<'a, S>,
    rhs: &Column<'a, S>,
    alloc: &'a Bump,
) -> (Column<'a, S>, &'a [S]) {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    assert!(
        lhs_len == rhs_len,
        "lhs and rhs should have the same length"
    );
    match (lhs, rhs) {
        (Column::Int128(left), Column::Int128(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::Int128(columns.0), columns.1)
        }
        (Column::Int128(left), Column::BigInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::Int128(columns.0), columns.1)
        }
        (Column::Int128(left), Column::Int(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::Int128(columns.0), columns.1)
        }
        (Column::Int128(left), Column::SmallInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::Int128(columns.0), columns.1)
        }
        (Column::Int128(left), Column::TinyInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::Int128(columns.0), columns.1)
        }
        (Column::BigInt(left), Column::Int128(right)) => {
            let columns = divide_integer_columns(left, right, alloc, true);
            (Column::BigInt(columns.0), columns.1)
        }
        (Column::BigInt(left), Column::BigInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::BigInt(columns.0), columns.1)
        }
        (Column::BigInt(left), Column::Int(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::BigInt(columns.0), columns.1)
        }
        (Column::BigInt(left), Column::SmallInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::BigInt(columns.0), columns.1)
        }
        (Column::BigInt(left), Column::TinyInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::BigInt(columns.0), columns.1)
        }
        (Column::Int(left), Column::Int128(right)) => {
            let columns = divide_integer_columns(left, right, alloc, true);
            (Column::Int(columns.0), columns.1)
        }
        (Column::Int(left), Column::BigInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, true);
            (Column::Int(columns.0), columns.1)
        }
        (Column::Int(left), Column::Int(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::Int(columns.0), columns.1)
        }
        (Column::Int(left), Column::SmallInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::Int(columns.0), columns.1)
        }
        (Column::Int(left), Column::TinyInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::Int(columns.0), columns.1)
        }
        (Column::SmallInt(left), Column::Int128(right)) => {
            let columns = divide_integer_columns(left, right, alloc, true);
            (Column::SmallInt(columns.0), columns.1)
        }
        (Column::SmallInt(left), Column::BigInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, true);
            (Column::SmallInt(columns.0), columns.1)
        }
        (Column::SmallInt(left), Column::Int(right)) => {
            let columns = divide_integer_columns(left, right, alloc, true);
            (Column::SmallInt(columns.0), columns.1)
        }
        (Column::SmallInt(left), Column::SmallInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::SmallInt(columns.0), columns.1)
        }
        (Column::SmallInt(left), Column::TinyInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::SmallInt(columns.0), columns.1)
        }
        (Column::TinyInt(left), Column::Int128(right)) => {
            let columns = divide_integer_columns(left, right, alloc, true);
            (Column::TinyInt(columns.0), columns.1)
        }
        (Column::TinyInt(left), Column::BigInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, true);
            (Column::TinyInt(columns.0), columns.1)
        }
        (Column::TinyInt(left), Column::Int(right)) => {
            let columns = divide_integer_columns(left, right, alloc, true);
            (Column::TinyInt(columns.0), columns.1)
        }
        (Column::TinyInt(left), Column::SmallInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, true);
            (Column::TinyInt(columns.0), columns.1)
        }
        (Column::TinyInt(left), Column::TinyInt(right)) => {
            let columns = divide_integer_columns(left, right, alloc, false);
            (Column::TinyInt(columns.0), columns.1)
        }
        _ => panic!(
            "Division not supported between {} and {}",
            lhs.column_type(),
            rhs.column_type()
        ),
    }
}

/// Take the modulo of one column against another.
/// # Panics
/// Panics if: `lhs` and `rhs` are not of the same length.
pub(crate) fn modulo_columns<'a, S: Scalar>(
    lhs: &Column<'a, S>,
    rhs: &Column<'a, S>,
    alloc: &'a Bump,
) -> Column<'a, S> {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    assert!(
        lhs_len == rhs_len,
        "lhs and rhs should have the same length"
    );

    match (lhs, rhs) {
        (Column::Int128(left), Column::Int128(right)) => {
            Column::Int128(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::Int128(left), Column::BigInt(right)) => {
            Column::Int128(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::Int128(left), Column::Int(right)) => {
            Column::Int128(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::Int128(left), Column::SmallInt(right)) => {
            Column::Int128(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::Int128(left), Column::TinyInt(right)) => {
            Column::Int128(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::BigInt(left), Column::Int128(right)) => {
            Column::BigInt(modulo_integer_columns(left, right, alloc, true))
        }
        (Column::BigInt(left), Column::BigInt(right)) => {
            Column::BigInt(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::BigInt(left), Column::Int(right)) => {
            Column::BigInt(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::BigInt(left), Column::SmallInt(right)) => {
            Column::BigInt(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::BigInt(left), Column::TinyInt(right)) => {
            Column::BigInt(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::Int(left), Column::Int128(right)) => {
            Column::Int(modulo_integer_columns(left, right, alloc, true))
        }
        (Column::Int(left), Column::BigInt(right)) => {
            Column::Int(modulo_integer_columns(left, right, alloc, true))
        }
        (Column::Int(left), Column::Int(right)) => {
            Column::Int(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::Int(left), Column::SmallInt(right)) => {
            Column::Int(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::Int(left), Column::TinyInt(right)) => {
            Column::Int(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::SmallInt(left), Column::Int128(right)) => {
            Column::SmallInt(modulo_integer_columns(left, right, alloc, true))
        }
        (Column::SmallInt(left), Column::BigInt(right)) => {
            Column::SmallInt(modulo_integer_columns(left, right, alloc, true))
        }
        (Column::SmallInt(left), Column::Int(right)) => {
            Column::SmallInt(modulo_integer_columns(left, right, alloc, true))
        }
        (Column::SmallInt(left), Column::SmallInt(right)) => {
            Column::SmallInt(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::SmallInt(left), Column::TinyInt(right)) => {
            Column::SmallInt(modulo_integer_columns(left, right, alloc, false))
        }
        (Column::TinyInt(left), Column::Int128(right)) => {
            Column::TinyInt(modulo_integer_columns(left, right, alloc, true))
        }
        (Column::TinyInt(left), Column::BigInt(right)) => {
            Column::TinyInt(modulo_integer_columns(left, right, alloc, true))
        }
        (Column::TinyInt(left), Column::Int(right)) => {
            Column::TinyInt(modulo_integer_columns(left, right, alloc, true))
        }
        (Column::TinyInt(left), Column::SmallInt(right)) => {
            Column::TinyInt(modulo_integer_columns(left, right, alloc, true))
        }
        (Column::TinyInt(left), Column::TinyInt(right)) => {
            Column::TinyInt(modulo_integer_columns(left, right, alloc, false))
        }
        _ => panic!(
            "Modulo not supported between {} and {}",
            lhs.column_type(),
            rhs.column_type()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{divide_columns, divide_integer_columns};
    use crate::{
        base::{
            database::Column,
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof_exprs::numerical_util::{modulo_columns, modulo_integer_columns},
    };
    use bumpalo::Bump;
    use itertools::{iproduct, Itertools};

    fn verify_tinyint_division(
        lhs: &[i8],
        rhs: &[i8],
        wrapped_quotient: &[i8],
        q: &[i16],
        r: &[i8],
    ) {
        let alloc = Bump::new();
        let quotient = divide_integer_columns::<_, _, TestScalar>(lhs, rhs, &alloc, false);
        let remainder: &[i8] = modulo_integer_columns(lhs, rhs, &alloc, false);
        assert_eq!(quotient.0, wrapped_quotient);
        assert_eq!(
            quotient.1.iter().copied().collect_vec(),
            q.iter().map(TestScalar::from).collect_vec()
        );
        assert_eq!(remainder, r);
    }

    #[test]
    fn we_can_divide_and_modulo_by_different_size_types() {
        let alloc = Bump::new();
        let a: &[i8] = &[2i8, 7, 0, 54];
        let b: &[i128] = &[-1i128, 300, 6, 0];
        let quotient_ab = divide_integer_columns::<_, _, TestScalar>(a, b, &alloc, true);
        let remainder_ab: &[i8] = modulo_integer_columns(a, b, &alloc, true);
        assert_eq!(quotient_ab.0, &[-2i8, 0, 0, 0]);
        assert_eq!(remainder_ab, &[0i8, 7, 0, 54]);
        let quotient_ba = divide_integer_columns::<_, _, TestScalar>(b, a, &alloc, false);
        let remainder_ba: &[i128] = modulo_integer_columns(b, a, &alloc, false);
        assert_eq!(quotient_ba.0, &[0i128, 42, 0, 0]);
        assert_eq!(remainder_ba, &[-1i128, 6, 6, 0]);
    }

    #[test]
    fn we_can_divide_nonnegative_only_columns() {
        verify_tinyint_division(
            &[2, 7, 0, 54],
            &[1, 33, 6, 36],
            &[2, 0, 0, 1],
            &[2, 0, 0, 1],
            &[0, 7, 0, 18],
        );
    }

    #[test]
    fn we_can_divide_nonpositive_only_columns() {
        verify_tinyint_division(
            &[-2, -7, 0, -54],
            &[-1, -33, -6, -36],
            &[2, 0, 0, 1],
            &[2, 0, 0, 1],
            &[0, -7, 0, -18],
        );
    }

    #[test]
    fn we_can_divide_nonpositive_numerator_and_positive_denominator_columns() {
        verify_tinyint_division(
            &[-2, -7, 0, -54],
            &[1, 33, 6, 36],
            &[-2, 0, 0, -1],
            &[-2, 0, 0, -1],
            &[0, -7, 0, -18],
        );
    }

    #[test]
    fn we_can_divide_nonnegative_numerator_and_negative_denominator_columns() {
        verify_tinyint_division(
            &[2, 7, 0, 54],
            &[-1, -33, -6, -36],
            &[-2, 0, 0, -1],
            &[-2, 0, 0, -1],
            &[0, 7, 0, 18],
        );
    }

    #[test]
    fn we_can_divide_zero_denominator_columns() {
        verify_tinyint_division(
            &[1, -1, 0, i8::MAX, i8::MIN],
            &[0, 0, 0, 0, 0],
            &[0, 0, 0, 0, 0],
            &[0, 0, 0, 0, 0],
            &[1, -1, 0, i8::MAX, i8::MIN],
        );
    }

    #[test]
    fn we_can_divide_minmax_numerator_and_plusminusonezero_denominator_columns() {
        verify_tinyint_division(
            &[i8::MAX, i8::MIN, i8::MAX, i8::MIN, i8::MAX, i8::MIN],
            &[1, 1, -1, -1, 0, 0],
            &[i8::MAX, i8::MIN, -i8::MAX, i8::MIN, 0, 0],
            &[
                i16::from(i8::MAX),
                i16::from(i8::MIN),
                -i16::from(i8::MAX),
                -i16::from(i8::MIN),
                0,
                0,
            ],
            &[0, 0, 0, 0, i8::MAX, i8::MIN],
        );
    }

    #[test]
    fn we_can_divide_columns_for_each_type() {
        let alloc = Bump::new();
        let tiny_int_column: Column<'_, TestScalar> = Column::<'_, TestScalar>::TinyInt(&[1]);
        let small_int_column = Column::<'_, TestScalar>::SmallInt(&[1]);
        let int_column = Column::<'_, TestScalar>::Int(&[1]);
        let big_int_column = Column::<'_, TestScalar>::BigInt(&[1]);
        let int128_column = Column::<'_, TestScalar>::Int128(&[1]);
        let columns = [
            tiny_int_column,
            small_int_column,
            int_column,
            big_int_column,
            int128_column,
        ];
        let scalar_column = [TestScalar::ONE].as_slice();
        for (numerator, denominator) in iproduct!(columns, columns) {
            let quotient = divide_columns(&numerator, &denominator, &alloc);
            assert_eq!(quotient, (numerator, scalar_column));
        }
    }

    /// The primary purpose of this test is to verify that the remainder column has the correct variant of `Column`.
    /// We use 0 % 0 for convenience, because that happens to be defined as 0 for `modulo_columns`, so all the columns are the same.
    #[test]
    fn we_can_modulo_columns_for_each_type() {
        let alloc = Bump::new();
        let tiny_int_column: Column<'_, TestScalar> = Column::<'_, TestScalar>::TinyInt(&[0]);
        let small_int_column = Column::<'_, TestScalar>::SmallInt(&[0]);
        let int_column = Column::<'_, TestScalar>::Int(&[0]);
        let big_int_column = Column::<'_, TestScalar>::BigInt(&[0]);
        let int128_column = Column::<'_, TestScalar>::Int128(&[0]);
        let columns = [
            tiny_int_column,
            small_int_column,
            int_column,
            big_int_column,
            int128_column,
        ];
        for (numerator, denominator) in iproduct!(columns, columns) {
            let remainder = modulo_columns(&numerator, &denominator, &alloc);
            assert_eq!(remainder, numerator);
        }
    }

    #[should_panic(expected = "lhs and rhs should have the same length")]
    #[test]
    fn we_can_error_divide_columns_if_columns_are_different_length() {
        let alloc = Bump::new();
        let tiny_int_column: Column<'_, TestScalar> = Column::<'_, TestScalar>::TinyInt(&[1, 1]);
        let small_int_column: Column<'_, TestScalar> = Column::<'_, TestScalar>::SmallInt(&[2]);
        divide_columns(&tiny_int_column, &small_int_column, &alloc);
    }

    #[should_panic(expected = "lhs and rhs should have the same length")]
    #[test]
    fn we_can_error_modulo_columns_if_columns_are_different_length() {
        let alloc = Bump::new();
        let tiny_int_column: Column<'_, TestScalar> = Column::<'_, TestScalar>::TinyInt(&[1, 1]);
        let small_int_column: Column<'_, TestScalar> = Column::<'_, TestScalar>::SmallInt(&[2]);
        modulo_columns(&tiny_int_column, &small_int_column, &alloc);
    }

    #[should_panic(expected = "Modulo not supported between UINT8 and SMALLINT")]
    #[test]
    fn we_can_error_modulo_columns_if_columns_are_unsupported_types() {
        let alloc = Bump::new();
        let unsigned_int_column: Column<'_, TestScalar> = Column::<'_, TestScalar>::Uint8(&[1, 1]);
        let small_int_column: Column<'_, TestScalar> = Column::<'_, TestScalar>::SmallInt(&[2, 2]);
        modulo_columns(&unsigned_int_column, &small_int_column, &alloc);
    }

    #[should_panic(expected = "Division not supported between UINT8 and SMALLINT")]
    #[test]
    fn we_can_error_divide_columns_if_columns_are_unsupported_types() {
        let alloc = Bump::new();
        let unsigned_int_column: Column<'_, TestScalar> = Column::<'_, TestScalar>::Uint8(&[1, 1]);
        let small_int_column: Column<'_, TestScalar> = Column::<'_, TestScalar>::SmallInt(&[2, 2]);
        divide_columns(&unsigned_int_column, &small_int_column, &alloc);
    }
}
