use super::VarInt;
use crate::base::scalar::{Curve25519Scalar, Scalar};
use alloc::{vec, vec::Vec};
use core::{
    fmt::Debug,
    ops::{Add, Neg},
};
use num_traits::{One, Zero};
use rand::Rng;

/**
 * Adapted from integer-encoding-rs
 *
 * See third_party/license/integer-encoding.LICENSE
 */

// -----------------------------------------------------------------------------------------------------
// The following tests are taken directly from integer-encoding-rs with minimal modification
// -----------------------------------------------------------------------------------------------------

#[test]
fn test_required_space() {
    assert_eq!(0_u32.required_space(), 1);
    assert_eq!(1_u32.required_space(), 1);
    assert_eq!(128_u32.required_space(), 2);
    assert_eq!(16384_u32.required_space(), 3);
    assert_eq!(2_097_151_u32.required_space(), 3);
    assert_eq!(2_097_152_u32.required_space(), 4);
}

#[test]
fn test_encode_u64() {
    assert_eq!(0_u32.encode_var_vec(), vec![0b0000_0000]);
    assert_eq!(300_u32.encode_var_vec(), vec![0b1010_1100, 0b0000_0010]);
}

#[test]
fn test_identity_u64() {
    for i in 1_u64..100 {
        assert_eq!(
            u64::decode_var(i.encode_var_vec().as_slice()).unwrap(),
            (i, 1)
        );
    }
    for i in 16400_u64..16500 {
        assert_eq!(
            u64::decode_var(i.encode_var_vec().as_slice()).unwrap(),
            (i, 3)
        );
    }
}

#[test]
fn test_decode_max_u64() {
    let max_vec_encoded = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
    assert_eq!(
        u64::decode_var(max_vec_encoded.as_slice()).unwrap().0,
        u64::MAX
    );
}

#[test]
fn test_encode_i64() {
    assert_eq!(0_i64.encode_var_vec(), 0_u32.encode_var_vec());
    assert_eq!(150_i64.encode_var_vec(), 300_u32.encode_var_vec());
    assert_eq!((-150_i64).encode_var_vec(), 299_u32.encode_var_vec());
    assert_eq!(
        (-2_147_483_648_i64).encode_var_vec(),
        4_294_967_295_u64.encode_var_vec()
    );
    assert_eq!(
        i64::MAX.encode_var_vec(),
        &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
    );
    assert_eq!(
        i64::MIN.encode_var_vec(),
        &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
    );
}

#[test]
fn test_decode_min_i64() {
    let min_vec_encoded = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
    assert_eq!(
        i64::decode_var(min_vec_encoded.as_slice()).unwrap().0,
        i64::MIN
    );
}

