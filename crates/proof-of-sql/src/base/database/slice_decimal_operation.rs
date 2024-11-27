use super::{ColumnOperationError, ColumnOperationResult};
use crate::base::{
    database::{
        column_type_operation::{
            try_add_subtract_column_types, try_divide_column_types, try_multiply_column_types,
        },
        ColumnType,
    },
    math::decimal::Precision,
    scalar::{Scalar, ScalarExt},
};
use alloc::vec::Vec;
use core::{cmp::Ordering, fmt::Debug};
use num_bigint::BigInt;
use num_traits::Zero;
/// Check whether a numerical slice is equal to a decimal one.
///
/// Note that we do not check for length equality here.
/// # Panics
/// This function requires that `lhs` and `rhs` have the same length.
/// This function requires that `left_column_type` and `right_column_type` have the same precision and scale.
pub(super) fn eq_decimal_columns<S, T>(
    lhs: &[T],
    rhs: &[S],
    left_column_type: ColumnType,
    right_column_type: ColumnType,
) -> Vec<bool>
where
    S: Scalar,
    T: Copy + Debug + PartialEq + Zero + Into<S>,
{
    let lhs_scale = left_column_type.scale().expect("Numeric types have scale");
    let rhs_scale = right_column_type.scale().expect("Decimal types have scale");
    let max_scale = lhs_scale.max(rhs_scale);
    // At most one of the scales is non-zero
    if lhs_scale < max_scale {
        // If scale difference is above max decimal precision values
        // are equal if they are both zero and unequal otherwise
        let upscale = max_scale - lhs_scale;
        if i8::try_from(
            right_column_type
                .precision_value()
                .expect("Decimal types have scale"),
        )
        .is_ok_and(|precision| upscale > precision)
        {
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool { l.is_zero() && *r == S::ZERO })
                .collect::<Vec<_>>()
        } else {
            let upscale_factor =
                S::pow10(u8::try_from(upscale).expect("Upscale factor is nonnegative"));
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool { Into::<S>::into(*l) * upscale_factor == *r })
                .collect::<Vec<_>>()
        }
    } else if rhs_scale < max_scale {
        let upscale = max_scale - rhs_scale;
        if i8::try_from(
            left_column_type
                .precision_value()
                .expect("Numeric types have scale"),
        )
        .is_ok_and(|precision| upscale > precision)
        {
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool { l.is_zero() && *r == S::ZERO })
                .collect::<Vec<_>>()
        } else {
            let upscale_factor =
                S::pow10(u8::try_from(upscale).expect("Upscale factor is nonnegative"));
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool { Into::<S>::into(*l) == *r * upscale_factor })
                .collect::<Vec<_>>()
        }
    } else {
        lhs.iter()
            .zip(rhs.iter())
            .map(|(l, r)| -> bool { Into::<S>::into(*l) == *r })
            .collect::<Vec<_>>()
    }
}

