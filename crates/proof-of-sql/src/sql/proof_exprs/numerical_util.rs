use crate::base::{
    database::{
        try_cast_types, try_decimal_scale_cast_types, Column, ColumnOperationResult, ColumnType,
    },
    math::decimal::Precision,
    scalar::{Scalar, ScalarExt},
};
use alloc::format;
use bnum::types::U256;
use bumpalo::Bump;
use core::{convert::TryInto, ops::Neg};
use itertools::izip;
use num_traits::{NumCast, PrimInt};

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
#[cfg_attr(not(test), expect(dead_code))]
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
#[cfg_attr(not(test), expect(dead_code))]
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

fn cast_bool_slice_to_signed_int_slice<'a, I: PrimInt>(
    alloc: &'a Bump,
    column: &[bool],
) -> &'a [I] {
    let cast_column = alloc.alloc_slice_fill_copy(column.len(), I::zero());
    for (cast_val, bool_val) in cast_column.iter_mut().zip(column) {
        *cast_val = if *bool_val { I::one() } else { I::zero() }
    }
    cast_column
}

/// Handles the casting of a bool column to a signed type
///
/// # Panics
/// Panics if casting is not supported between the two types
fn cast_bool_column_to_signed_int_column<'a, S: Scalar>(
    alloc: &'a Bump,
    column: &[bool],
    to_type: ColumnType,
) -> Column<'a, S> {
    match to_type {
        ColumnType::TinyInt => Column::TinyInt(cast_bool_slice_to_signed_int_slice(alloc, column)),
        ColumnType::SmallInt => {
            Column::SmallInt(cast_bool_slice_to_signed_int_slice(alloc, column))
        }
        ColumnType::Int => Column::Int(cast_bool_slice_to_signed_int_slice(alloc, column)),
        ColumnType::BigInt => Column::BigInt(cast_bool_slice_to_signed_int_slice(alloc, column)),
        ColumnType::Int128 => Column::Int128(cast_bool_slice_to_signed_int_slice(alloc, column)),
        _ => panic!(
            "Casting not supported between {} and {}",
            ColumnType::Boolean,
            to_type
        ),
    }
}

/// # Panics
/// Panics if `I` cannot be cast to `O`
fn cast_int_slice_to_int_slice<'a, I: NumCast + PrimInt, O: NumCast + PrimInt>(
    alloc: &'a Bump,
    column: &[I],
) -> &'a [O] {
    alloc.alloc_slice_fill_iter(column.iter().copied().map(|i| NumCast::from(i).unwrap()))
}

/// # Panics
/// Panics if the to type is not supported
fn cast_int_slice_to_int_column<'a, S: Scalar, I: NumCast + PrimInt>(
    alloc: &'a Bump,
    column: &[I],
    to_type: ColumnType,
) -> Column<'a, S> {
    match to_type {
        ColumnType::Uint8 => Column::Uint8(cast_int_slice_to_int_slice(alloc, column)),
        ColumnType::TinyInt => Column::TinyInt(cast_int_slice_to_int_slice(alloc, column)),
        ColumnType::SmallInt => Column::SmallInt(cast_int_slice_to_int_slice(alloc, column)),
        ColumnType::Int => Column::Int(cast_int_slice_to_int_slice(alloc, column)),
        ColumnType::BigInt => Column::BigInt(cast_int_slice_to_int_slice(alloc, column)),
        ColumnType::Int128 => Column::Int128(cast_int_slice_to_int_slice(alloc, column)),
        _ => panic!("Unsupported cast from int type to {to_type}"),
    }
}

/// # Panics
/// Panics if the from type is not supported
fn cast_int_column_to_int_column<'a, S: Scalar>(
    alloc: &'a Bump,
    from_column: Column<'a, S>,
    to_type: ColumnType,
) -> Column<'a, S> {
    match from_column {
        Column::Uint8(column) => cast_int_slice_to_int_column(alloc, column, to_type),
        Column::TinyInt(column) => cast_int_slice_to_int_column(alloc, column, to_type),
        Column::SmallInt(column) => cast_int_slice_to_int_column(alloc, column, to_type),
        Column::Int(column) => cast_int_slice_to_int_column(alloc, column, to_type),
        Column::BigInt(column) => cast_int_slice_to_int_column(alloc, column, to_type),
        Column::Int128(column) => cast_int_slice_to_int_column(alloc, column, to_type),
        _ => panic!(
            "{}",
            format!(
                "Unsupported cast from {} to {to_type}",
                from_column.column_type()
            )
        ),
    }
}

