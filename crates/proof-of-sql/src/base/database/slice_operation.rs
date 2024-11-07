use super::{ColumnOperationError, ColumnOperationResult};
use alloc::{format, vec::Vec};
use core::fmt::Debug;
use num_traits::ops::checked::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

// Unary operations

/// Negate a slice of boolean values.
pub(super) fn slice_not(input: &[bool]) -> Vec<bool> {
    input.iter().map(|l| -> bool { !*l }).collect::<Vec<_>>()
}

// Binary operations on slices of the same type

/// Element-wise AND on two boolean slices of the same length.
///
/// We do not check for length equality here.
pub(super) fn slice_and(lhs: &[bool], rhs: &[bool]) -> Vec<bool> {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> bool { *l && *r })
        .collect::<Vec<_>>()
}

/// Element-wise OR on two boolean slices of the same length.
///
/// We do not check for length equality here.
pub(super) fn slice_or(lhs: &[bool], rhs: &[bool]) -> Vec<bool> {
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> bool { *l || *r })
        .collect::<Vec<_>>()
}

/// Try to check whether two slices of the same length are equal element-wise.
///
/// We do not check for length equality here.
pub(super) fn slice_eq<T>(lhs: &[T], rhs: &[T]) -> Vec<bool>
where
    T: PartialEq + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> bool { *l == *r })
        .collect::<Vec<_>>()
}

/// Try to check whether a slice is less than or equal to another element-wise.
///
/// We do not check for length equality here.
pub(super) fn slice_le<T>(lhs: &[T], rhs: &[T]) -> Vec<bool>
where
    T: PartialOrd + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> bool { *l <= *r })
        .collect::<Vec<_>>()
}

/// Try to check whether a slice is greater than or equal to another element-wise.
///
/// We do not check for length equality here.
pub(super) fn slice_ge<T>(lhs: &[T], rhs: &[T]) -> Vec<bool>
where
    T: PartialOrd + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> bool { *l >= *r })
        .collect::<Vec<_>>()
}

/// Try to add two slices of the same length.
///
/// We do not check for length equality here. However, we do check for integer overflow.
pub(super) fn try_add_slices<T>(lhs: &[T], rhs: &[T]) -> ColumnOperationResult<Vec<T>>
where
    T: CheckedAdd<Output = T> + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<T> {
            l.checked_add(r)
                .ok_or(ColumnOperationError::IntegerOverflow {
                    error: format!("Overflow in integer addition {l:?} + {r:?}"),
                })
        })
        .collect::<ColumnOperationResult<Vec<T>>>()
}

/// Subtract one slice from another of the same length.
///
/// We do not check for length equality here. However, we do check for integer overflow.
pub(super) fn try_subtract_slices<T>(lhs: &[T], rhs: &[T]) -> ColumnOperationResult<Vec<T>>
where
    T: CheckedSub<Output = T> + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<T> {
            l.checked_sub(r)
                .ok_or(ColumnOperationError::IntegerOverflow {
                    error: format!("Overflow in integer subtraction {l:?} - {r:?}"),
                })
        })
        .collect::<ColumnOperationResult<Vec<T>>>()
}

/// Multiply two slices of the same length.
///
/// We do not check for length equality here. However, we do check for integer overflow.
pub(super) fn try_multiply_slices<T>(lhs: &[T], rhs: &[T]) -> ColumnOperationResult<Vec<T>>
where
    T: CheckedMul<Output = T> + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<T> {
            l.checked_mul(r)
                .ok_or(ColumnOperationError::IntegerOverflow {
                    error: format!("Overflow in integer multiplication {l:?} * {r:?}"),
                })
        })
        .collect::<ColumnOperationResult<Vec<T>>>()
}

/// Divide one slice by another of the same length.
///
/// We do not check for length equality here. However, we do check for division by 0.
pub(super) fn try_divide_slices<T>(lhs: &[T], rhs: &[T]) -> ColumnOperationResult<Vec<T>>
where
    T: CheckedDiv<Output = T> + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<T> {
            l.checked_div(r).ok_or(ColumnOperationError::DivisionByZero)
        })
        .collect::<ColumnOperationResult<Vec<T>>>()
}

// Casting required for binary operations on different types

