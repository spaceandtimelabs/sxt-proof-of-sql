use crate::base::{polynomial::ArkScalar, scalar::ToArkScalar};
use ark_ff::BigInteger;
use byte_slice_cast::AsByteSlice;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
};
use rand_core::SeedableRng;
use std::collections::HashSet;

#[test]
fn we_can_convert_unsigned_integers_to_ark_scalars() {
    assert_eq!(0_u8.to_ark_scalar(), ArkScalar::from(0_u8));
    assert_eq!(1_u16.to_ark_scalar(), ArkScalar::from(1_u16));
    assert_eq!(1234_u32.to_ark_scalar(), ArkScalar::from(1234_u32));
    assert_eq!(u64::MAX.to_ark_scalar(), ArkScalar::from(u64::MAX));
}

#[test]
fn we_can_convert_signed_positive_integers_to_ark_scalars() {
    assert_eq!(0_i8.to_ark_scalar(), ArkScalar::from(0_u8));
    assert_eq!(1_i16.to_ark_scalar(), ArkScalar::from(1_u16));
    assert_eq!(1234_i32.to_ark_scalar(), ArkScalar::from(1234_u32));
    assert_eq!(i64::MAX.to_ark_scalar(), ArkScalar::from(i64::MAX as u64));
}

#[test]
fn we_can_convert_signed_negative_integers_to_ark_scalars() {
    assert_eq!((-1_i16).to_ark_scalar(), -ArkScalar::from(1_i16 as u16));
    assert_eq!(
        (-1234_i32).to_ark_scalar(),
        -ArkScalar::from(1234_i32 as u32)
    );
    assert_eq!(
        (-i64::MAX).to_ark_scalar(),
        -ArkScalar::from(i64::MAX as u64)
    );
    assert_eq!(i64::MIN.to_ark_scalar(), -ArkScalar::from(1_u64 << 63));
}

#[test]
fn the_empty_string_will_be_mapped_to_the_zero_scalar() {
    assert_eq!("".to_ark_scalar(), ArkScalar::zero());
    assert_eq!(<&str>::default().to_ark_scalar(), ArkScalar::zero());
}

#[test]
fn two_different_strings_map_to_different_scalars() {
    let s = "abc12";
    assert_ne!(s.to_ark_scalar(), ArkScalar::zero());
    assert_ne!(s.to_ark_scalar(), "abc123".to_ark_scalar());
}

#[test]
fn the_empty_buffer_will_be_mapped_to_the_zero_scalar() {
    assert_eq!([].to_ark_scalar(), ArkScalar::zero());
    assert_eq!([].to_ark_scalar(), ArkScalar::zero());
    assert_eq!(Vec::default().to_ark_scalar(), ArkScalar::zero());
    assert_eq!(<Vec<u8>>::default().to_ark_scalar(), ArkScalar::zero());
}

#[test]
fn byte_arrays_with_the_same_content_but_different_types_map_to_different_scalars() {
    let array = [1_u8, 2_u8, 34_u8];
    assert_ne!(array.as_byte_slice().to_ark_scalar(), ArkScalar::zero());
    assert_ne!(
        array.as_byte_slice().to_ark_scalar(),
        [1_u32, 2_u32, 34_u32].as_byte_slice().to_ark_scalar()
    );
}

#[test]
fn strings_of_arbitrary_size_map_to_different_scalars() {
    let mut prev_scalars = HashSet::new();
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, 100);

    for _ in 0..100 {
        let s = dist.sample(&mut rng).to_string()
            + "testing string to scalar"
                .repeat(dist.sample(&mut rng))
                .as_str();
        assert!(prev_scalars.insert(s.as_str().to_ark_scalar()));
    }
}

#[test]
fn byte_arrays_of_arbitrary_size_map_to_different_scalars() {
    let mut prev_scalars = HashSet::new();
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, 100);

    for _ in 0..100 {
        let v = (0..dist.sample(&mut rng))
            .map(|_v| (dist.sample(&mut rng) % 255) as u8)
            .collect::<Vec<u8>>();
        assert!(prev_scalars.insert((v[..]).to_ark_scalar()));
    }
}

#[test]
fn the_string_hash_implementation_uses_the_full_range_of_bits() {
    let max_iters = 20;
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, i32::MAX);

    for i in 0..252 {
        let mut curr_iters = 0;
        let mut bset = HashSet::new();

        loop {
            let s = dist.sample(&mut rng).to_string().as_str().to_ark_scalar();
            let bytes = BigInteger::to_bytes_le(&s.into_bigint()); //Note: this is the only spot that these tests are different from the to_scalar tests.

            let is_ith_bit_set = bytes[i / 8] & (1 << (i % 8)) != 0;

            bset.insert(is_ith_bit_set);

            if bset == HashSet::from([false, true]) {
                break;
            }

            // this guarantees that, if the above test fails,
            // we'll be able to identify it's failing
            assert!(curr_iters <= max_iters);

            curr_iters += 1;
        }
    }
}