/// Cast a slice of [`Scalar`]s to a slice of ints
///
/// # Panics
/// Panics if casting fails on any element
fn cast_scalar_slice_to_int_slice<'a, I: Copy, S: Scalar + TryInto<I>>(
    alloc: &'a Bump,
    column: &[S],
) -> &'a [I] {
    alloc.alloc_slice_fill_iter(column.iter().map(|s| {
        TryInto::<I>::try_into(*s)
            .map_err(|_| format!("Failed to cast {} to {}", s, core::any::type_name::<I>()))
            .unwrap()
    }))
}

/// Cast a slice of [`Scalar`]s to a [`Column`] of ints
///
/// # Panics
/// Panics if casting fails on any element
fn cast_scalar_slice_to_int_column<'a, S: Scalar>(
    alloc: &'a Bump,
    column: &[S],
    to_type: ColumnType,
) -> Column<'a, S> {
    match to_type {
        ColumnType::Uint8 => Column::Uint8(cast_scalar_slice_to_int_slice::<u8, S>(alloc, column)),
        ColumnType::TinyInt => {
            Column::TinyInt(cast_scalar_slice_to_int_slice::<i8, S>(alloc, column))
        }
        ColumnType::SmallInt => {
            Column::SmallInt(cast_scalar_slice_to_int_slice::<i16, S>(alloc, column))
        }
        ColumnType::Int => Column::Int(cast_scalar_slice_to_int_slice::<i32, S>(alloc, column)),
        ColumnType::BigInt => {
            Column::BigInt(cast_scalar_slice_to_int_slice::<i64, S>(alloc, column))
        }
        ColumnType::Int128 => {
            Column::Int128(cast_scalar_slice_to_int_slice::<i128, S>(alloc, column))
        }
        _ => panic!("Unsupported cast from int type to {to_type}"),
    }
}

/// Handles the casting of one column to another
///
/// # Panics
/// Panics if casting is not supported between the two types
pub fn cast_column<'a, S: Scalar>(
    alloc: &'a Bump,
    from_column: Column<'a, S>,
    from_type: ColumnType,
    to_type: ColumnType,
) -> Column<'a, S> {
    try_cast_types(from_type, to_type)
        .unwrap_or_else(|_| panic!("Unable to cast between types {from_type} and {to_type}"));
    match (from_column, to_type) {
        (
            Column::Boolean(vals),
            ColumnType::TinyInt
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::BigInt
            | ColumnType::Int128,
        ) => cast_bool_column_to_signed_int_column(alloc, vals, to_type),
        (
            Column::TinyInt(_)
            | Column::Uint8(_)
            | Column::SmallInt(_)
            | Column::Int(_)
            | Column::BigInt(_)
            | Column::Int128(_),
            ColumnType::Decimal75(precision, 0),
        ) => Column::Decimal75(
            precision,
            0,
            alloc.alloc_slice_fill_with(from_column.len(), |i| from_column.scalar_at(i).unwrap())
                as &[_],
        ),
        (Column::Decimal75(_, from_scale, scalars), ColumnType::Decimal75(precision, to_scale)) => {
            assert_eq!(
                from_scale, to_scale,
                "Casting not supported between {from_type} and {to_type}"
            );
            Column::Decimal75(precision, to_scale, scalars)
        }
        (
            Column::TinyInt(_)
            | Column::Uint8(_)
            | Column::SmallInt(_)
            | Column::Int(_)
            | Column::BigInt(_)
            | Column::Int128(_),
            ColumnType::TinyInt
            | ColumnType::Uint8
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::BigInt
            | ColumnType::Int128,
        ) => cast_int_column_to_int_column(alloc, from_column, to_type),
        (Column::TimestampTZ(_, _, vals), ColumnType::BigInt) => Column::BigInt(vals),
        // This is due to the current arithmetic expressions causing results to be scalars
        (
            Column::Scalar(vals),
            ColumnType::TinyInt
            | ColumnType::Uint8
            | ColumnType::SmallInt
            | ColumnType::Int
            | ColumnType::BigInt
            | ColumnType::Int128,
        ) => {
            let from_scale = from_type.scale().unwrap();
            assert_eq!(
                from_scale, 0,
                "Casting not supported between {from_type} and {to_type}"
            );
            cast_scalar_slice_to_int_column(alloc, vals, to_type)
        }
        (Column::Scalar(vals), ColumnType::Decimal75(to_precision, to_scale)) => {
            let from_scale = from_type.scale().unwrap();
            assert_eq!(
                from_scale, to_scale,
                "Casting not supported between {from_type} and {to_type}"
            );
            Column::Decimal75(to_precision, to_scale, vals)
        }
        _ => panic!("Casting not supported between {from_type} and {to_type}"),
    }
}

