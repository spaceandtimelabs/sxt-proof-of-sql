use crate::base::polynomial::ArkScalar;
use byte_slice_cast::AsByteSlice;
use num_traits::{One, Zero};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng,
};
use rand_core::SeedableRng;
use std::collections::HashSet;

#[test]
fn the_zero_integer_maps_to_the_zero_scalar() {
    assert_eq!(ArkScalar::from(0_u32), ArkScalar::zero());
    assert_eq!(ArkScalar::from(0_u64), ArkScalar::zero());
    assert_eq!(ArkScalar::from(0_u128), ArkScalar::zero());
    assert_eq!(ArkScalar::from(0_i32), ArkScalar::zero());
    assert_eq!(ArkScalar::from(0_i64), ArkScalar::zero());
    assert_eq!(ArkScalar::from(0_i128), ArkScalar::zero());
}

#[test]
fn bools_map_to_ark_scalar_properly() {
    assert_eq!(ArkScalar::from(true), ArkScalar::one());
    assert_eq!(ArkScalar::from(false), ArkScalar::zero());
}

#[test]
fn the_one_integer_maps_to_the_zero_scalar() {
    assert_eq!(ArkScalar::from(1_u32), ArkScalar::one());
    assert_eq!(ArkScalar::from(1_u64), ArkScalar::one());
    assert_eq!(ArkScalar::from(1_u128), ArkScalar::one());
    assert_eq!(ArkScalar::from(1_i32), ArkScalar::one());
    assert_eq!(ArkScalar::from(1_i64), ArkScalar::one());
    assert_eq!(ArkScalar::from(1_i128), ArkScalar::one());
}

#[test]
fn the_zero_scalar_is_the_additive_identity() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for _ in 0..1000 {
        let a = ArkScalar::from(rng.gen::<i128>());
        let b = ArkScalar::from(rng.gen::<i128>());
        assert_eq!(a + b, b + a);
        assert_eq!(a + ArkScalar::zero(), a);
        assert_eq!(b + ArkScalar::zero(), b);
        assert_eq!(ArkScalar::zero() + ArkScalar::zero(), ArkScalar::zero());
    }
}

#[test]
fn the_one_scalar_is_the_multiplicative_identity() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for _ in 0..1000 {
        let a = ArkScalar::from(rng.gen::<i128>());
        let b = ArkScalar::from(rng.gen::<i128>());
        assert_eq!(a * b, b * a);
        assert_eq!(a * ArkScalar::one(), a);
        assert_eq!(b * ArkScalar::one(), b);
        assert_eq!(ArkScalar::one() * ArkScalar::one(), ArkScalar::one());
    }
}

#[test]
fn the_empty_string_will_be_mapped_to_the_zero_scalar() {
    assert_eq!(ArkScalar::from(""), ArkScalar::zero());
    assert_eq!(ArkScalar::from(<&str>::default()), ArkScalar::zero());
}

#[test]
fn two_different_strings_map_to_different_scalars() {
    let s = "abc12";
    assert_ne!(ArkScalar::from(s), ArkScalar::zero());
    assert_ne!(ArkScalar::from(s), ArkScalar::from("abc123"));
}

#[test]
fn the_empty_buffer_will_be_mapped_to_the_zero_scalar() {
    let buf = Vec::<u8>::default();
    assert_eq!(ArkScalar::from(&buf[..]), ArkScalar::zero());
}

#[test]
fn byte_arrays_with_the_same_content_but_different_types_map_to_different_scalars() {
    let array = [1_u8, 2_u8, 34_u8];
    assert_ne!(ArkScalar::from(array.as_byte_slice()), ArkScalar::zero());
    assert_ne!(
        ArkScalar::from(array.as_byte_slice()),
        ArkScalar::from([1_u32, 2_u32, 34_u32].as_byte_slice())
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
        assert!(prev_scalars.insert(ArkScalar::from(s.as_str())));
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
        assert!(prev_scalars.insert(ArkScalar::from(&v[..])));
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
            let s: ArkScalar = dist.sample(&mut rng).to_string().as_str().into();
            let bytes = s.to_bytes_le(); //Note: this is the only spot that these tests are different from the to_ark_scalar tests.

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
