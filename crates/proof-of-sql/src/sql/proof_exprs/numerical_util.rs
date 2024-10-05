use crate::base::{database::Column, math::decimal::scale_scalar, scalar::Scalar};
use bumpalo::Bump;

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

/// Multiply two columns together.
/// # Panics
/// Panics if: The lengths of `lhs` and `rhs` are not equal.`lhs.scalar_at(i)` or `rhs.scalar_at(i)` returns `None`, which occurs if the column does not have, a scalar at the given index `i`.
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
    let left_scaled_eval = scale_scalar(lhs_eval, max_scale - lhs_scale)
        .expect("scaling factor should not be negative");
    let right_scaled_eval = scale_scalar(rhs_eval, max_scale - rhs_scale)
        .expect("scaling factor should not be negative");
    if is_subtract {
        left_scaled_eval - right_scaled_eval
    } else {
        left_scaled_eval + right_scaled_eval
    }
}