/// Tries to get the scale factor between the from and to types.
/// The precision and scale are returned along with the scale so that the unwrapping
/// can occur in the function that confirms that the types are castable
#[expect(clippy::missing_panics_doc)]
pub fn try_get_scaling_factor_with_precision_and_scale(
    from_type: ColumnType,
    to_type: ColumnType,
) -> ColumnOperationResult<(U256, u8, i8)> {
    try_decimal_scale_cast_types(from_type, to_type)?;
    let to_precision = to_type.precision_value().unwrap();
    let to_scale = to_type.scale().unwrap();
    let power = u32::try_from(to_scale - from_type.scale().unwrap()).unwrap();
    Ok((U256::TEN.pow(power), to_precision, to_scale))
}

/// Casts `from_column` to a column with a column type of `to_type`
///
/// # Panics
/// Panics if casting is invalid between the two types
pub fn cast_column_to_decimal_with_scaling<'a, S: Scalar>(
    alloc: &'a Bump,
    from_column: Column<'a, S>,
    to_type: ColumnType,
) -> Column<'a, S> {
    let from_type = from_column.column_type();
    let (scaling_factor, precision, scale) =
        try_get_scaling_factor_with_precision_and_scale(from_type, to_type).unwrap_or_else(|_| {
            panic!("Unable to get scaling factor between types {from_type} and {to_type}")
        });
    let cast_scalars = alloc.alloc_slice_fill_with(from_column.len(), |i| {
        S::from_wrapping(scaling_factor) * from_column.scalar_at(i).unwrap()
    });
    Column::Decimal75(
        Precision::new(precision).unwrap(),
        scale,
        cast_scalars as &[_],
    )
}

#[cfg(test)]
mod tests {
    use super::{
        cast_bool_column_to_signed_int_column, cast_column, cast_int_slice_to_int_column,
        divide_columns, divide_integer_columns,
    };
    use crate::{
        base::{
            database::{try_cast_types, Column, ColumnType},
            math::decimal::Precision,
            posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
            scalar::{test_scalar::TestScalar, Scalar},
        },
        sql::proof_exprs::numerical_util::{
            cast_column_to_decimal_with_scaling, modulo_columns, modulo_integer_columns,
            try_get_scaling_factor_with_precision_and_scale,
        },
    };
    use bnum::types::U256;
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

    #[test]
    fn we_can_cast_bool_columns_to_signed_int_column() {
        let alloc = Bump::new();
        let bool_column = Column::<TestScalar>::Boolean(&[true, false, true]);
        let expected_tiny_int_column = Column::<TestScalar>::TinyInt(&[1i8, 0, 1]);
        let expected_small_int_column = Column::<TestScalar>::SmallInt(&[1i16, 0, 1]);
        let expected_int_column = Column::<TestScalar>::Int(&[1i32, 0, 1]);
        let expected_big_int_column = Column::<TestScalar>::BigInt(&[1i64, 0, 1]);
        let expected_int_128_column = Column::<TestScalar>::Int128(&[1i128, 0, 1]);
        for expected_signed_column in [
            expected_tiny_int_column,
            expected_small_int_column,
            expected_int_column,
            expected_int_128_column,
            expected_big_int_column,
        ] {
            let signed_column = cast_column(
                &alloc,
                bool_column,
                ColumnType::Boolean,
                expected_signed_column.column_type(),
            );
            assert_eq!(signed_column, expected_signed_column);
        }
    }

