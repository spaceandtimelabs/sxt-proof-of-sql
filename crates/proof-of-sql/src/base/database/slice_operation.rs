use super::{ColumnOperationError, ColumnOperationResult};
use alloc::{format, vec::Vec};
use core::fmt::Debug;
use num_traits::ops::checked::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};

/// Reverse a binary operator. That is, $a *_{op} b = b * a$.
///
/// With this function we don't need to consider the case of applying
/// a binary operator to a single value and a slice, because we can
/// always use `reverse_op` to reverse the order of the arguments.
pub(crate) fn reverse_op<F, T>(op: F) -> impl Fn(&T, &T) -> T
where
    F: Fn(&T, &T) -> T,
    T: Copy,
{
    move |l, r| op(r, l)
}

/// Function for checked addition with overflow error handling
pub(super) fn try_add<T>(l: &T, r: &T) -> ColumnOperationResult<T>
where
    T: CheckedAdd<Output = T> + Debug,
{
    l.checked_add(r)
        .ok_or(ColumnOperationError::IntegerOverflow {
            error: format!("Overflow in integer addition {l:?} + {r:?}"),
        })
}

/// Function for checked subtraction with overflow error handling
pub(super) fn try_sub<T>(l: &T, r: &T) -> ColumnOperationResult<T>
where
    T: CheckedSub<Output = T> + Debug,
{
    l.checked_sub(r)
        .ok_or(ColumnOperationError::IntegerOverflow {
            error: format!("Overflow in integer subtraction {l:?} - {r:?}"),
        })
}

/// Function for checked multiplication with overflow error handling
pub(super) fn try_mul<T>(l: &T, r: &T) -> ColumnOperationResult<T>
where
    T: CheckedMul<Output = T> + Debug,
{
    l.checked_mul(r)
        .ok_or(ColumnOperationError::IntegerOverflow {
            error: format!("Overflow in integer multiplication {l:?} * {r:?}"),
        })
}

/// Function for checked division with division by zero error handling
pub(super) fn try_div<T>(l: &T, r: &T) -> ColumnOperationResult<T>
where
    T: CheckedDiv<Output = T> + Debug,
{
    l.checked_div(r).ok_or(ColumnOperationError::DivisionByZero)
}

// Generic binary operations on slice and a single value

/// Apply a binary operator to a slice and a single value.
pub(crate) fn slice_lit_binary_op<S, T, U, F>(lhs: &[S], rhs: &T, op: F) -> Vec<U>
where
    F: Fn(&S, &T) -> U,
{
    lhs.iter().map(|l| -> U { op(l, rhs) }).collect::<Vec<_>>()
}

/// Apply a binary operator to a slice and a single value returning results.
pub(crate) fn try_slice_lit_binary_op<S, T, U, F>(
    lhs: &[S],
    rhs: &T,
    op: F,
) -> ColumnOperationResult<Vec<U>>
where
    F: Fn(&S, &T) -> ColumnOperationResult<U>,
{
    lhs.iter()
        .map(|l| op(l, rhs))
        .collect::<ColumnOperationResult<Vec<_>>>()
}

/// Apply a binary operator to a slice and a single value, upcasting the slice.
pub(crate) fn slice_lit_binary_op_left_upcast<S, T, U, F>(lhs: &[S], rhs: &T, op: F) -> Vec<U>
where
    S: Copy + Into<T>,
    T: Copy,
    F: Fn(&T, &T) -> U,
{
    slice_lit_binary_op(lhs, rhs, |l, r| op(&Into::<T>::into(*l), r))
}

/// Apply a binary operator to a slice and a single value with left upcasting returning results.
pub(crate) fn try_slice_lit_binary_op_left_upcast<S, T, U, F>(
    lhs: &[S],
    rhs: &T,
    op: F,
) -> ColumnOperationResult<Vec<U>>
where
    S: Copy + Into<T>,
    T: Copy,
    F: Fn(&T, &T) -> ColumnOperationResult<U>,
{
    try_slice_lit_binary_op(lhs, rhs, |l, r| op(&Into::<T>::into(*l), r))
}

/// Apply a binary operator to a slice and a single value, upcasting the single value.
pub(crate) fn slice_lit_binary_op_right_upcast<S, T, U, F>(lhs: &[S], rhs: &T, op: F) -> Vec<U>
where
    S: Copy,
    T: Copy + Into<S>,
    F: Fn(&S, &S) -> U,
{
    slice_lit_binary_op(lhs, rhs, |l, r| op(l, &Into::<S>::into(*r)))
}

/// Apply a binary operator to a slice and a single value with right upcasting returning results.
pub(crate) fn try_slice_lit_binary_op_right_upcast<S, T, U, F>(
    lhs: &[S],
    rhs: &T,
    op: F,
) -> ColumnOperationResult<Vec<U>>
where
    S: Copy,
    T: Copy + Into<S>,
    F: Fn(&S, &S) -> ColumnOperationResult<U>,
{
    try_slice_lit_binary_op(lhs, rhs, |l, r| op(l, &Into::<S>::into(*r)))
}