/// Check whether two slices of the same length are equal element-wise.
///
/// Note that we cast elements of the left slice to the type of the right slice.
/// Also note that we do not check for length equality here.
pub(super) fn slice_eq_with_casting<SmallerType, LargerType>(
    numbers_of_smaller_type: &[SmallerType],
    numbers_of_larger_type: &[LargerType],
) -> Vec<bool>
where
    SmallerType: Copy + Debug + Into<LargerType>,
    LargerType: PartialEq + Copy + Debug,
{
    numbers_of_smaller_type
        .iter()
        .zip(numbers_of_larger_type.iter())
        .map(|(l, r)| -> bool { Into::<LargerType>::into(*l) == *r })
        .collect::<Vec<_>>()
}

/// Check whether a slice is less than or equal to another element-wise.
///
/// Note that we cast elements of the left slice to the type of the right slice.
/// Also note that we do not check for length equality here.
pub(super) fn slice_le_with_casting<SmallerType, LargerType>(
    numbers_of_smaller_type: &[SmallerType],
    numbers_of_larger_type: &[LargerType],
) -> Vec<bool>
where
    SmallerType: Copy + Debug + Into<LargerType>,
    LargerType: PartialOrd + Copy + Debug,
{
    numbers_of_smaller_type
        .iter()
        .zip(numbers_of_larger_type.iter())
        .map(|(l, r)| -> bool { Into::<LargerType>::into(*l) <= *r })
        .collect::<Vec<_>>()
}

/// Check whether a slice is greater than or equal to another element-wise.
///
/// Note that we cast elements of the left slice to the type of the right slice.
/// Also note that we do not check for length equality here.
pub(super) fn slice_ge_with_casting<SmallerType, LargerType>(
    numbers_of_smaller_type: &[SmallerType],
    numbers_of_larger_type: &[LargerType],
) -> Vec<bool>
where
    SmallerType: Copy + Debug + Into<LargerType>,
    LargerType: PartialOrd + Copy + Debug,
{
    numbers_of_smaller_type
        .iter()
        .zip(numbers_of_larger_type.iter())
        .map(|(l, r)| -> bool { Into::<LargerType>::into(*l) >= *r })
        .collect::<Vec<_>>()
}

/// Add two slices of the same length, casting the left slice to the type of the right slice.
///
/// We do not check for length equality here. However, we do check for integer overflow.
pub(super) fn try_add_slices_with_casting<SmallerType, LargerType>(
    numbers_of_smaller_type: &[SmallerType],
    numbers_of_larger_type: &[LargerType],
) -> ColumnOperationResult<Vec<LargerType>>
where
    SmallerType: Copy + Debug + Into<LargerType>,
    LargerType: CheckedAdd<Output = LargerType> + Copy + Debug,
{
    numbers_of_smaller_type
        .iter()
        .zip(numbers_of_larger_type.iter())
        .map(|(l, r)| -> ColumnOperationResult<LargerType> {
            Into::<LargerType>::into(*l).checked_add(r).ok_or(
                ColumnOperationError::IntegerOverflow {
                    error: format!("Overflow in integer addition {l:?} + {r:?}"),
                },
            )
        })
        .collect()
}

/// Subtract one slice from another of the same length, casting the left slice to the type of the right slice.
///
/// We do not check for length equality here
pub(super) fn try_subtract_slices_left_upcast<SmallerType, LargerType>(
    lhs: &[SmallerType],
    rhs: &[LargerType],
) -> ColumnOperationResult<Vec<LargerType>>
where
    SmallerType: Copy + Debug + Into<LargerType>,
    LargerType: CheckedSub<Output = LargerType> + Copy + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<LargerType> {
            Into::<LargerType>::into(*l).checked_sub(r).ok_or(
                ColumnOperationError::IntegerOverflow {
                    error: format!("Overflow in integer subtraction {l:?} - {r:?}"),
                },
            )
        })
        .collect()
}

/// Subtract one slice from another of the same length, casting the right slice to the type of the left slice.
///
/// We do not check for length equality here
pub(super) fn try_subtract_slices_right_upcast<SmallerType, LargerType>(
    lhs: &[LargerType],
    rhs: &[SmallerType],
) -> ColumnOperationResult<Vec<LargerType>>
where
    SmallerType: Copy + Debug + Into<LargerType>,
    LargerType: CheckedSub<Output = LargerType> + Copy + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<LargerType> {
            l.checked_sub(&Into::<LargerType>::into(*r)).ok_or(
                ColumnOperationError::IntegerOverflow {
                    error: format!("Overflow in integer subtraction {l:?} - {r:?}"),
                },
            )
        })
        .collect()
}