    #[test]
    fn we_can_cast_numeric_types_to_numeric_types_when_castable() {
        let alloc = Bump::new();
        let small_decimal_column =
            Column::<TestScalar>::Decimal75(Precision::new(2).unwrap(), 0, &[TestScalar::ONE]);
        let uint8_column = Column::<TestScalar>::Uint8(&[1u8]);
        let tiny_int_column = Column::<TestScalar>::TinyInt(&[1i8]);
        let small_int_column = Column::<TestScalar>::SmallInt(&[1i16]);
        let int_column = Column::<TestScalar>::Int(&[1i32]);
        let big_int_column = Column::<TestScalar>::BigInt(&[1i64]);
        let int_128_column = Column::<TestScalar>::Int128(&[1i128]);
        let big_decimal_column =
            Column::<TestScalar>::Decimal75(Precision::new(2).unwrap(), 0, &[TestScalar::ONE]);
        for (from_column, to_column) in iproduct!(
            [
                small_decimal_column,
                uint8_column,
                tiny_int_column,
                small_int_column,
                int_column,
                big_int_column,
                int_128_column
            ],
            [
                uint8_column,
                tiny_int_column,
                small_int_column,
                int_column,
                big_int_column,
                int_128_column,
                big_decimal_column
            ]
        ) {
            let to_type = to_column.column_type();
            if let Ok(()) = try_cast_types(from_column.column_type(), to_type) {
                assert_eq!(
                    cast_column(&alloc, from_column, from_column.column_type(), to_type),
                    to_column
                );
            }
        }
    }

    #[test]
    fn we_can_cast_scalar_columns_with_numeric_types_to_numeric_types_when_castable() {
        let alloc = Bump::new();
        let scalar_column = Column::<TestScalar>::Scalar(&[TestScalar::ONE]);
        let uint8_column = Column::<TestScalar>::Uint8(&[1u8]);
        let tiny_int_column = Column::<TestScalar>::TinyInt(&[1i8]);
        let small_int_column = Column::<TestScalar>::SmallInt(&[1i16]);
        let int_column = Column::<TestScalar>::Int(&[1i32]);
        let big_int_column = Column::<TestScalar>::BigInt(&[1i64]);
        let int_128_column = Column::<TestScalar>::Int128(&[1i128]);
        let big_decimal_column =
            Column::<TestScalar>::Decimal75(Precision::new(2).unwrap(), 0, &[TestScalar::ONE]);
        for (from_type, to_column) in iproduct!(
            [
                ColumnType::Decimal75(Precision::new(2).unwrap(), 0),
                ColumnType::Uint8,
                ColumnType::TinyInt,
                ColumnType::SmallInt,
                ColumnType::Int,
                ColumnType::BigInt,
                ColumnType::Int128
            ],
            [
                uint8_column,
                tiny_int_column,
                small_int_column,
                int_column,
                big_int_column,
                int_128_column,
                big_decimal_column
            ]
        ) {
            let to_type = to_column.column_type();
            if let Ok(()) = try_cast_types(from_type, to_type) {
                assert_eq!(
                    cast_column(&alloc, scalar_column, from_type, to_type),
                    to_column
                );
            }
        }
    }

    #[test]
    fn we_can_cast_decimal_column_to_decimal_column_with_same_scale() {
        let alloc = Bump::new();
        let decimal_column_with_scale =
            Column::<TestScalar>::Decimal75(Precision::new(2).unwrap(), 1, &[TestScalar::ONE]);
        let res = cast_column(
            &alloc,
            decimal_column_with_scale,
            ColumnType::Decimal75(Precision::new(2).unwrap(), 1),
            ColumnType::Decimal75(Precision::new(3).unwrap(), 1),
        );
        assert_eq!(
            res,
            Column::<TestScalar>::Decimal75(Precision::new(3).unwrap(), 1, &[TestScalar::ONE])
        );
    }

    #[test]
    fn we_can_cast_scalar_column_with_decimal_type_to_decimal_column_with_same_scale() {
        let alloc = Bump::new();
        let scalar_column = Column::<TestScalar>::Scalar(&[TestScalar::ONE]);
        let res = cast_column(
            &alloc,
            scalar_column,
            ColumnType::Decimal75(Precision::new(2).unwrap(), 1),
            ColumnType::Decimal75(Precision::new(3).unwrap(), 1),
        );
        assert_eq!(
            res,
            Column::<TestScalar>::Decimal75(Precision::new(3).unwrap(), 1, &[TestScalar::ONE])
        );
    }

