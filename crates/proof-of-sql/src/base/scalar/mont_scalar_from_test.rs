use crate::base::{
    map::IndexSet,
    scalar::{test_scalar::TestScalar, Scalar},
};
use alloc::{format, string::ToString, vec::Vec};
use byte_slice_cast::AsByteSlice;
use core::cmp::Ordering;
use num_traits::{One, Zero};
use rand::{
    distributions::{Distribution, Uniform},
    rngs::StdRng,
    Rng,
};
use rand_core::SeedableRng;

#[test]
fn the_zero_integer_maps_to_the_zero_scalar() {
    assert_eq!(TestScalar::from(0_u32), TestScalar::zero());
    assert_eq!(TestScalar::from(0_u64), TestScalar::zero());
    assert_eq!(TestScalar::from(0_u128), TestScalar::zero());
    assert_eq!(TestScalar::from(0_i32), TestScalar::zero());
    assert_eq!(TestScalar::from(0_i64), TestScalar::zero());
    assert_eq!(TestScalar::from(0_i128), TestScalar::zero());
}

#[test]
fn bools_map_to_curve25519_scalar_properly() {
    assert_eq!(TestScalar::from(true), TestScalar::one());
    assert_eq!(TestScalar::from(false), TestScalar::zero());
}

#[test]
fn the_one_integer_maps_to_the_zero_scalar() {
    assert_eq!(TestScalar::from(1_u32), TestScalar::one());
    assert_eq!(TestScalar::from(1_u64), TestScalar::one());
    assert_eq!(TestScalar::from(1_u128), TestScalar::one());
    assert_eq!(TestScalar::from(1_i32), TestScalar::one());
    assert_eq!(TestScalar::from(1_i64), TestScalar::one());
    assert_eq!(TestScalar::from(1_i128), TestScalar::one());
}

#[test]
fn the_zero_scalar_is_the_additive_identity() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for _ in 0..1000 {
        let a = TestScalar::from(rng.gen::<i128>());
        let b = TestScalar::from(rng.gen::<i128>());
        assert_eq!(a + b, b + a);
        assert_eq!(a + TestScalar::zero(), a);
        assert_eq!(b + TestScalar::zero(), b);
        assert_eq!(TestScalar::zero() + TestScalar::zero(), TestScalar::zero());
    }
}

#[test]
fn the_one_scalar_is_the_multiplicative_identity() {
    let mut rng = StdRng::seed_from_u64(0u64);
    for _ in 0..1000 {
        let a = TestScalar::from(rng.gen::<i128>());
        let b = TestScalar::from(rng.gen::<i128>());
        assert_eq!(a * b, b * a);
        assert_eq!(a * TestScalar::one(), a);
        assert_eq!(b * TestScalar::one(), b);
        assert_eq!(TestScalar::one() * TestScalar::one(), TestScalar::one());
    }
}

#[test]
fn scalar_comparison_works() {
    let zero = TestScalar::ZERO;
    let one = TestScalar::ONE;
    let two = TestScalar::TWO;
    let max = TestScalar::MAX_SIGNED;
    let min = max + one;
    assert_eq!(max.signed_cmp(&one), Ordering::Greater);
    assert_eq!(one.signed_cmp(&zero), Ordering::Greater);
    assert_eq!(min.signed_cmp(&zero), Ordering::Less);
    assert_eq!((two * max).signed_cmp(&zero), Ordering::Less);
    assert_eq!(two * max + one, zero);
}

#[test]
fn the_empty_string_will_be_mapped_to_the_zero_scalar() {
    assert_eq!(TestScalar::from(""), TestScalar::zero());
    assert_eq!(TestScalar::from(<&str>::default()), TestScalar::zero());
}

#[test]
fn two_different_strings_map_to_different_scalars() {
    let s = "abc12";
    assert_ne!(TestScalar::from(s), TestScalar::zero());
    assert_ne!(TestScalar::from(s), TestScalar::from("abc123"));
}

#[test]
fn the_empty_buffer_will_be_mapped_to_the_zero_scalar() {
    let buf = Vec::<u8>::default();
    assert_eq!(TestScalar::from(&buf[..]), TestScalar::zero());
}

#[test]
fn byte_arrays_with_the_same_content_but_different_types_map_to_different_scalars() {
    let array = [1_u8, 2_u8, 34_u8];
    assert_ne!(TestScalar::from(array.as_byte_slice()), TestScalar::zero());
    assert_ne!(
        TestScalar::from(array.as_byte_slice()),
        TestScalar::from([1_u32, 2_u32, 34_u32].as_byte_slice())
    );
}

#[test]
fn strings_of_arbitrary_size_map_to_different_scalars() {
    let mut prev_scalars = IndexSet::default();
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, 100);

    for i in 0..100 {
        let s = format!(
            "{}_{}_{}",
            dist.sample(&mut rng),
            i,
            "testing string to scalar".repeat(dist.sample(&mut rng))
        );
        assert!(prev_scalars.insert(TestScalar::from(s.as_str())));
    }
}

#[test]
fn byte_arrays_of_arbitrary_size_map_to_different_scalars() {
    let mut prev_scalars = IndexSet::default();
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, 100);

    for _ in 0..100 {
        let v = (0..dist.sample(&mut rng))
            .map(|_v| (dist.sample(&mut rng) % 255) as u8)
            .collect::<Vec<u8>>();
        assert!(prev_scalars.insert(TestScalar::from(&v[..])));
    }
}

#[test]
fn the_string_hash_implementation_uses_the_full_range_of_bits() {
    let max_iters = 20;
    let mut rng = StdRng::from_seed([0u8; 32]);
    let dist = Uniform::new(1, i32::MAX);

    for i in 0..252 {
        let mut curr_iters = 0;
        let mut bset = IndexSet::default();

        loop {
            let s: TestScalar = dist.sample(&mut rng).to_string().as_str().into();
            let bytes = s.to_bytes_le(); //Note: this is the only spot that these tests are different from the to_curve25519_scalar tests.

            let is_ith_bit_set = bytes[i / 8] & (1 << (i % 8)) != 0;

            bset.insert(is_ith_bit_set);

            if bset == IndexSet::from_iter([false, true]) {
                break;
            }

            // this guarantees that, if the above test fails,
            // we'll be able to identify it's failing
            assert!(curr_iters <= max_iters);

            curr_iters += 1;
        }
    }
}
