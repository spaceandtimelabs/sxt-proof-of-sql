use crate::{
    base::{
        database::Column,
        math::decimal::{scale_scalar, DecimalError, Precision},
        scalar::Scalar,
    },
    sql::parse::{type_check_binary_operation, ConversionError, ConversionResult},
};
use bumpalo::Bump;
use proof_of_sql_parser::intermediate_ast::BinaryOperator;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};

fn unchecked_subtract_impl<'a, S: Scalar>(
    alloc: &'a Bump,
    lhs: &[S],
    rhs: &[S],
    table_length: usize,
) -> ConversionResult<&'a [S]> {
    let res = alloc.alloc_slice_fill_default(table_length);
    res.par_iter_mut()
        .zip(lhs.par_iter().zip(rhs.par_iter()))
        .for_each(|(a, (l, r))| {
            *a = *l - *r;
        });
    Ok(res)
}

/// Scale LHS and RHS to the same scale if at least one of them is decimal
/// and take the difference. This function is used for comparisons.
pub(crate) fn scale_and_subtract<'a, S: Scalar>(
    alloc: &'a Bump,
    lhs: Column<'a, S>,
    rhs: Column<'a, S>,
    is_equal: bool,
) -> ConversionResult<&'a [S]> {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    if lhs_len != rhs_len {
        return Err(ConversionError::DifferentColumnLength(lhs_len, rhs_len));
    }
    let lhs_type = lhs.column_type();
    let rhs_type = rhs.column_type();
    let operator = if is_equal {
        BinaryOperator::Equal
    } else {
        BinaryOperator::LessThanOrEqual
    };
    if !type_check_binary_operation(&lhs_type, &rhs_type, operator) {
        return Err(ConversionError::DataTypeMismatch(
            lhs_type.to_string(),
            rhs_type.to_string(),
        ));
    }
    let lhs_scale = lhs_type.scale().unwrap_or(0);
    let rhs_scale = rhs_type.scale().unwrap_or(0);
    let max_scale = std::cmp::max(lhs_scale, rhs_scale);
    let lhs_upscale = max_scale - lhs_scale;
    let rhs_upscale = max_scale - rhs_scale;
    // Only check precision overflow issues if at least one side is decimal
    if max_scale != 0 {
        let lhs_precision_value = lhs_type
            .precision_value()
            .expect("If scale is set, precision must be set");
        let rhs_precision_value = rhs_type
            .precision_value()
            .expect("If scale is set, precision must be set");
        let max_precision_value = std::cmp::max(
            lhs_precision_value + (max_scale - lhs_scale) as u8,
            rhs_precision_value + (max_scale - rhs_scale) as u8,
        );
        // Check if the precision is valid
        let _max_precision = Precision::new(max_precision_value).map_err(|_| {
            ConversionError::Decimal(DecimalError::InvalidPrecision(max_precision_value))
        })?;
    }
    unchecked_subtract_impl(
        alloc,
        &lhs.to_scalar_with_scaling(lhs_upscale),
        &rhs.to_scalar_with_scaling(rhs_upscale),
        lhs_len,
    )
}

/// The counterpart of `scale_and_subtract` for evaluating decimal expressions.
pub(crate) fn scale_and_subtract_eval<S: Scalar>(
    lhs_eval: S,
    rhs_eval: S,
    lhs_scale: i8,
    rhs_scale: i8,
) -> ConversionResult<S> {
    let max_scale = lhs_scale.max(rhs_scale);
    let scaled_lhs_eval = scale_scalar(lhs_eval, max_scale - lhs_scale)?;
    let scaled_rhs_eval = scale_scalar(rhs_eval, max_scale - rhs_scale)?;
    Ok(scaled_lhs_eval - scaled_rhs_eval)
}