    #[test]
    fn we_can_cast_timestamp_column_to_bigint_column() {
        let alloc = Bump::new();
        let timestamp_column = Column::<TestScalar>::TimestampTZ(
            PoSQLTimeUnit::Microsecond,
            PoSQLTimeZone::new(1),
            &[1i64, 9, -1],
        );
        let expected_big_int_column = Column::<TestScalar>::BigInt(&[1i64, 9, -1]);
        let big_int_column = cast_column(
            &alloc,
            timestamp_column,
            ColumnType::TimestampTZ(PoSQLTimeUnit::Microsecond, PoSQLTimeZone::new(1)),
            ColumnType::BigInt,
        );
        assert_eq!(big_int_column, expected_big_int_column);
    }

    #[should_panic(expected = "Unable to cast between types BOOLEAN and BINARY")]
    #[test]
    fn we_cannot_cast_column_of_uncastable_type() {
        let alloc = Bump::new();
        let bool_column = Column::<TestScalar>::Boolean(&[true, false, true]);
        cast_column(
            &alloc,
            bool_column,
            ColumnType::Boolean,
            ColumnType::VarBinary,
        );
    }

    #[should_panic(expected = "Casting not supported between BOOLEAN and BINARY")]
    #[test]
    fn we_cannot_cast_bool_column_to_uncastable_type() {
        let alloc = Bump::new();
        let bool_column = &[true, false, true];
        cast_bool_column_to_signed_int_column::<TestScalar>(
            &alloc,
            bool_column,
            ColumnType::VarBinary,
        );
    }

    #[should_panic(expected = "Unsupported cast from int type to BINARY")]
    #[test]
    fn we_cannot_cast_int_slice_to_uncastable_type() {
        let alloc = Bump::new();
        let int_column = &[1];
        cast_int_slice_to_int_column::<TestScalar, _>(&alloc, int_column, ColumnType::VarBinary);
    }

    #[test]
    fn we_can_properly_determine_scaling_factors_for_ints() {
        for from in [
            ColumnType::Uint8,
            ColumnType::TinyInt,
            ColumnType::SmallInt,
            ColumnType::Int,
            ColumnType::BigInt,
            ColumnType::Int128,
        ] {
            let from_precision = Precision::new(from.precision_value().unwrap()).unwrap();
            let forty_prec = Precision::new(40).unwrap();
            let triple = try_get_scaling_factor_with_precision_and_scale(
                from,
                ColumnType::Decimal75(from_precision, 0),
            )
            .unwrap();
            assert_eq!(triple, (U256::ONE, from_precision.value(), 0));

            let triple = try_get_scaling_factor_with_precision_and_scale(
                from,
                ColumnType::Decimal75(forty_prec, 0),
            )
            .unwrap();
            assert_eq!(triple, (U256::ONE, forty_prec.value(), 0));

            let triple = try_get_scaling_factor_with_precision_and_scale(
                from,
                ColumnType::Decimal75(forty_prec, 1),
            )
            .unwrap();
            assert_eq!(triple, (U256::TEN, forty_prec.value(), 1));
        }
    }

