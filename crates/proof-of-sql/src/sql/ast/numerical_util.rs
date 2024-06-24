use crate::{
    base::{
        database::{Column, ColumnType},
        math::decimal::{scale_scalar, DecimalError, Precision},
        scalar::Scalar,
    },
    sql::parse::{ConversionError, ConversionResult},
};
use bumpalo::Bump;

// For decimal type manipulation please refer to
// https://learn.microsoft.com/en-us/sql/t-sql/data-types/precision-scale-and-length-transact-sql?view=sql-server-ver16

/// Determine the output type of an add or subtract operation if it is possible
/// to add or subtract the two input types. If the types are not compatible, return
/// an error.
pub(crate) fn try_add_subtract_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ConversionResult<ColumnType> {
    if !lhs.is_numeric() || !rhs.is_numeric() {
        return Err(ConversionError::DataTypeMismatch(
            lhs.to_string(),
            rhs.to_string(),
        ));
    }
    if lhs.is_integer() && rhs.is_integer() {
        // We can unwrap here because we know that both types are integers
        return Ok(lhs.max_integer_type(&rhs).unwrap());
    }
    if lhs == ColumnType::Scalar || rhs == ColumnType::Scalar {
        Ok(ColumnType::Scalar)
    } else {
        let left_precision_value = lhs.precision_value().unwrap_or(0) as i16;
        let right_precision_value = rhs.precision_value().unwrap_or(0) as i16;
        let left_scale = lhs.scale().unwrap_or(0);
        let right_scale = rhs.scale().unwrap_or(0);
        let scale = left_scale.max(right_scale);
        let precision_value: i16 = scale as i16
            + (left_precision_value - left_scale as i16)
                .max(right_precision_value - right_scale as i16)
            + 1_i16;
        let precision = u8::try_from(precision_value)
            .map_err(|_| {
                ConversionError::DecimalConversion(DecimalError::InvalidPrecision(
                    precision_value.to_string(),
                ))
            })
            .and_then(|p| {
                Precision::new(p).map_err(|_| {
                    ConversionError::DecimalConversion(DecimalError::InvalidPrecision(
                        p.to_string(),
                    ))
                })
            })?;
        Ok(ColumnType::Decimal75(precision, scale))
    }
}

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
    let _res: &mut [S] = alloc.alloc_slice_fill_default(lhs_len);
    let max_scale = lhs_scale.max(rhs_scale);
    let lhs_scalar = lhs.to_scalar_with_scaling(max_scale - lhs_scale);
    let rhs_scalar = rhs.to_scalar_with_scaling(max_scale - rhs_scale);
    let res = alloc.alloc_slice_fill_with(lhs_len, |i| {
        if is_subtract {
            lhs_scalar[i] - rhs_scalar[i]
        } else {
            lhs_scalar[i] + rhs_scalar[i]
        }
    });
    res
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
    let scaled_lhs_eval = scale_scalar(lhs_eval, max_scale - lhs_scale)
        .expect("scaling factor should not be negative");
    let scaled_rhs_eval = scale_scalar(rhs_eval, max_scale - rhs_scale)
        .expect("scaling factor should not be negative");
    if is_subtract {
        scaled_lhs_eval - scaled_rhs_eval
    } else {
        scaled_lhs_eval + scaled_rhs_eval
    }
}