#[test]
fn test_decode_max_i64() {
    let max_vec_encoded = vec![0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
    assert_eq!(
        i64::decode_var(max_vec_encoded.as_slice()).unwrap().0,
        i64::MAX
    );
}

#[test]
fn test_encode_i16() {
    assert_eq!(150_i16.encode_var_vec(), 300_u32.encode_var_vec());
    assert_eq!((-150_i16).encode_var_vec(), 299_u32.encode_var_vec());
}

#[test]
fn test_unterminated_varint() {
    let buf = vec![0xff_u8; 12];
    assert!(u64::decode_var(&buf).is_none());
}

#[test]
fn test_unterminated_varint_2() {
    let buf = [0xff, 0xff];
    assert!(u64::decode_var(&buf).is_none());
}

#[test]
fn test_decode_extra_bytes_u64() {
    let mut encoded = 0x12345u64.encode_var_vec();
    assert_eq!(u64::decode_var(&encoded[..]), Some((0x12345, 3)));

    encoded.push(0x99);
    assert_eq!(u64::decode_var(&encoded[..]), Some((0x12345, 3)));

    let encoded = [0xFF, 0xFF, 0xFF];
    assert_eq!(u64::decode_var(&encoded[..]), None);

    // Overflow
    let mut encoded = vec![0xFF; 64];
    encoded.push(0x00);
    assert_eq!(u64::decode_var(&encoded[..]), None);
}

#[test]
fn test_decode_extra_bytes_i64() {
    let mut encoded = (-0x12345i64).encode_var_vec();
    assert_eq!(i64::decode_var(&encoded[..]), Some((-0x12345, 3)));

    encoded.push(0x99);
    assert_eq!(i64::decode_var(&encoded[..]), Some((-0x12345, 3)));

    let encoded = [0xFF, 0xFF, 0xFF];
    assert_eq!(i64::decode_var(&encoded[..]), None);

    // Overflow
    let mut encoded = vec![0xFF; 64];
    encoded.push(0x00);
    assert_eq!(i64::decode_var(&encoded[..]), None);
}

#[test]
fn test_regression_22() {
    let encoded: Vec<u8> = 0x0011_2233_u64.encode_var_vec();
    assert!(i8::decode_var(&encoded).is_none());
}

// ------------------------------------------------------------------------------
// End of tests taken directly from integer-encoding-rs with minimal modification
// ------------------------------------------------------------------------------

// ------------------
// VarInt trait tests
// ------------------

pub(super) fn test_encode_decode<T: VarInt + PartialEq + Debug, const N: usize>(
    val: T,
    encoded: [u8; N],
) {
    let result: &mut [u8] = &mut [0; N];
    assert_eq!(val.required_space(), N);
    assert_eq!(val.encode_var(result), N);
    assert_eq!(result, encoded);
    assert_eq!((val, N), T::decode_var(result).unwrap());
}

fn test_small_unsigned_values_encode_and_decode_properly<
    T: VarInt + Zero + One + Add + PartialEq + Debug,
>() {
    test_encode_decode(T::zero(), [0]);
    test_encode_decode(T::one(), [1]);
    test_encode_decode(T::one() + T::one(), [2]);
    test_encode_decode(T::one() + T::one() + T::one(), [3]);
}

pub(super) fn test_small_signed_values_encode_and_decode_properly<T>(one: T)
where
    T: VarInt + Add<Output = T> + PartialEq + Debug + Neg<Output = T>,
{
    test_encode_decode(one + (-one), [0]);
    test_encode_decode(-one, [1]);
    test_encode_decode(one, [2]);
    test_encode_decode(-(one + one), [3]);
    test_encode_decode(one + one, [4]);
    test_encode_decode(-(one + one + one), [5]);
    test_encode_decode(one + one + one, [6]);
}

pub(super) fn test_encode_and_decode_types_align<Small, Large>(
    align_tests: &[Small],
    too_large_tests: &[Large],
    buffer_size: usize,
) where
    Small: VarInt + Into<Large>,
    Large: VarInt + PartialEq + Debug,
{
    for &val_small in align_tests {
        let val_large: Large = val_small.into();
        let mut result_small = vec![0u8; buffer_size];
        let mut result_large = vec![0u8; buffer_size];
        assert_eq!(
            val_small.encode_var(&mut result_small),
            val_large.encode_var(&mut result_large)
        );
        assert_eq!(result_small, result_large);
        let decode_small = Small::decode_var(&result_small);
        let decode_large = Large::decode_var(&result_small);
        assert_eq!(decode_small.map(|(v, s)| (v.into(), s)), decode_large);
    }

    for too_large in too_large_tests {
        let mut buffer = vec![0u8; buffer_size];
        too_large.encode_var(&mut buffer);
        let decode_small = Small::decode_var(&buffer);
        let decode_large = Large::decode_var(&buffer);
        assert!(decode_small.is_none());
        assert!(decode_large.is_some());
    }
}

#[test]
fn we_can_encode_and_decode_small_i64_values() {
    test_small_signed_values_encode_and_decode_properly::<i64>(1);
}

#[test]
fn we_can_encode_and_decode_small_u64_values() {
    test_small_unsigned_values_encode_and_decode_properly::<u64>();
}

#[test]
fn we_can_encode_and_decode_large_u64_values() {
    test_encode_decode(
        u64::MAX,
        [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
    );
    assert!(
        u64::decode_var(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02]).is_none()
    );
}
#[test]
fn we_can_encode_and_decode_large_i64_values() {
    test_encode_decode(
        i64::MAX,
        [0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
    );
    test_encode_decode(
        i64::MIN,
        [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
    );
    assert!(
        i64::decode_var(&[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02]).is_none()
    );
    assert!(
        i64::decode_var(&[0x81, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02]).is_none()
    );
}

#[test]
fn we_can_encode_and_decode_small_i32_values() {
    test_small_signed_values_encode_and_decode_properly::<i32>(1);
}

#[test]
fn we_can_encode_and_decode_small_u32_values() {
    test_small_unsigned_values_encode_and_decode_properly::<u32>();
}

#[test]
fn we_can_encode_and_decode_large_u32_values() {
    test_encode_decode(u32::MAX, [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    assert!(u32::decode_var(&[0x80, 0x80, 0x80, 0x80, 0x20]).is_none());
}
#[test]
fn we_can_encode_and_decode_large_i32_values() {
    test_encode_decode(i32::MAX, [0xFE, 0xFF, 0xFF, 0xFF, 0x0F]);
    test_encode_decode(i32::MIN, [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    assert!(i32::decode_var(&[0x80, 0x80, 0x80, 0x80, 0x10]).is_none());
    assert!(i32::decode_var(&[0x81, 0x80, 0x80, 0x80, 0x10]).is_none());
}

#[test]
fn we_can_encode_and_decode_i32_and_i64_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<i32, i64>(
        &rng.gen::<[_; 32]>(),
        &[
            i64::from(i32::MAX) + 1,
            i64::from(i32::MIN) - 1,
            i64::from(i32::MAX) * 1000,
            i64::from(i32::MIN) * 1000,
        ],
        100,
    );
}

#[test]
fn we_can_encode_and_decode_u32_and_u64_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<u32, u64>(
        &rng.gen::<[_; 32]>(),
        &[u64::from(u32::MAX) + 1, u64::from(u32::MAX) * 1000],
        100,
    );
}

#[test]
fn we_can_encode_and_decode_large_positive_u128() {
    #[allow(clippy::unusual_byte_groupings)]
    let value: u128 =
        0b110_0010101_1111111_1111111_1111111_1111111_1111111_1111111_1111111_1111111_0011100;
    let expected_result: &[u8] = &[
        0b1001_1100, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111,
        0b1111_1111, 0b1111_1111, 0b1001_0101, 0b0000_0110,
    ];
    let result: &mut [u8] = &mut [0; 11];
    assert_eq!(value.required_space(), 11);
    value.encode_var(result);
    assert_eq!(expected_result, result);
    assert_eq!((value, 11), u128::decode_var(result).unwrap());
}

#[test]
fn we_can_encode_and_decode_large_positive_i128() {
    #[allow(clippy::unusual_byte_groupings)]
    let value: i128 =
        0b110_0010101_1111111_1111111_1111111_1111111_1111111_1111111_1111111_1111111_001110;
    let expected_result: &[u8] = &[
        0b1001_1100, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111,
        0b1111_1111, 0b1111_1111, 0b1001_0101, 0b0000_0110,
    ];
    let result: &mut [u8] = &mut [0; 11];
    assert_eq!(value.required_space(), 11);
    value.encode_var(result);
    assert_eq!(expected_result, result);
    assert_eq!((value, 11), i128::decode_var(result).unwrap());
}

#[test]
fn we_can_encode_and_decode_large_negative_i128() {
    #[allow(clippy::unusual_byte_groupings)]
    let value: i128 =
        -1 - 0b110_0010101_1111111_1111111_1111111_1111111_1111111_1111111_1111111_1111111_001110;
    let expected_result: &[u8] = &[
        0b1001_1101, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111,
        0b1111_1111, 0b1111_1111, 0b1001_0101, 0b0000_0110,
    ];
    let result: &mut [u8] = &mut [0; 11];
    assert_eq!(value.required_space(), 11);
    value.encode_var(result);
    assert_eq!(expected_result, result);
    assert_eq!((value, 11), i128::decode_var(result).unwrap());
}
#[test]
fn we_can_encode_and_decode_small_i128_values() {
    test_small_signed_values_encode_and_decode_properly::<i128>(1);
}

#[test]
fn we_can_encode_and_decode_small_u128_values() {
    test_small_unsigned_values_encode_and_decode_properly::<u128>();
}

#[test]
fn we_can_encode_and_decode_small_curve25519_scalar_values() {
    test_small_signed_values_encode_and_decode_properly::<Curve25519Scalar>(Curve25519Scalar::ONE);
}

#[test]
fn we_can_encode_and_decode_i128_and_curve25519_scalar_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<i128, Curve25519Scalar>(
        &rng.gen::<[_; 32]>(),
        &[
            Curve25519Scalar::from(i128::MAX) + Curve25519Scalar::one(),
            Curve25519Scalar::from(i128::MIN) - Curve25519Scalar::one(),
            Curve25519Scalar::from(i128::MAX) * Curve25519Scalar::from(1000),
            Curve25519Scalar::from(i128::MIN) * Curve25519Scalar::from(1000),
        ],
        100,
    );
}

#[test]
fn we_can_encode_and_decode_i64_and_i128_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<i64, i128>(
        &rng.gen::<[_; 32]>(),
        &[
            i128::from(i64::MAX) + 1,
            i128::from(i64::MIN) - 1,
            i128::from(i64::MAX) * 1000,
            i128::from(i64::MIN) * 1000,
        ],
        100,
    );
}

#[test]
fn we_can_encode_and_decode_u64_and_u128_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<u64, u128>(
        &rng.gen::<[_; 32]>(),
        &[u64::MAX as u128 + 1, u64::MAX as u128 * 1000],
        100,
    );
}

// ----------------------
// End VarInt trait tests
// ----------------------