    #[test]
    fn we_can_properly_determine_scaling_factors_for_decimals() {
        let twenty_prec = Precision::new(20).unwrap();

        // from_with_negative_scale
        let neg_scale = ColumnType::Decimal75(twenty_prec, -3);

        let triple = try_get_scaling_factor_with_precision_and_scale(
            neg_scale,
            ColumnType::Decimal75(twenty_prec, -3),
        )
        .unwrap();
        assert_eq!(triple, (U256::ONE, twenty_prec.value(), -3));

        let twenty_one_prec = Precision::new(21).unwrap();
        let triple = try_get_scaling_factor_with_precision_and_scale(
            neg_scale,
            ColumnType::Decimal75(twenty_one_prec, -3),
        )
        .unwrap();
        assert_eq!(triple, (U256::ONE, twenty_one_prec.value(), -3));

        let triple = try_get_scaling_factor_with_precision_and_scale(
            neg_scale,
            ColumnType::Decimal75(twenty_one_prec, -2),
        )
        .unwrap();
        assert_eq!(triple, (U256::TEN, twenty_one_prec.value(), -2));

        // from_with_zero_scale
        let zero_scale = ColumnType::Decimal75(twenty_prec, 0);

        let triple = try_get_scaling_factor_with_precision_and_scale(
            zero_scale,
            ColumnType::Decimal75(twenty_prec, 0),
        )
        .unwrap();
        assert_eq!(triple, (U256::ONE, twenty_prec.value(), 0));

        let triple = try_get_scaling_factor_with_precision_and_scale(
            zero_scale,
            ColumnType::Decimal75(twenty_one_prec, 0),
        )
        .unwrap();
        assert_eq!(triple, (U256::ONE, twenty_one_prec.value(), 0));

        let triple = try_get_scaling_factor_with_precision_and_scale(
            zero_scale,
            ColumnType::Decimal75(twenty_one_prec, 1),
        )
        .unwrap();
        assert_eq!(triple, (U256::TEN, twenty_one_prec.value(), 1));

        // from_with_positive_scale
        let pos_scale = ColumnType::Decimal75(twenty_prec, 3);

        let triple = try_get_scaling_factor_with_precision_and_scale(
            pos_scale,
            ColumnType::Decimal75(twenty_prec, 3),
        )
        .unwrap();
        assert_eq!(triple, (U256::ONE, twenty_prec.value(), 3));
        let triple = try_get_scaling_factor_with_precision_and_scale(
            pos_scale,
            ColumnType::Decimal75(twenty_one_prec, 3),
        )
        .unwrap();
        assert_eq!(triple, (U256::ONE, twenty_one_prec.value(), 3));
        let triple = try_get_scaling_factor_with_precision_and_scale(
            pos_scale,
            ColumnType::Decimal75(twenty_one_prec, 4),
        )
        .unwrap();
        assert_eq!(triple, (U256::TEN, twenty_one_prec.value(), 4));
    }

    #[test]
    fn we_can_get_scaling_factor_for_min_and_max_types() {
        let triple = try_get_scaling_factor_with_precision_and_scale(
            ColumnType::Uint8,
            ColumnType::Decimal75(Precision::new(75).unwrap(), 72),
        )
        .unwrap();
        assert_eq!(triple, (U256::TEN.pow(72), 75, 72));
    }

    #[test]
    fn we_cannot_get_scaling_factor_with_uncastable_types() {
        try_get_scaling_factor_with_precision_and_scale(ColumnType::Int128, ColumnType::Boolean)
            .unwrap_err();
    }

