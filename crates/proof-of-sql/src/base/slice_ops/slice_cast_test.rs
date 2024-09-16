use super::*;
use crate::base::scalar::test_scalar::TestScalar;
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_slice_map_to_vec() {
    let a: Vec<u32> = vec![1, 2, 3, 4];
    let b: Vec<u64> = vec![1, 2, 3, 4];
    let a: Vec<u64> = slice_cast_with(&a, |&x| x as u64);
    assert_eq!(a, b);
}

/// add tests for slice_cast_with
#[test]
fn test_slice_cast_with_from_curve25519_scalar_to_dalek_scalar() {
    let a: Vec<TestScalar> = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
    let b: Vec<Scalar> = vec![Scalar::from(1u64), Scalar::from(2u64)];
    let a: Vec<Scalar> = slice_cast_with(&a, |s| s.into());
    assert_eq!(a, b);
}

/// add tests for slice_cast
#[test]
fn test_slice_cast_from_curve25519_scalar_to_dalek_scalar() {
    let a: Vec<TestScalar> = vec![TestScalar::from(1u64), TestScalar::from(2u64)];
    let b: Vec<Scalar> = vec![Scalar::from(1u64), Scalar::from(2u64)];
    let a: Vec<Scalar> = slice_cast(&a);
    assert_eq!(a, b);
}

/// random test for slice_cast_with
#[test]
fn test_slice_cast_with_random() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let a: Vec<u32> = (0..100).map(|_| rng.gen()).collect();
    let b: Vec<u64> = a.iter().map(|&x| x as u64).collect();
    let a: Vec<u64> = slice_cast_with(&a, |&x| x as u64);
    assert_eq!(a, b);
}

/// random test casting from integer to TestScalar
#[test]
fn test_slice_cast_with_random_from_integer_to_testscalar() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let a: Vec<u32> = (0..100).map(|_| rng.gen()).collect();
    let b: Vec<TestScalar> = a.iter().map(|&x| TestScalar::from(x)).collect();
    let a: Vec<TestScalar> = slice_cast_with(&a, |&x| TestScalar::from(x));
    assert_eq!(a, b);
}

/// random test auto casting from integer to TestScalar
#[test]
fn test_slice_cast_random_from_integer_to_testscalar() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let a: Vec<u32> = (0..100).map(|_| rng.gen()).collect();
    let b: Vec<TestScalar> = a.iter().map(|&x| TestScalar::from(x)).collect();
    let a: Vec<TestScalar> = slice_cast(&a);
    assert_eq!(a, b);
}

/// Test that mut cast does the same as vec cast
#[test]
fn test_slice_cast_mut() {
    let a: Vec<u32> = vec![1, 2, 3, 4];
    let mut b: Vec<u64> = vec![0, 0, 0, 0];
    slice_cast_mut_with(&a, &mut b, |&x| x as u64);
    assert_eq!(b, slice_cast_with(&a, |&x| x as u64));
}

/// random test for slice_cast_mut_with
#[test]
fn test_slice_cast_mut_with_random() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let a: Vec<u32> = (0..100).map(|_| rng.gen()).collect();
    let mut b: Vec<u64> = vec![0; 100];
    slice_cast_mut_with(&a, &mut b, |&x| x as u64);
    assert_eq!(b, slice_cast_with(&a, |&x| x as u64));
}

/// random test for slice_cast_mut_with
#[test]
fn test_slice_cast_mut_random() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let a: Vec<u32> = (0..100).map(|_| rng.gen()).collect();
    let mut b: Vec<TestScalar> = vec![TestScalar::default(); 100];
    slice_cast_mut(&a, &mut b);
    assert_eq!(b, slice_cast(&a));
}