/// Check whether a numerical slice is less than or equal to a decimal one.
///
/// Note that we do not check for length equality here.
/// # Panics
/// This function requires that `lhs` and `rhs` have the same length.
/// This function requires that `left_column_type` and `right_column_type` have the same precision and scale.
pub(super) fn le_decimal_columns<S, T>(
    lhs: &[T],
    rhs: &[S],
    left_column_type: ColumnType,
    right_column_type: ColumnType,
) -> Vec<bool>
where
    S: Scalar,
    T: Copy + Debug + Ord + Zero + Into<S>,
{
    let lhs_scale = left_column_type.scale().expect("Numeric types have scale");
    let rhs_scale = right_column_type.scale().expect("Decimal types have scale");
    let max_scale = lhs_scale.max(rhs_scale);
    // At most one of the scales is non-zero
    if lhs_scale < max_scale {
        // If scale difference is above max decimal precision the upscaled
        // always have larger absolute value than the other one as long as it is nonzero
        // Hence a (extremely upscaled) <= b if and only if a < 0 or (a == 0 and b >= 0)
        let upscale = max_scale - lhs_scale;
        if i8::try_from(
            right_column_type
                .precision_value()
                .expect("Decimal types have scale"),
        )
        .is_ok_and(|precision| upscale > precision)
        {
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool {
                    Into::<S>::into(*l).signed_cmp(&S::ZERO) == Ordering::Less
                        || (l.is_zero() && r.signed_cmp(&S::ZERO) != Ordering::Less)
                })
                .collect::<Vec<_>>()
        } else {
            let upscale_factor =
                S::pow10(u8::try_from(upscale).expect("Upscale factor is nonnegative"));
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool {
                    (Into::<S>::into(*l) * upscale_factor).signed_cmp(r) != Ordering::Greater
                })
                .collect::<Vec<_>>()
        }
    } else if rhs_scale < max_scale {
        let upscale = max_scale - rhs_scale;
        if i8::try_from(
            left_column_type
                .precision_value()
                .expect("Numeric types have scale"),
        )
        .is_ok_and(|precision| upscale > precision)
        {
            // Similarly with extreme scaling we have
            // a <= (extremely upscaled) b if and only if a < 0 or (a == 0 and b >= 0)
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool {
                    (Into::<S>::into(*l).signed_cmp(&S::ZERO) != Ordering::Greater && *r == S::ZERO)
                        || r.signed_cmp(&S::ZERO) == Ordering::Greater
                })
                .collect::<Vec<_>>()
        } else {
            let upscale_factor =
                S::pow10(u8::try_from(upscale).expect("Upscale factor is nonnegative"));
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool {
                    Into::<S>::into(*l).signed_cmp(&(*r * upscale_factor)) != Ordering::Greater
                })
                .collect::<Vec<_>>()
        }
    } else {
        lhs.iter()
            .zip(rhs.iter())
            .map(|(l, r)| -> bool { Into::<S>::into(*l).signed_cmp(r) != Ordering::Greater })
            .collect::<Vec<_>>()
    }
}

/// Check whether a numerical slice is greater than or equal to a decimal one.
///
/// Note that we do not check for length equality here.
/// # Panics
/// This function requires that `lhs` and `rhs` have the same length.
/// This function requires that `left_column_type` and `right_column_type` have the same precision and scale.
pub(super) fn ge_decimal_columns<S, T>(
    lhs: &[T],
    rhs: &[S],
    left_column_type: ColumnType,
    right_column_type: ColumnType,
) -> Vec<bool>
where
    S: Scalar,
    T: Copy + Debug + PartialEq + Zero + Into<S>,
{
    let lhs_scale = left_column_type.scale().expect("Numeric types have scale");
    let rhs_scale = right_column_type.scale().expect("Decimal types have scale");
    let max_scale = lhs_scale.max(rhs_scale);
    // At most one of the scales is non-zero
    if lhs_scale < max_scale {
        // If scale difference is above max decimal precision the upscaled
        // always have larger absolute value than the other one as long as it is nonzero
        // Hence a (extremely upscaled) >= b if and only if a > 0 or (a == 0 and b <= 0)
        let upscale = max_scale - lhs_scale;
        if i8::try_from(
            right_column_type
                .precision_value()
                .expect("Decimal types have scale"),
        )
        .is_ok_and(|precision| upscale > precision)
        {
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool {
                    Into::<S>::into(*l).signed_cmp(&S::ZERO) == Ordering::Greater
                        || (l.is_zero() && r.signed_cmp(&S::ZERO) != Ordering::Greater)
                })
                .collect::<Vec<_>>()
        } else {
            let upscale_factor =
                S::pow10(u8::try_from(upscale).expect("Upscale factor is nonnegative"));
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool {
                    (Into::<S>::into(*l) * upscale_factor).signed_cmp(r) != Ordering::Less
                })
                .collect::<Vec<_>>()
        }
    } else if rhs_scale < max_scale {
        let upscale = max_scale - rhs_scale;
        if i8::try_from(
            left_column_type
                .precision_value()
                .expect("Numeric types have scale"),
        )
        .is_ok_and(|precision| upscale > precision)
        {
            // Similarly with extreme scaling we have
            // a >= (extremely upscaled) b if and only if b < 0 or (a >= 0 and b == 0)
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool {
                    (Into::<S>::into(*l).signed_cmp(&S::ZERO) != Ordering::Less && *r == S::ZERO)
                        || r.signed_cmp(&S::ZERO) == Ordering::Less
                })
                .collect::<Vec<_>>()
        } else {
            let upscale_factor =
                S::pow10(u8::try_from(upscale).expect("Upscale factor is nonnegative"));
            lhs.iter()
                .zip(rhs.iter())
                .map(|(l, r)| -> bool {
                    Into::<S>::into(*l).signed_cmp(&(*r * upscale_factor)) != Ordering::Less
                })
                .collect::<Vec<_>>()
        }
    } else {
        lhs.iter()
            .zip(rhs.iter())
            .map(|(l, r)| -> bool { Into::<S>::into(*l).signed_cmp(r) != Ordering::Less })
            .collect::<Vec<_>>()
    }
}

