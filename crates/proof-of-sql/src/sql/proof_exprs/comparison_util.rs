use crate::{
    base::{
        database::Column,
        if_rayon,
        math::decimal::{DecimalError, Precision},
        scalar::Scalar,
    },
    sql::parse::{type_check_binary_operation, ConversionError, ConversionResult},
};
use alloc::string::ToString;
use bumpalo::Bump;
use core::cmp;
use proof_of_sql_parser::intermediate_ast::BinaryOperator;
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

fn unchecked_subtract_impl<'a, S: Scalar>(
    alloc: &'a Bump,
    lhs: &[S],
    rhs: &[S],
    table_length: usize,
) -> ConversionResult<&'a [S]> {
    let result = alloc.alloc_slice_fill_default(table_length);
    if_rayon!(result.par_iter_mut(), result.iter_mut())
        .zip(lhs)
        .zip(rhs)
        .for_each(|((a, l), r)| {
            *a = *l - *r;
        });
    Ok(result)
}

/// Scale LHS and RHS to the same scale if at least one of them is decimal
/// and take the difference. This function is used for comparisons.
pub(crate) fn scale_and_subtract<'a, S: Scalar>(
    alloc: &'a Bump,
    lhs: Column<'a, S>,
    rhs: Column<'a, S>,
    lhs_scale: i8,
    rhs_scale: i8,
    is_equal: bool,
) -> ConversionResult<&'a [S]> {
    let lhs_len = lhs.len();
    let rhs_len = rhs.len();
    if lhs_len != rhs_len {
        return Err(ConversionError::DifferentColumnLength {
            len_a: lhs_len,
            len_b: rhs_len,
        });
    }
    let lhs_type = lhs.column_type();
    let rhs_type = rhs.column_type();
    let operator = if is_equal {
        BinaryOperator::Equal
    } else {
        BinaryOperator::LessThanOrEqual
    };
    if !type_check_binary_operation(&lhs_type, &rhs_type, operator) {
        return Err(ConversionError::DataTypeMismatch {
            left_type: lhs_type.to_string(),
            right_type: rhs_type.to_string(),
        });
    }
    let max_scale = cmp::max(lhs_scale, rhs_scale);
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
        let max_precision_value = cmp::max(
            lhs_precision_value + (max_scale - lhs_scale) as u8,
            rhs_precision_value + (max_scale - rhs_scale) as u8,
        );
        // Check if the precision is valid
        let _max_precision = Precision::new(max_precision_value).map_err(|_| {
            ConversionError::DecimalConversionError {
                source: DecimalError::InvalidPrecision {
                    error: max_precision_value.to_string(),
                },
            }
        })?;
    }
    unchecked_subtract_impl(
        alloc,
        &lhs.to_scalar_with_scaling(lhs_upscale),
        &rhs.to_scalar_with_scaling(rhs_upscale),
        lhs_len,
    )
}
