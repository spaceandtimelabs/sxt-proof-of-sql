use crate::{
    base::{
        database::{Column, ColumnType, OwnedColumn},
        math::decimal::{scale_scalar, DecimalError, Precision},
        scalar::Scalar,
    },
    sql::{
        parse::{ConversionError, ConversionError::DecimalConversionError, ConversionResult},
        postprocessing::{PostprocessingError, PostprocessingResult},
    },
};
use bigdecimal::{BigDecimal, Zero};
use bumpalo::Bump;
use num_bigint::BigInt;

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
                DecimalConversionError(DecimalError::InvalidPrecision(precision_value.to_string()))
            })
            .and_then(|p| {
                Precision::new(p).map_err(|_| {
                    DecimalConversionError(DecimalError::InvalidPrecision(p.to_string()))
                })
            })?;
        Ok(ColumnType::Decimal75(precision, scale))
    }
}

/// Determine the output type of a multiplication operation if it is possible
/// to multiply the two input types. If the types are not compatible, return
/// an error.
pub(crate) fn try_multiply_column_types(
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
        let left_precision_value = lhs.precision_value().unwrap_or(0);
        let right_precision_value = rhs.precision_value().unwrap_or(0);
        let precision_value = left_precision_value + right_precision_value + 1;
        let precision = Precision::new(precision_value).map_err(|_| {
            DecimalConversionError(DecimalError::InvalidPrecision(format!(
                "Required precision {} is beyond what we can support",
                precision_value
            )))
        })?;
        let left_scale = lhs.scale().unwrap_or(0);
        let right_scale = rhs.scale().unwrap_or(0);
        let scale = left_scale
            .checked_add(right_scale)
            .ok_or(DecimalConversionError(DecimalError::InvalidScale(
                left_scale as i16 + right_scale as i16,
            )))?;
        Ok(ColumnType::Decimal75(precision, scale))
    }
}

