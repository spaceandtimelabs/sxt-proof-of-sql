use core::mem;
use num_traits::{PrimInt, Unsigned};

pub fn log2_down<T: PrimInt + Unsigned>(x: T) -> usize {
    mem::size_of::<T>() * 8 - (x.leading_zeros() as usize) - 1
}

pub fn is_pow2<T: PrimInt + Unsigned>(x: T) -> bool {
    debug_assert!(x > T::zero());
    x & (x - T::one()) == T::zero()
}

pub fn log2_up<T: PrimInt + Unsigned>(x: T) -> usize {
    let is_not_pow_2 = usize::from(!is_pow2(x));
    log2_down(x) + is_not_pow_2
}

/// Determine if the (unsigned) bytes data is a power of 2.
///
/// The first byte in the array should represent the smallest digit.
/// 0 is treated as a power of 2 instead of panicking.
#[cfg(test)]
pub fn is_pow2_bytes<const N: usize>(data: &[u8; N]) -> bool {
    let mut filter = data.iter().rev().filter(|b| **b != 0);
    if let Some(head) = filter.next() {
        is_pow2(*head) && filter.next().is_none()
    } else {
        true
    }
}

/// Calculate the floored `log_2` of the (unsigned) bytes data.
///
/// The first byte in the array should represent the smallest digit.
/// If the data is 0, returns 0 instead of panicking.
#[cfg(test)]
pub fn log2_down_bytes<const N: usize>(data: &[u8; N]) -> usize {
    let leading_zeros = data.iter().rev().take_while(|b| **b == 0).count();
    if let Some(head_byte) = data.iter().rev().nth(leading_zeros) {
        log2_down(*head_byte) + (N - leading_zeros - 1) * 8
    } else {
        // The data is 0
        0
    }
}

/// Calculate the ceiled `log_2` of the (unsigned) bytes data.
///
/// The first byte in the array should represent the smallest digit.
/// If the data is 0, returns 0 instead of panicking.
#[cfg(test)]
pub fn log2_up_bytes<const N: usize>(data: &[u8; N]) -> usize {
    let is_not_pow_2 = usize::from(!is_pow2_bytes(data));
    log2_down_bytes(data) + is_not_pow_2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log2() {
        assert_eq!(log2_down(1u32), 0);
        assert_eq!(log2_down(2u32), 1);
        assert_eq!(log2_down(3u32), 1);
        assert_eq!(log2_down(4u32), 2);

        assert_eq!(log2_up(1u32), 0);
        assert_eq!(log2_up(2u32), 1);
        assert_eq!(log2_up(3u32), 2);
        assert_eq!(log2_up(4u32), 2);
    }

    #[test]
    fn test_log2_bytes_ceil() {
        // 0-1 edge cases
        assert_eq!(log2_up_bytes(&[0, 0, 0, 0]), 0f32.log2().ceil() as usize);
        assert_eq!(log2_up_bytes(&[1, 0, 0, 0]), 1f32.log2().ceil() as usize);
        assert_eq!(log2_up_bytes(&[0, 1, 0, 0]), 256f32.log2().ceil() as usize);
        assert_eq!(
            log2_up_bytes(&[0, 0, 1, 0]),
            65536f32.log2().ceil() as usize
        );
        assert_eq!(
            log2_up_bytes(&[0, 0, 0, 1]),
            16_777_216_f32.log2().ceil() as usize
        );

        // Bytes are non-trivial powers of 2
        assert_eq!(
            log2_up_bytes(&[128, 0, 0, 0]),
            128f32.log2().ceil() as usize
        );
        assert_eq!(
            log2_up_bytes(&[0, 128, 0, 0]),
            32768f32.log2().ceil() as usize
        );
        assert_eq!(
            log2_up_bytes(&[128, 128, 0, 0]),
            32896f32.log2().ceil() as usize
        );

        // Bytes aren't powers of 2
        assert_eq!(
            log2_up_bytes(&[129, 0, 0, 0]),
            129f32.log2().ceil() as usize
        );
        assert_eq!(
            log2_up_bytes(&[0, 255, 0, 0]),
            65280f32.log2().ceil() as usize
        );
        assert_eq!(
            log2_up_bytes(&[6, 5, 3, 0]),
            197_894_f32.log2().ceil() as usize
        );
        assert_eq!(
            log2_up_bytes(&[255, 255, 255, 255]),
            4_294_967_295_f32.log2().ceil() as usize
        );
    }
}