// Generic binary operations on slices
/// Apply a binary operator to two slices of the same length.
pub(crate) fn slice_binary_op<S, T, U, F>(lhs: &[S], rhs: &[T], op: F) -> Vec<U>
where
    F: Fn(&S, &T) -> U,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> U { op(l, r) })
        .collect::<Vec<_>>()
}

/// Apply a binary operator to two slices of the same length returning results.
pub(crate) fn try_slice_binary_op<S, T, U, F>(
    lhs: &[S],
    rhs: &[T],
    op: F,
) -> ColumnOperationResult<Vec<U>>
where
    F: Fn(&S, &T) -> ColumnOperationResult<U>,
{
    lhs.iter()
        .zip(rhs.iter())
        .map(|(l, r)| -> ColumnOperationResult<U> { op(l, r) })
        .collect::<ColumnOperationResult<Vec<U>>>()
}

/// Apply a binary operator to two slices of the same length with left upcasting.
pub(crate) fn slice_binary_op_left_upcast<S, T, U, F>(lhs: &[S], rhs: &[T], op: F) -> Vec<U>
where
    S: Copy + Into<T>,
    F: Fn(&T, &T) -> U,
{
    slice_binary_op(lhs, rhs, |l, r| -> U { op(&Into::<T>::into(*l), r) })
}

/// Apply a binary operator to two slices of the same length with left upcasting returning results.
pub(crate) fn try_slice_binary_op_left_upcast<S, T, U, F>(
    lhs: &[S],
    rhs: &[T],
    op: F,
) -> ColumnOperationResult<Vec<U>>
where
    S: Copy + Into<T>,
    F: Fn(&T, &T) -> ColumnOperationResult<U>,
{
    try_slice_binary_op(lhs, rhs, |l, r| -> ColumnOperationResult<U> {
        op(&Into::<T>::into(*l), r)
    })
}

/// Apply a binary operator to two slices of the same length with right upcasting.
pub(crate) fn slice_binary_op_right_upcast<S, T, U, F>(lhs: &[S], rhs: &[T], op: F) -> Vec<U>
where
    T: Copy + Into<S>,
    F: Fn(&S, &S) -> U,
{
    slice_binary_op(lhs, rhs, |l, r| -> U { op(l, &Into::<S>::into(*r)) })
}

/// Apply a binary operator to two slices of the same length with right upcasting returning results.
pub(crate) fn try_slice_binary_op_right_upcast<S, T, U, F>(
    lhs: &[S],
    rhs: &[T],
    op: F,
) -> ColumnOperationResult<Vec<U>>
where
    T: Copy + Into<S>,
    F: Fn(&S, &S) -> ColumnOperationResult<U>,
{
    try_slice_binary_op(lhs, rhs, |l, r| -> ColumnOperationResult<U> {
        op(l, &Into::<S>::into(*r))
    })
}

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
    slice_binary_op(lhs, rhs, |l, r| -> bool { *l && *r })
}

/// Element-wise OR on two boolean slices of the same length.
///
/// We do not check for length equality here.
pub(super) fn slice_or(lhs: &[bool], rhs: &[bool]) -> Vec<bool> {
    slice_binary_op(lhs, rhs, |l, r| -> bool { *l || *r })
}