/// Add two numerical slices as decimals.
///
/// We do not check for length equality here
/// # Panics
/// This function requires that `lhs` and `rhs` have the same length.
/// This function requires that `left_column_type` and `right_column_type` have the same precision and scale.
pub(super) fn try_add_decimal_columns<S, T0, T1>(
    lhs: &[T0],
    rhs: &[T1],
    left_column_type: ColumnType,
    right_column_type: ColumnType,
) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
where
    S: Scalar + core::convert::From<T0> + core::convert::From<T1>,
    T0: Copy,
    T1: Copy,
{
    let new_column_type = try_add_subtract_column_types(left_column_type, right_column_type)?;
    let new_precision_value = new_column_type
        .precision_value()
        .expect("numeric columns have precision");
    let new_scale = new_column_type.scale().expect("numeric columns have scale");
    let left_upscale = new_scale
        - left_column_type
            .scale()
            .expect("numeric columns have scale");
    let right_upscale = new_scale
        - right_column_type
            .scale()
            .expect("numeric columns have scale");
    // One of left_scale and right_scale is 0 so we can avoid scaling when unnecessary
    let scalars: Vec<S> = if left_upscale > 0 {
        let upscale_factor =
            S::pow10(u8::try_from(left_upscale).expect("Upscale factor is nonnegative"));
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) * upscale_factor + S::from(*r))
            .collect()
    } else if right_upscale > 0 {
        let upscale_factor =
            S::pow10(u8::try_from(right_upscale).expect("Upscale factor is nonnegative"));
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) + upscale_factor * S::from(*r))
            .collect()
    } else {
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) + S::from(*r))
            .collect()
    };
    Ok((
        Precision::new(new_precision_value).expect("Precision value is valid"),
        new_scale,
        scalars,
    ))
}

/// Subtract one numerical slice from another as decimals.
///
/// We do not check for length equality here
/// # Panics
/// This function requires that `lhs` and `rhs` have the same length.
/// This function requires that `left_column_type` and `right_column_type` have the same precision and scale.
pub(super) fn try_subtract_decimal_columns<S, T0, T1>(
    lhs: &[T0],
    rhs: &[T1],
    left_column_type: ColumnType,
    right_column_type: ColumnType,
) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
where
    S: Scalar + core::convert::From<T0> + core::convert::From<T1>,
    T0: Copy,
    T1: Copy,
{
    let new_column_type = try_add_subtract_column_types(left_column_type, right_column_type)?;
    let new_precision_value = new_column_type
        .precision_value()
        .expect("numeric columns have precision");
    let new_scale = new_column_type.scale().expect("numeric columns have scale");
    let left_upscale = new_scale
        - left_column_type
            .scale()
            .expect("numeric columns have scale");
    let right_upscale = new_scale
        - right_column_type
            .scale()
            .expect("numeric columns have scale");
    // One of left_scale and right_scale is 0 so we can avoid scaling when unnecessary
    let scalars: Vec<S> = if left_upscale > 0 {
        let upscale_factor =
            S::pow10(u8::try_from(left_upscale).expect("Upscale factor is nonnegative"));
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) * upscale_factor - S::from(*r))
            .collect()
    } else if right_upscale > 0 {
        let upscale_factor =
            S::pow10(u8::try_from(right_upscale).expect("Upscale factor is nonnegative"));
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) - upscale_factor * S::from(*r))
            .collect()
    } else {
        lhs.iter()
            .zip(rhs)
            .map(|(l, r)| S::from(*l) - S::from(*r))
            .collect()
    };
    Ok((
        Precision::new(new_precision_value).expect("Precision value is valid"),
        new_scale,
        scalars,
    ))
}