/// Determine the output type of a division operation if it is possible
/// to multiply the two input types. If the types are not compatible, return
/// an error.
pub(crate) fn try_divide_column_types(
    lhs: ColumnType,
    rhs: ColumnType,
) -> ConversionResult<ColumnType> {
    if !lhs.is_numeric()
        || !rhs.is_numeric()
        || lhs == ColumnType::Scalar
        || rhs == ColumnType::Scalar
    {
        return Err(ConversionError::DataTypeMismatch(
            lhs.to_string(),
            rhs.to_string(),
        ));
    }

    let left_precision_value = lhs.precision_value().unwrap_or(0) as i16;
    let right_precision_value = rhs.precision_value().unwrap_or(0) as i16;
    let left_scale = lhs.scale().unwrap_or(0) as i16;
    let right_scale = rhs.scale().unwrap_or(0) as i16;
    let raw_scale = (left_scale + right_precision_value + 1_i16).max(6_i16);
    let precision_value: i16 = left_precision_value - left_scale + right_scale + raw_scale;
    let scale = i8::try_from(raw_scale)
        .map_err(|_| DecimalConversionError(DecimalError::InvalidScale(raw_scale)))?;
    let precision = u8::try_from(precision_value)
        .map_err(|_| {
            DecimalConversionError(DecimalError::InvalidPrecision(precision_value.to_string()))
        })
        .and_then(|p| {
            Precision::new(p)
                .map_err(|_| DecimalConversionError(DecimalError::InvalidPrecision(p.to_string())))
        })?;
    Ok(ColumnType::Decimal75(precision, scale))
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

// Add or subtract two owned columns together.
pub(crate) fn add_subtract_owned_columns<S: Scalar>(
    lhs: &OwnedColumn<S>,
    rhs: &OwnedColumn<S>,
    is_subtract: bool,
) -> OwnedColumn<S> {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    assert!(
        lhs_len == rhs_len,
        "lhs and rhs should have the same length"
    );
    let res_type = try_add_subtract_column_types(lhs.column_type(), rhs.column_type())
        .expect("Operation is not supported");
    // Check that lhs and rhs are numeric
    assert!(lhs.column_type().is_numeric());
    assert!(rhs.column_type().is_numeric());
    let lhs_scale = lhs.column_type().scale().unwrap_or(0);
    let rhs_scale = rhs.column_type().scale().unwrap_or(0);
    let max_scale = lhs_scale.max(rhs_scale);
    let lhs_upscale_factor =
        scale_scalar(S::ONE, max_scale - lhs_scale).expect("Invalid scale factor");
    let rhs_upscale_factor =
        scale_scalar(S::ONE, max_scale - rhs_scale).expect("Invalid scale factor");
    let res: Vec<S> = (0..lhs_len)
        .map(|i| {
            if is_subtract {
                lhs.scalar_at(i).unwrap() * lhs_upscale_factor
                    - rhs.scalar_at(i).unwrap() * rhs_upscale_factor
            } else {
                lhs.scalar_at(i).unwrap() * lhs_upscale_factor
                    + rhs.scalar_at(i).unwrap() * rhs_upscale_factor
            }
        })
        .collect();
    OwnedColumn::from_scalars(res, res_type).expect("Invalid column type")
}

/// Multiply two columns together.
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

/// Multiply two owned columns together.
pub(crate) fn multiply_owned_columns<S: Scalar>(
    lhs: &OwnedColumn<S>,
    rhs: &OwnedColumn<S>,
) -> OwnedColumn<S> {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    assert!(
        lhs_len == rhs_len,
        "lhs and rhs should have the same length"
    );
    let res_type = try_multiply_column_types(lhs.column_type(), rhs.column_type())
        .expect("Operation is not supported");
    // Check that lhs and rhs are numeric
    assert!(lhs.column_type().is_numeric());
    assert!(rhs.column_type().is_numeric());
    let res: Vec<S> = (0..lhs_len)
        .map(|i| lhs.scalar_at(i).unwrap() * rhs.scalar_at(i).unwrap())
        .collect();
    OwnedColumn::from_scalars(res, res_type).expect("Invalid column type")
}

/// Divide an owned column by another.
pub(crate) fn try_divide_owned_columns<S: Scalar>(
    lhs: &OwnedColumn<S>,
    rhs: &OwnedColumn<S>,
) -> PostprocessingResult<OwnedColumn<S>> {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    assert!(
        lhs_len == rhs_len,
        "lhs and rhs should have the same length"
    );
    // Check that lhs and rhs are numeric
    assert!(lhs.column_type().is_numeric());
    assert!(rhs.column_type().is_numeric());
    let res_type = try_divide_column_types(lhs.column_type(), rhs.column_type())
        .expect("Operation is not supported");
    let res_scale = res_type.scale().expect("Can't divide non-numeric types");
    let lhs_scale = lhs.column_type().scale().unwrap_or(0);
    let rhs_scale = rhs.column_type().scale().unwrap_or(0);
    let res: Vec<S> = (0..lhs_len)
        .map(|i| -> PostprocessingResult<S> {
            let lhs_bigdecimal = BigDecimal::new(
                Into::<BigInt>::into(lhs.scalar_at(i).unwrap()),
                lhs_scale as i64,
            );
            let rhs_bigdecimal = BigDecimal::new(
                Into::<BigInt>::into(rhs.scalar_at(i).unwrap()),
                rhs_scale as i64,
            );
            if rhs_bigdecimal.is_zero() {
                return Err(PostprocessingError::DivisionByZero);
            }
            let res_bigdecimal = (lhs_bigdecimal / rhs_bigdecimal).round(res_scale as i64);
            let (res_bigint, _) = res_bigdecimal.into_bigint_and_exponent();
            Ok(S::try_from(res_bigint).expect("Division result should fit into scalar"))
        })
        .collect::<PostprocessingResult<Vec<_>>>()?;
    Ok(OwnedColumn::from_scalars(res, res_type).expect("Invalid column type"))
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