    #[test]
    fn we_can_scale_cast_integer_to_decimal() {
        let alloc = Bump::new();

        // tiny int
        let tiny_int_slice = [i8::MAX, i8::MIN, 0];
        let tiny_int_column = Column::<TestScalar>::TinyInt(&tiny_int_slice);
        let prec = Precision::new(5).unwrap();
        let scale = 1i8;
        let scalar_slice = tiny_int_slice
            .map(TestScalar::from)
            .map(|s| s * TestScalar::from(10));
        assert_eq!(
            cast_column_to_decimal_with_scaling(
                &alloc,
                tiny_int_column,
                ColumnType::Decimal75(prec, scale)
            ),
            Column::<TestScalar>::Decimal75(prec, scale, &scalar_slice)
        );

        // uint8
        let uint8_slice = [u8::MAX, u8::MIN];
        let uint8_column = Column::<TestScalar>::Uint8(&uint8_slice);
        let prec = Precision::new(4).unwrap();
        let scale = 1i8;
        let scalar_slice = uint8_slice
            .map(TestScalar::from)
            .map(|s| s * TestScalar::from(10));
        assert_eq!(
            cast_column_to_decimal_with_scaling(
                &alloc,
                uint8_column,
                ColumnType::Decimal75(prec, scale)
            ),
            Column::<TestScalar>::Decimal75(prec, scale, &scalar_slice)
        );

        // small int
        let small_int_slice = [i16::MAX, i16::MIN, 0];
        let small_int_column = Column::<TestScalar>::SmallInt(&small_int_slice);
        let prec = Precision::new(10).unwrap();
        let scale = 0i8;
        let scalar_slice = small_int_slice.map(TestScalar::from);
        assert_eq!(
            cast_column_to_decimal_with_scaling(
                &alloc,
                small_int_column,
                ColumnType::Decimal75(prec, scale)
            ),
            Column::<TestScalar>::Decimal75(prec, scale, &scalar_slice)
        );

        // int
        let int_slice = [i32::MAX, i32::MIN, 0];
        let int_column = Column::<TestScalar>::Int(&int_slice);
        let prec = Precision::new(10).unwrap();
        let scale = 0i8;
        let scalar_slice = int_slice.map(TestScalar::from);
        assert_eq!(
            cast_column_to_decimal_with_scaling(
                &alloc,
                int_column,
                ColumnType::Decimal75(prec, scale)
            ),
            Column::<TestScalar>::Decimal75(prec, scale, &scalar_slice)
        );

        // big int
        let big_int_slice = [i64::MAX, i64::MIN, 0];
        let big_int_column = Column::<TestScalar>::BigInt(&big_int_slice);
        let prec = Precision::new(21).unwrap();
        let scale = 2i8;
        let scalar_slice = big_int_slice
            .map(TestScalar::from)
            .map(|s| s * TestScalar::from(100));
        assert_eq!(
            cast_column_to_decimal_with_scaling(
                &alloc,
                big_int_column,
                ColumnType::Decimal75(prec, scale)
            ),
            Column::<TestScalar>::Decimal75(prec, scale, &scalar_slice)
        );

        // int 128
        let int_128_slice = [i128::MAX, i128::MIN, 0];
        let int_128_column = Column::<TestScalar>::Int128(&int_128_slice);
        let prec = Precision::new(40).unwrap();
        let scale = 1i8;
        let scalar_slice = int_128_slice
            .map(TestScalar::from)
            .map(|s| s * TestScalar::from(10));
        assert_eq!(
            cast_column_to_decimal_with_scaling(
                &alloc,
                int_128_column,
                ColumnType::Decimal75(prec, scale)
            ),
            Column::<TestScalar>::Decimal75(prec, scale, &scalar_slice)
        );
    }

    #[test]
    fn we_can_scale_cast_decimal_to_decimal() {
        let alloc = Bump::new();

        // negative to negative
        let decimal_slice = [TestScalar::ONE, TestScalar::TEN];
        let decimal_column =
            Column::<TestScalar>::Decimal75(Precision::new(2).unwrap(), -2, &decimal_slice);
        let prec = Precision::new(3).unwrap();
        let scale = -1i8;
        let scalar_slice = decimal_slice.map(|s| s * TestScalar::TEN);
        assert_eq!(
            cast_column_to_decimal_with_scaling(
                &alloc,
                decimal_column,
                ColumnType::Decimal75(prec, scale)
            ),
            Column::<TestScalar>::Decimal75(prec, scale, &scalar_slice)
        );

        // negative to positive
        let decimal_slice = [TestScalar::ONE, TestScalar::TEN];
        let decimal_column =
            Column::<TestScalar>::Decimal75(Precision::new(2).unwrap(), -2, &decimal_slice);
        let prec = Precision::new(5).unwrap();
        let scale = 1i8;
        let scalar_slice = decimal_slice.map(|s| s * TestScalar::from(1_000));
        assert_eq!(
            cast_column_to_decimal_with_scaling(
                &alloc,
                decimal_column,
                ColumnType::Decimal75(prec, scale)
            ),
            Column::<TestScalar>::Decimal75(prec, scale, &scalar_slice)
        );

        // positive to positive
        let decimal_slice = [TestScalar::ONE, TestScalar::TEN];
        let decimal_column =
            Column::<TestScalar>::Decimal75(Precision::new(2).unwrap(), 1, &decimal_slice);
        let prec = Precision::new(3).unwrap();
        let scale = 2i8;
        let scalar_slice = decimal_slice.map(|s| s * TestScalar::TEN);
        assert_eq!(
            cast_column_to_decimal_with_scaling(
                &alloc,
                decimal_column,
                ColumnType::Decimal75(prec, scale)
            ),
            Column::<TestScalar>::Decimal75(prec, scale, &scalar_slice)
        );
    }
}