#[cfg(test)]
mod test {
    use super::*;
    use core::cmp::{PartialEq, PartialOrd};

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
        let actual = slice_binary_op(&lhs, &rhs, PartialEq::eq);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);

        // Try strings
        let lhs = ["Chloe".to_string(), "Margaret".to_string()];
        let rhs = ["Chloe".to_string(), "Chloe".to_string()];
        let actual = slice_binary_op(&lhs, &rhs, PartialEq::eq);
        let expected = vec![true, false];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_eq_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [1_i32, 3, 3];
        let actual = slice_binary_op_left_upcast(&lhs, &rhs, PartialEq::eq);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);
    }

    // <=
    #[test]
    fn we_can_le_slices() {
        let lhs = [1_i32, 2, 3];
        let rhs = [1_i32, 3, 2];
        let actual = slice_binary_op(&lhs, &rhs, PartialOrd::le);
        let expected = vec![true, true, false];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_le_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [1_i64, 3, 2];
        let actual = slice_binary_op_left_upcast(&lhs, &rhs, PartialOrd::le);
        let expected = vec![true, true, false];
        assert_eq!(expected, actual);
    }

    // >=
    #[test]
    fn we_can_ge_slices() {
        let lhs = [1_i128, 2, 3];
        let rhs = [1_i128, 3, 2];
        let actual = slice_binary_op(&lhs, &rhs, PartialOrd::ge);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_can_ge_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [1_i64, 3, 2];
        let actual = slice_binary_op_left_upcast(&lhs, &rhs, PartialOrd::ge);
        let expected = vec![true, false, true];
        assert_eq!(expected, actual);
    }

    // +
    #[test]
    fn we_can_try_add_slices() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_slice_binary_op(&lhs, &rhs, try_add).unwrap();
        let expected = vec![5_i16, -3, 9];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_add_slices_if_overflow() {
        let lhs = [i16::MAX, 1];
        let rhs = [1_i16, 1];
        assert!(matches!(
            try_slice_binary_op(&lhs, &rhs, try_add),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    #[test]
    fn we_can_try_add_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i32, -5, 6];
        let actual = try_slice_binary_op_left_upcast(&lhs, &rhs, try_add).unwrap();
        let expected = vec![5_i32, -3, 9];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_add_slices_with_cast_if_overflow() {
        let lhs = [-1_i16, 1];
        let rhs = [i32::MIN, 1];
        assert!(matches!(
            try_slice_binary_op_left_upcast(&lhs, &rhs, try_add),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    // -
    #[test]
    fn we_can_try_subtract_slices() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_slice_binary_op(&lhs, &rhs, try_sub).unwrap();
        let expected = vec![-3_i16, 7, -3];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_subtract_slices_if_overflow() {
        let lhs = [i128::MIN, 1];
        let rhs = [1_i128, 1];
        assert!(matches!(
            try_slice_binary_op(&lhs, &rhs, try_sub),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    #[test]
    fn we_can_try_subtract_slices_left_upcast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i32, -5, 6];
        let actual = try_slice_binary_op_left_upcast(&lhs, &rhs, try_sub).unwrap();
        let expected = vec![-3_i32, 7, -3];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_subtract_slices_left_upcast_if_overflow() {
        let lhs = [0_i16, 1];
        let rhs = [i32::MIN, 1];
        assert!(matches!(
            try_slice_binary_op_left_upcast(&lhs, &rhs, try_sub),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    #[test]
    fn we_can_try_subtract_slices_right_upcast() {
        let lhs = [1_i32, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_slice_binary_op_right_upcast(&lhs, &rhs, try_sub).unwrap();
        let expected = vec![-3_i32, 7, -3];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_subtract_slices_right_upcast_if_overflow() {
        let lhs = [i32::MIN, 1];
        let rhs = [1_i16, 1];
        assert!(matches!(
            try_slice_binary_op_right_upcast(&lhs, &rhs, try_sub),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    // *
    #[test]
    fn we_can_try_multiply_slices() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i16, -5, 6];
        let actual = try_slice_binary_op(&lhs, &rhs, try_mul).unwrap();
        let expected = vec![4_i16, -10, 18];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_multiply_slices_if_overflow() {
        let lhs = [i32::MAX, 2];
        let rhs = [2, 2];
        assert!(matches!(
            try_slice_binary_op(&lhs, &rhs, try_mul),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    #[test]
    fn we_can_try_multiply_slices_with_cast() {
        let lhs = [1_i16, 2, 3];
        let rhs = [4_i32, -5, 6];
        let actual = try_slice_binary_op_left_upcast(&lhs, &rhs, try_mul).unwrap();
        let expected = vec![4_i32, -10, 18];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_multiply_slices_with_cast_if_overflow() {
        let lhs = [2_i16, 2];
        let rhs = [i32::MAX, 2];
        assert!(matches!(
            try_slice_binary_op_left_upcast(&lhs, &rhs, try_mul),
            Err(ColumnOperationError::IntegerOverflow { .. })
        ));
    }

    // /
    #[test]
    fn we_can_try_divide_slices() {
        let lhs = [5_i16, -5, -7, 9];
        let rhs = [-3_i16, 3, -4, 5];
        let actual = try_slice_binary_op(&lhs, &rhs, try_div).unwrap();
        let expected = vec![-1_i16, -1, 1, 1];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_divide_slices_if_divide_by_zero() {
        let lhs = [1_i32, 2, 3];
        let rhs = [0_i32, -5, 6];
        assert!(matches!(
            try_slice_binary_op(&lhs, &rhs, try_div),
            Err(ColumnOperationError::DivisionByZero)
        ));
    }

    #[test]
    fn we_can_try_divide_slices_left_upcast() {
        let lhs = [5_i16, -4, -9, 9];
        let rhs = [-3_i32, 3, -4, 5];
        let actual = try_slice_binary_op_left_upcast(&lhs, &rhs, try_div).unwrap();
        let expected = vec![-1_i32, -1, 2, 1];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_divide_slices_left_upcast_if_divide_by_zero() {
        let lhs = [1_i16, 2];
        let rhs = [0_i32, 2];
        assert!(matches!(
            try_slice_binary_op_left_upcast(&lhs, &rhs, try_div),
            Err(ColumnOperationError::DivisionByZero)
        ));
    }

    #[test]
    fn we_can_try_divide_slices_right_upcast() {
        let lhs = [15_i128, -82, -7, 9];
        let rhs = [-3_i32, 3, -4, 5];
        let actual = try_slice_binary_op_right_upcast(&lhs, &rhs, try_div).unwrap();
        let expected = vec![-5_i128, -27, 1, 1];
        assert_eq!(expected, actual);
    }

    #[test]
    fn we_cannot_try_divide_slices_right_upcast_if_divide_by_zero() {
        let lhs = [1_i32, 2];
        let rhs = [0_i16, 2];
        assert!(matches!(
            try_slice_binary_op_right_upcast(&lhs, &rhs, try_div),
            Err(ColumnOperationError::DivisionByZero)
        ));
    }
}
