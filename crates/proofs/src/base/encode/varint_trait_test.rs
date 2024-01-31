use super::VarInt;
use core::{
    fmt::Debug,
    ops::{Add, Neg},
};
use num_traits::{One, Zero};
use rand::{distributions::Standard, prelude::Distribution, Rng};

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
    assert_eq!(2097151_u32.required_space(), 3);
    assert_eq!(2097152_u32.required_space(), 4);
}

#[test]
fn test_encode_u64() {
    assert_eq!(0_u32.encode_var_vec(), vec![0b00000000]);
    assert_eq!(300_u32.encode_var_vec(), vec![0b10101100, 0b00000010]);
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
        u64::max_value()
    );
}

#[test]
fn test_encode_i64() {
    assert_eq!(0_i64.encode_var_vec(), 0_u32.encode_var_vec());
    assert_eq!(150_i64.encode_var_vec(), 300_u32.encode_var_vec());
    assert_eq!((-150_i64).encode_var_vec(), 299_u32.encode_var_vec());
    assert_eq!(
        (-2147483648_i64).encode_var_vec(),
        4294967295_u64.encode_var_vec()
    );
    assert_eq!(
        i64::max_value().encode_var_vec(),
        &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
    );
    assert_eq!(
        i64::min_value().encode_var_vec(),
        &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
    );
}

#[test]
fn test_decode_min_i64() {
    let min_vec_encoded = vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
    assert_eq!(
        i64::decode_var(min_vec_encoded.as_slice()).unwrap().0,
        i64::min_value()
    );
}

#[test]
fn test_decode_max_i64() {
    let max_vec_encoded = vec![0xFE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
    assert_eq!(
        i64::decode_var(max_vec_encoded.as_slice()).unwrap().0,
        i64::max_value()
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
    let encoded: Vec<u8> = 0x112233_u64.encode_var_vec();
    assert!(i8::decode_var(&encoded).is_none());
}

// ------------------------------------------------------------------------------
// End of tests taken directly from integer-encoding-rs with minimal modification
// ------------------------------------------------------------------------------

// ------------------
// VarInt trait tests
// ------------------

fn test_encode_decode<T: VarInt + PartialEq + Debug, const N: usize>(val: T, encoded: [u8; N]) {
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

fn test_small_signed_values_encode_and_decode_properly<
    T: VarInt + Zero + One + Add + PartialEq + Debug + Neg<Output = T>,
>() {
    test_encode_decode(T::zero(), [0]);
    test_encode_decode(-T::one(), [1]);
    test_encode_decode(T::one(), [2]);
    test_encode_decode(-(T::one() + T::one()), [3]);
    test_encode_decode(T::one() + T::one(), [4]);
    test_encode_decode(-(T::one() + T::one() + T::one()), [5]);
    test_encode_decode(T::one() + T::one() + T::one(), [6]);
}

fn test_encode_and_decode_types_align<Small, Large>(
    rng: &mut impl Rng,
    too_large_tests: &[Large],
    buffer_size: usize,
) where
    Standard: Distribution<Small>,
    Small: VarInt + Into<Large>,
    Large: VarInt + PartialEq + Debug,
{
    for _ in 0..32 {
        let val_small: Small = rng.gen();
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
    test_small_signed_values_encode_and_decode_properly::<i64>();
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
    test_small_signed_values_encode_and_decode_properly::<i32>();
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
        &mut rng,
        &[
            i32::MAX as i64 + 1,
            i32::MIN as i64 - 1,
            i32::MAX as i64 * 1000,
            i32::MIN as i64 * 1000,
        ],
        100,
    );
}

#[test]
fn we_can_encode_and_decode_u32_and_u64_the_same() {
    let mut rng = rand::thread_rng();
    test_encode_and_decode_types_align::<u32, u64>(
        &mut rng,
        &[u32::MAX as u64 + 1, u32::MAX as u64 * 1000],
        100,
    );
}

// ----------------------
// End VarInt trait tests
// ----------------------