/// Multiply two numerical slices as decimals.
///
/// We do not check for length equality here
/// # Panics
/// This function requires that `lhs` and `rhs` have the same length.
/// This function requires that `left_column_type` and `right_column_type` have the same precision and scale.
pub(super) fn try_multiply_decimal_columns<S, T0, T1>(
    lhs: &[T0],
    rhs: &[T1],
    left_column_type: ColumnType,
    right_column_type: ColumnType,
) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
where
    S: Scalar + core::convert::From<T0> + core::convert::From<T1>,
    T0: Copy,
    T1: Copy,
{
    let new_column_type = try_multiply_column_types(left_column_type, right_column_type)?;
    let new_precision_value = new_column_type
        .precision_value()
        .expect("numeric columns have precision");
    let new_scale = new_column_type.scale().expect("numeric columns have scale");
    let scalars: Vec<S> = lhs
        .iter()
        .zip(rhs)
        .map(|(l, r)| S::from(*l) * S::from(*r))
        .collect();
    Ok((
        Precision::new(new_precision_value).expect("Precision value is valid"),
        new_scale,
        scalars,
    ))
}

/// Divide an owned column by another.
///
/// Notes:
/// 1. We do not check for length equality here.
/// 2. We use floor division for rounding.
/// 3. If division by zero occurs, we return an error.
/// 4. Precision and scale follow T-SQL rules. That is,
///   - `new_scale = max(6, right_precision + left_scale + 1)`
///   - `new_precision = left_precision - left_scale + right_scale + new_scale`
#[allow(clippy::missing_panics_doc)]
pub(crate) fn try_divide_decimal_columns<S, T0, T1>(
    lhs: &[T0],
    rhs: &[T1],
    left_column_type: ColumnType,
    right_column_type: ColumnType,
) -> ColumnOperationResult<(Precision, i8, Vec<S>)>
where
    S: Scalar,
    T0: Copy + Debug + Into<BigInt>,
    T1: Copy + Debug + Into<BigInt>,
{
    let new_column_type = try_divide_column_types(left_column_type, right_column_type)?;
    let new_precision_value = new_column_type
        .precision_value()
        .expect("numeric columns have precision");
    let new_scale = new_column_type.scale().expect("numeric columns have scale");
    let lhs_scale = left_column_type
        .scale()
        .expect("numeric columns have scale");
    let rhs_scale = right_column_type
        .scale()
        .expect("numeric columns have scale");
    let applied_scale = rhs_scale - lhs_scale + new_scale;
    let applied_scale_factor = BigInt::from(10).pow(u32::from(applied_scale.unsigned_abs()));
    let result: Vec<S> = lhs
        .iter()
        .zip(rhs)
        .map(|(l, r)| -> ColumnOperationResult<S> {
            let lhs_bigint = Into::<BigInt>::into(*l);
            let rhs_bigint = Into::<BigInt>::into(*r);
            if rhs_bigint.is_zero() {
                return Err(ColumnOperationError::DivisionByZero);
            }
            let new_bigint = if applied_scale >= 0 {
                lhs_bigint * &applied_scale_factor / rhs_bigint
            } else {
                lhs_bigint / rhs_bigint / &applied_scale_factor
            };
            Ok(S::try_from(new_bigint).expect("Division result should fit into scalar"))
        })
        .collect::<ColumnOperationResult<Vec<_>>>()?;
    Ok((
        Precision::new(new_precision_value).expect("Precision value is valid"),
        new_scale,
        result,
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn we_can_eq_decimal_columns() {
        // lhs is integer and rhs is decimal with nonnegative scale
        let lhs = [1_i8, -2, 3];
        let rhs = [100_i8, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::TinyInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = eq_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, false, false];
        assert_eq!(expected, actual);

        let lhs = [1_i16, -2, 3];
        let rhs = [100_i16, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::SmallInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = eq_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, false, false];
        assert_eq!(expected, actual);

        // lhs is integer and rhs is decimal with negative scale
        let lhs = [400_i64, -82, -200];
        let rhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::BigInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = eq_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = [4_i16, -80, 230]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -8, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 3);
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = eq_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, true, true];
        assert_eq!(expected, actual);

        // lhs is decimal with negative scale and rhs is decimal with nonnegative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, 150_000, -20000]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), 2);
        let actual = eq_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, true, true];
        assert_eq!(expected, actual);

        // lhs is decimal with nonnegative scale and rhs is decimal with negative scale
        let lhs = [71_i64, 150_000, -20000]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), 2);
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = eq_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, true, true];
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with negative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, 150_000, -20000]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -50);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), -46);
        let actual = eq_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, true, true];
        assert_eq!(expected, actual);

        // lhs and rhs are decimals with extreme differences in scale
        let lhs = [4_i16, 0, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, 0, -20000]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -50);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), 26);
        let actual = eq_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, true, false];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_le_decimal_columns() {
        // lhs is integer and rhs is decimal with nonnegative scale
        let lhs = [1_i8, -2, 3];
        let rhs = [100_i8, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::TinyInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = le_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, true, false];
        assert_eq!(expected, actual);

        let lhs = [1_i16, -2, 3];
        let rhs = [100_i16, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::SmallInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = le_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, true, false];
        assert_eq!(expected, actual);

        // lhs is integer and rhs is decimal with negative scale
        let lhs = [400_i64, -82, -199];
        let rhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::BigInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = le_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, true, false];
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = [4_i16, -80, 230]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -8, 22]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 3);
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = le_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, true, false];
        assert_eq!(expected, actual);

        // lhs is decimal with negative scale and rhs is decimal with nonnegative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, 150_000, -30000]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), 2);
        let actual = le_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, true, false];
        assert_eq!(expected, actual);

        // lhs is decimal with nonnegative scale and rhs is decimal with negative scale
        let lhs = [71_i64, 150_000, -19000]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), 2);
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = le_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, true, false];
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with negative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71000_i64, 150_000, -21000]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -50);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), -46);
        let actual = le_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, true, false];
        assert_eq!(expected, actual);

        // lhs and rhs are decimals with extreme differences in scale
        let lhs = [1_i16, 1, 1, 0, 0, 0, -1, -1, -1]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [1_i64, 0, -1, 1, 0, -1, 1, 0, -1]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -50);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), 26);
        let actual = le_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, false, false, true, true, false, true, true, true];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_ge_decimal_columns() {
        // lhs is integer and rhs is decimal with nonnegative scale
        let lhs = [1_i8, -2, 3];
        let rhs = [100_i8, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::TinyInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = ge_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);

        let lhs = [1_i16, -2, 3];
        let rhs = [100_i16, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::SmallInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = ge_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);

        // lhs is integer and rhs is decimal with negative scale
        let lhs = [400_i64, -82, 199];
        let rhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::BigInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = ge_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = [4_i16, -80, 230]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -8, -22]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 3);
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual = ge_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, true, true];
        assert_eq!(expected, actual);

        // lhs is decimal with negative scale and rhs is decimal with nonnegative scale
        let lhs = [-4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, 150_000, -30000]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), 2);
        let actual = ge_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, true, true];
        assert_eq!(expected, actual);

        // lhs is decimal with nonnegative scale and rhs is decimal with negative scale
        let lhs = [71_i64, 150_000, -19000]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), 2);
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let actual = ge_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, true, true];
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with negative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71000_i64, 150_000, -21000]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -50);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), -46);
        let actual = ge_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![false, true, true];
        assert_eq!(expected, actual);

        // lhs and rhs are decimals with extreme differences in scale
        let lhs = [1_i16, 1, 1, 0, 0, 0, -1, -1, -1]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [1_i64, 0, -1, 1, 0, -1, 1, 0, -1]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -50);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), 26);
        let actual = ge_decimal_columns(&lhs, &rhs, left_column_type, right_column_type);
        let expected = vec![true, true, true, false, true, true, false, false, false];
        assert_eq!(expected, actual);
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn we_can_try_add_decimal_columns() {
        // lhs is integer and rhs is decimal with nonnegative scale
        let lhs = [1_i8, -2, 3];
        let rhs = [4_i8, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::TinyInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(104),
            TestScalar::from(-195),
            TestScalar::from(298),
        ];
        let expected = (Precision::new(11).unwrap(), 2, expected_scalars);
        assert_eq!(expected, actual);

        let lhs = [1_i16, -2, 3];
        let rhs = [4_i16, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::SmallInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(104),
            TestScalar::from(-195),
            TestScalar::from(298),
        ];
        let expected = (Precision::new(11).unwrap(), 2, expected_scalars);
        assert_eq!(expected, actual);

        // lhs is decimal with negative scale and rhs is integer
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23];
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::BigInt;
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(471),
            TestScalar::from(1418),
            TestScalar::from(-177),
        ];
        let expected = (Precision::new(20).unwrap(), 0, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(12).unwrap(), 2);
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 3);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(111),
            TestScalar::from(68),
            TestScalar::from(3),
        ];
        let expected = (Precision::new(14).unwrap(), 3, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        // and with result having maximum precision
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(69).unwrap(), -2);
        let right_column_type = ColumnType::Decimal75(Precision::new(50).unwrap(), 3);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(400_071),
            TestScalar::from(1_499_918),
            TestScalar::from(-199_977),
        ];
        let expected = (Precision::new(75).unwrap(), 3, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with negative scale
        // and with result having maximum precision and minimum scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(74).unwrap(), -128);
        let right_column_type = ColumnType::Decimal75(Precision::new(74).unwrap(), -128);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_add_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(75),
            TestScalar::from(-67),
            TestScalar::from(21),
        ];
        let expected = (Precision::new(75).unwrap(), -128, expected_scalars);
        assert_eq!(expected, actual);
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn we_can_try_subtract_decimal_columns() {
        // lhs is integer and rhs is decimal with nonnegative scale
        let lhs = [1_i8, -2, 3];
        let rhs = [4_i8, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::TinyInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(96),
            TestScalar::from(-205),
            TestScalar::from(302),
        ];
        let expected = (Precision::new(11).unwrap(), 2, expected_scalars);
        assert_eq!(expected, actual);

        let lhs = [1_i16, -2, 3];
        let rhs = [4_i16, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::SmallInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(96),
            TestScalar::from(-205),
            TestScalar::from(302),
        ];
        let expected = (Precision::new(11).unwrap(), 2, expected_scalars);
        assert_eq!(expected, actual);

        // lhs is decimal with negative scale and rhs is integer
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23];
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::BigInt;
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(329),
            TestScalar::from(1582),
            TestScalar::from(-223),
        ];
        let expected = (Precision::new(20).unwrap(), 0, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(12).unwrap(), 2);
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 3);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(-31),
            TestScalar::from(232),
            TestScalar::from(-43),
        ];
        let expected = (Precision::new(14).unwrap(), 3, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        // and with result having maximum precision
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(69).unwrap(), -2);
        let right_column_type = ColumnType::Decimal75(Precision::new(50).unwrap(), 3);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(399_929),
            TestScalar::from(1_500_082),
            TestScalar::from(-200_023),
        ];
        let expected = (Precision::new(75).unwrap(), 3, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with negative scale
        // and with result having maximum precision and minimum scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(74).unwrap(), -128);
        let right_column_type = ColumnType::Decimal75(Precision::new(74).unwrap(), -128);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_subtract_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(-67),
            TestScalar::from(97),
            TestScalar::from(-25),
        ];
        let expected = (Precision::new(75).unwrap(), -128, expected_scalars);
        assert_eq!(expected, actual);
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn we_can_try_multiply_decimal_columns() {
        // lhs is integer and rhs is decimal with nonnegative scale
        let lhs = [1_i8, -2, 3];
        let rhs = [4_i8, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::TinyInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(4),
            TestScalar::from(-10),
            TestScalar::from(-6),
        ];
        let expected = (Precision::new(14).unwrap(), 2, expected_scalars);
        assert_eq!(expected, actual);

        let lhs = [1_i16, -2, 3];
        let rhs = [4_i16, 5, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::SmallInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), 2);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(4),
            TestScalar::from(-10),
            TestScalar::from(-6),
        ];
        let expected = (Precision::new(16).unwrap(), 2, expected_scalars);
        assert_eq!(expected, actual);

        // lhs is decimal with negative scale and rhs is integer
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23];
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::BigInt;
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(284),
            TestScalar::from(-1230),
            TestScalar::from(-46),
        ];
        let expected = (Precision::new(30).unwrap(), -2, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        // and with result having maximum precision and maximum scale
        let lhs = [4_i16, 25, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(42).unwrap(), 72);
        let right_column_type = ColumnType::Decimal75(Precision::new(32).unwrap(), 55);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(284),
            TestScalar::from(-2050),
            TestScalar::from(-46),
        ];
        let expected = (Precision::new(75).unwrap(), 127, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        // and with result having maximum precision
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(69).unwrap(), -2);
        let right_column_type = ColumnType::Decimal75(Precision::new(5).unwrap(), 3);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(284),
            TestScalar::from(-1230),
            TestScalar::from(-46),
        ];
        let expected = (Precision::new(75).unwrap(), 1, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with negative scale
        // and with result having maximum precision and minimum scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(34).unwrap(), -64);
        let right_column_type = ColumnType::Decimal75(Precision::new(40).unwrap(), -64);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_multiply_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(284),
            TestScalar::from(-1230),
            TestScalar::from(-46),
        ];
        let expected = (Precision::new(75).unwrap(), -128, expected_scalars);
        assert_eq!(expected, actual);
    }

    #[allow(clippy::too_many_lines)]
    #[test]
    fn we_can_try_divide_decimal_columns() {
        // lhs is integer and rhs is decimal with nonnegative scale
        let lhs = [0_i8, 2, 3];
        let rhs = [4_i8, 5, 2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::TinyInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(3).unwrap(), 2);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_divide_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(0_i64),
            TestScalar::from(40_000_000_i64),
            TestScalar::from(150_000_000_i64),
        ];
        let expected = (Precision::new(11).unwrap(), 6, expected_scalars);
        assert_eq!(expected, actual);

        let lhs = [0_i16, 2, 3];
        let rhs = [4_i16, 5, 2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::SmallInt;
        let right_column_type = ColumnType::Decimal75(Precision::new(3).unwrap(), 2);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_divide_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(0_i64),
            TestScalar::from(40_000_000_i64),
            TestScalar::from(150_000_000_i64),
        ];
        let expected = (Precision::new(13).unwrap(), 6, expected_scalars);
        assert_eq!(expected, actual);

        // lhs is decimal with negative scale and rhs is integer
        let lhs = [4_i8, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23];
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::TinyInt;
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_divide_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(5_633_802),
            TestScalar::from(-18_292_682),
            TestScalar::from(-8_695_652),
        ];
        let expected = (Precision::new(18).unwrap(), 6, expected_scalars);
        assert_eq!(expected, actual);

        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23];
        let left_column_type = ColumnType::Decimal75(Precision::new(10).unwrap(), -2);
        let right_column_type = ColumnType::SmallInt;
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_divide_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(5_633_802),
            TestScalar::from(-18_292_682),
            TestScalar::from(-8_695_652),
        ];
        let expected = (Precision::new(18).unwrap(), 6, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with nonnegative scale
        let lhs = [4_i16, 2, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [3_i64, -5, 7]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(4).unwrap(), 2);
        let right_column_type = ColumnType::Decimal75(Precision::new(3).unwrap(), 2);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_divide_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(1_333_333),
            TestScalar::from(-400_000),
            TestScalar::from(-285_714),
        ];
        let expected = (Precision::new(10).unwrap(), 6, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals one of which has negative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(2).unwrap(), -2);
        let right_column_type = ColumnType::Decimal75(Precision::new(3).unwrap(), 3);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_divide_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(5_633_802_816_i128),
            TestScalar::from(-18_292_682_926_i128),
            TestScalar::from(-8_695_652_173_i128),
        ];
        let expected = (Precision::new(13).unwrap(), 6, expected_scalars);
        assert_eq!(expected, actual);

        // lhs and rhs are both decimals with negative scale
        let lhs = [4_i16, 15, -2]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let rhs = [71_i64, -82, 23]
            .into_iter()
            .map(TestScalar::from)
            .collect::<Vec<_>>();
        let left_column_type = ColumnType::Decimal75(Precision::new(2).unwrap(), -3);
        let right_column_type = ColumnType::Decimal75(Precision::new(3).unwrap(), -2);
        let actual: (Precision, i8, Vec<TestScalar>) =
            try_divide_decimal_columns(&lhs, &rhs, left_column_type, right_column_type).unwrap();
        let expected_scalars = vec![
            TestScalar::from(563_380),
            TestScalar::from(-1_829_268),
            TestScalar::from(-869_565),
        ];
        let expected = (Precision::new(9).unwrap(), 6, expected_scalars);
        assert_eq!(expected, actual);
    }
}