/// Multiply two slices of the same length, casting the left slice to the type of the right slice.
///
/// We do not check for length equality here. However, we do check for integer overflow.
pub(super) fn try_multiply_slices_with_casting<SmallerType, LargerType>(
    numbers_of_smaller_type: &[SmallerType],
    numbers_of_larger_type: &[LargerType],
) -> ColumnOperationResult<Vec<LargerType>>
where
    SmallerType: Copy + Debug + Into<LargerType>,
    LargerType: CheckedMul<Output = LargerType> + Copy + Debug,
{
    numbers_of_smaller_type
        .iter()
        .zip(numbers_of_larger_type.iter())
        .map(|(l, r)| -> ColumnOperationResult<LargerType> {
            Into::<LargerType>::into(*l).checked_mul(r).ok_or(
                ColumnOperationError::IntegerOverflow {
                    error: format!("Overflow in integer multiplication {l:?} * {r:?}"),
                },
            )
        })
        .collect()
}

/// Divide one slice by another of the same length, casting the left slice to the type of the right slice.
///
/// We do not check for length equality here
pub(super) fn try_divide_slices_left_upcast<SmallerType, LargerType>(
    lhs: &[SmallerType],
    rhs: &[LargerType],
) -> ColumnOperationResult<Vec<LargerType>>
where
    SmallerType: Copy + Debug + Into<LargerType>,
    LargerType: CheckedDiv<Output = LargerType> + Copy + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<LargerType> {
            Into::<LargerType>::into(*l)
                .checked_div(r)
                .ok_or(ColumnOperationError::DivisionByZero)
        })
        .collect()
}

