use crate::base::{
    database::{literal_value::ToScalar, Column, ColumnError, ColumnarValue},
    scalar::{Scalar, ScalarExt},
};
use alloc::{format, vec};
use bumpalo::Bump;
use core::cmp::Ordering;
use sqlparser::ast::{DataType, Expr as SqlExpr, ObjectName};

#[allow(clippy::cast_sign_loss)]
/// Add or subtract two literals together.
pub(crate) fn add_subtract_literals<S: Scalar>(
    lhs: &SqlExpr,
    rhs: &SqlExpr,
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

#[allow(
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
#[allow(dead_code)]
pub(crate) fn add_subtract_columnar_values<'a, S: Scalar>(
    lhs: ColumnarValue<'a, S>,
    rhs: ColumnarValue<'a, S>,
    lhs_scale: i8,
    rhs_scale: i8,
    alloc: &'a Bump,
    is_subtract: bool,
) -> Result<ColumnarValue<'a, S>, ColumnError> {
    match (lhs, rhs) {
        (ColumnarValue::Column(lhs), ColumnarValue::Column(rhs)) => {
            Ok(ColumnarValue::Column(Column::Scalar(add_subtract_columns(
                lhs,
                rhs,
                lhs_scale,
                rhs_scale,
                alloc,
                is_subtract,
            ))))
        }
        (ColumnarValue::Literal(lhs), ColumnarValue::Column(rhs)) => {
            Ok(ColumnarValue::Column(Column::Scalar(add_subtract_columns(
                Column::from_literal_with_length(&lhs, rhs.len(), alloc)?,
                rhs,
                lhs_scale,
                rhs_scale,
                alloc,
                is_subtract,
            ))))
        }
        (ColumnarValue::Column(lhs), ColumnarValue::Literal(rhs)) => {
            let rhs_column = Column::from_literal_with_length(&rhs, lhs.len(), alloc)?;
            Ok(ColumnarValue::Column(Column::Scalar(add_subtract_columns(
                lhs,
                rhs_column,
                lhs_scale,
                rhs_scale,
                alloc,
                is_subtract,
            ))))
        }
        (ColumnarValue::Literal(lhs), ColumnarValue::Literal(rhs)) => {
            let result_scalar =
                add_subtract_literals::<S>(&lhs, &rhs, lhs_scale, rhs_scale, is_subtract);
            Ok(ColumnarValue::Literal(SqlExpr::TypedString {
                data_type: DataType::Custom(ObjectName(vec![]), vec![]),
                value: format!("scalar:{result_scalar}"),
            }))
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

#[allow(dead_code)]
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
            let lhs_scalar = (*lhs).to_scalar::<S>();
            let result =
                alloc.alloc_slice_fill_with(rhs.len(), |i| lhs_scalar * rhs.scalar_at(i).unwrap());
            ColumnarValue::Column(Column::Scalar(result))
        }
        (ColumnarValue::Column(lhs), ColumnarValue::Literal(rhs)) => {
            let rhs_scalar = (*rhs).to_scalar();
            let result =
                alloc.alloc_slice_fill_with(lhs.len(), |i| lhs.scalar_at(i).unwrap() * rhs_scalar);
            ColumnarValue::Column(Column::Scalar(result))
        }
        (ColumnarValue::Literal(lhs), ColumnarValue::Literal(rhs)) => {
            let result = (*lhs).to_scalar::<S>() * (*rhs).to_scalar();
            ColumnarValue::Literal(SqlExpr::TypedString {
                data_type: DataType::Custom(ObjectName(vec![]), vec![]),
                value: format!("scalar:{result}"),
            })
        }
    }
}

#[allow(
    clippy::missing_panics_doc,
    reason = "scaling factor is guaranteed to not be negative based on input validation prior to calling this function"
)]
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