/// Divide one slice by another of the same length, casting the right slice to the type of the left slice.
///
/// We do not check for length equality here
pub(super) fn try_divide_slices_right_upcast<SmallerType, LargerType>(
    lhs: &[LargerType],
    rhs: &[SmallerType],
) -> ColumnOperationResult<Vec<LargerType>>
where
    SmallerType: Copy + Debug + Into<LargerType>,
    LargerType: CheckedDiv<Output = LargerType> + Copy + Debug,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<LargerType> {
            l.checked_div(&Into::<LargerType>::into(*r))
                .ok_or(ColumnOperationError::DivisionByZero)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    // NOT
    #[test]
    fn we_can_negate_boolean_slices() {
        let input = [true, false, true];
        let actual = slice_not(&input);
        let expected = vec![false, true, false];
        assert_eq!(expected, actual);
    }

    // AND
    #[test]
    fn we_can_and_boolean_slices() {
        let lhs = [true, false, true, false];
        let rhs = [true, true, false, false];
        let actual = slice_and(&lhs, &rhs);
        let expected = vec![true, false, false, false];
        assert_eq!(expected, actual);
    }

    // OR
    #[test]
    fn we_can_or_boolean_slices() {
        let lhs = [true, false, true, false];
        let rhs = [true, true, false, false];
        let actual = slice_or(&lhs, &rhs);
        let expected = vec![true, true, true, false];
        assert_eq!(expected, actual);
    }

    // =
    #[test]
    fn we_can_eq_slices() {
        let lhs = [1_i16, 2, 3];
        let rhs = [1_i16, 3, 3];
        let actual = slice_eq(&lhs, &rhs);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);

        // Try strings
        let lhs = ["Chloe".to_string(), "Margaret".to_string()];
        let rhs = ["Chloe".to_string(), "Chloe".to_string()];
        let actual = slice_eq(&lhs, &rhs);
        let expected = vec![true, false];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_eq_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [1_i32, 3, 3];
        let actual = slice_eq_with_casting(&lhs, &rhs);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);
    }

    // <=
    #[test]
    fn we_can_le_slices() {
        let lhs = [1_i32, 2, 3];
        let rhs = [1_i32, 3, 2];
        let actual = slice_le(&lhs, &rhs);
        let expected = vec![true, true, false];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_le_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [1_i64, 3, 2];
        let actual = slice_le_with_casting(&lhs, &rhs);
        let expected = vec![true, true, false];
        assert_eq!(expected, actual);
    }

    // >=
    #[test]
    fn we_can_ge_slices() {
        let lhs = [1_i128, 2, 3];
        let rhs = [1_i128, 3, 2];
        let actual = slice_ge(&lhs, &rhs);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_ge_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [1_i64, 3, 2];
        let actual = slice_ge_with_casting(&lhs, &rhs);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);
    }

    // +
    #[test]
    fn we_can_try_add_slices() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_add_slices(&lhs, &rhs).unwrap();
        let expected = vec![5_i16, -3, 9];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_add_slices_if_overflow() {
        let lhs = [i16::MAX, 1];
        let rhs = [1_i16, 1];
        assert!(matches!(
            try_add_slices(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    #[test]
    fn we_can_try_add_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i32, -5, 6];
        let actual = try_add_slices_with_casting(&lhs, &rhs).unwrap();
        let expected = vec![5_i32, -3, 9];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_add_slices_with_cast_if_overflow() {
        let lhs = [-1_i16, 1];
        let rhs = [i32::MIN, 1];
        assert!(matches!(
            try_add_slices_with_casting(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    // -
    #[test]
    fn we_can_try_subtract_slices() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_subtract_slices(&lhs, &rhs).unwrap();
        let expected = vec![-3_i16, 7, -3];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_subtract_slices_if_overflow() {
        let lhs = [i128::MIN, 1];
        let rhs = [1_i128, 1];
        assert!(matches!(
            try_subtract_slices(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    #[test]
    fn we_can_try_subtract_slices_left_upcast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i32, -5, 6];
        let actual = try_subtract_slices_left_upcast(&lhs, &rhs).unwrap();
        let expected = vec![-3_i32, 7, -3];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_subtract_slices_left_upcast_if_overflow() {
        let lhs = [0_i16, 1];
        let rhs = [i32::MIN, 1];
        assert!(matches!(
            try_subtract_slices_left_upcast(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    #[test]
    fn we_can_try_subtract_slices_right_upcast() {
        let lhs = [1_i32, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_subtract_slices_right_upcast(&lhs, &rhs).unwrap();
        let expected = vec![-3_i32, 7, -3];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_subtract_slices_right_upcast_if_overflow() {
        let lhs = [i32::MIN, 1];
        let rhs = [1_i16, 1];
        assert!(matches!(
            try_subtract_slices_right_upcast(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    // *
    #[test]
    fn we_can_try_multiply_slices() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_multiply_slices(&lhs, &rhs).unwrap();
        let expected = vec![4_i16, -10, 18];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_multiply_slices_if_overflow() {
        let lhs = [i32::MAX, 2];
        let rhs = [2, 2];
        assert!(matches!(
            try_multiply_slices(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    #[test]
    fn we_can_try_multiply_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i32, -5, 6];
        let actual = try_multiply_slices_with_casting(&lhs, &rhs).unwrap();
        let expected = vec![4_i32, -10, 18];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_multiply_slices_with_cast_if_overflow() {
        let lhs = [2_i16, 2];
        let rhs = [i32::MAX, 2];
        assert!(matches!(
            try_multiply_slices_with_casting(&lhs, &rhs),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    // /
    #[test]
    fn we_can_try_divide_slices() {
        let lhs = [5_i16, -5, -7, 9];
        let rhs = [-3_i16, 3, -4, 5];
        let actual = try_divide_slices(&lhs, &rhs).unwrap();
        let expected = vec![-1_i16, -1, 1, 1];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_divide_slices_if_divide_by_zero() {
        let lhs = [1_i32, 2, 3];
        let rhs = [0_i32, -5, 6];
        assert!(matches!(
            try_divide_slices(&lhs, &rhs),
            Err(ColumnOperationError::DivisionByZero)
        ));
    }

    #[test]
    fn we_can_try_divide_slices_left_upcast() {
        let lhs = [5_i16, -4, -9, 9];
        let rhs = [-3_i32, 3, -4, 5];
        let actual = try_divide_slices_left_upcast(&lhs, &rhs).unwrap();
        let expected = vec![-1_i32, -1, 2, 1];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_divide_slices_left_upcast_if_divide_by_zero() {
        let lhs = [1_i16, 2];
        let rhs = [0_i32, 2];
        assert!(matches!(
            try_divide_slices_left_upcast(&lhs, &rhs),
            Err(ColumnOperationError::DivisionByZero)
        ));
    }

    #[test]
    fn we_can_try_divide_slices_right_upcast() {
        let lhs = [15_i128, -82, -7, 9];
        let rhs = [-3_i32, 3, -4, 5];
        let actual = try_divide_slices_right_upcast(&lhs, &rhs).unwrap();
        let expected = vec![-5_i128, -27, 1, 1];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_divide_slices_right_upcast_if_divide_by_zero() {
        let lhs = [1_i32, 2];
        let rhs = [0_i16, 2];
        assert!(matches!(
            try_divide_slices_right_upcast(&lhs, &rhs),
            Err(ColumnOperationError::DivisionByZero)
        ));
    }
}
