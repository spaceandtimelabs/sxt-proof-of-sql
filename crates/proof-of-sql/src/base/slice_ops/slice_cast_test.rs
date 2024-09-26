use super::*;
use crate::base::scalar::Curve25519Scalar;
use curve25519_dalek::scalar::Scalar;

#[test]
fn test_slice_map_to_vec() {
    let a: Vec<u32> = vec![1, 2, 3, 4];
    let b: Vec<u64> = vec![1, 2, 3, 4];
    let a: Vec<u64> = slice_cast_with(&a, |&x| u64::from(x));
    assert_eq!(a, b);
}

/// add tests for `slice_cast_with`
#[test]
fn test_slice_cast_with_from_curve25519_scalar_to_dalek_scalar() {
    let a: Vec<Curve25519Scalar> = vec![Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let b: Vec<Scalar> = vec![Scalar::from(1u64), Scalar::from(2u64)];
    let a: Vec<Scalar> = slice_cast_with(&a, std::convert::Into::into);
    assert_eq!(a, b);
}

/// add tests for `slice_cast`
#[test]
fn test_slice_cast_from_curve25519_scalar_to_dalek_scalar() {
    let a: Vec<Curve25519Scalar> = vec![Curve25519Scalar::from(1u64), Curve25519Scalar::from(2u64)];
    let b: Vec<Scalar> = vec![Scalar::from(1u64), Scalar::from(2u64)];
    let a: Vec<Scalar> = slice_cast(&a);
    assert_eq!(a, b);
}

/// random test for `slice_cast_with`
#[test]
fn test_slice_cast_with_random() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let a: Vec<u32> = (0..100).map(|_| rng.gen()).collect();
    let b: Vec<u64> = a.iter().map(|&x| u64::from(x)).collect();
    let a: Vec<u64> = slice_cast_with(&a, |&x| u64::from(x));
    assert_eq!(a, b);
}

/// random test casting from integer to curve25519scalar
#[test]
fn test_slice_cast_with_random_from_integer_to_curve25519scalar() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let a: Vec<u32> = (0..100).map(|_| rng.gen()).collect();
    let b: Vec<Curve25519Scalar> = a.iter().map(|&x| Curve25519Scalar::from(x)).collect();
    let a: Vec<Curve25519Scalar> = slice_cast_with(&a, |&x| Curve25519Scalar::from(x));
    assert_eq!(a, b);
}

/// random test auto casting from integer to curve25519scalar
#[test]
fn test_slice_cast_random_from_integer_to_curve25519scalar() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let a: Vec<u32> = (0..100).map(|_| rng.gen()).collect();
    let b: Vec<Curve25519Scalar> = a.iter().map(|&x| Curve25519Scalar::from(x)).collect();
    let a: Vec<Curve25519Scalar> = slice_cast(&a);
    assert_eq!(a, b);
}

/// Test that mut cast does the same as vec cast
#[test]
fn test_slice_cast_mut() {
    let a: Vec<u32> = vec![1, 2, 3, 4];
    let mut b: Vec<u64> = vec![0, 0, 0, 0];
    slice_cast_mut_with(&a, &mut b, |&x| u64::from(x));
    assert_eq!(b, slice_cast_with(&a, |&x| u64::from(x)));
}

/// random test for `slice_cast_mut_with`
#[test]
fn test_slice_cast_mut_with_random() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let a: Vec<u32> = (0..100).map(|_| rng.gen()).collect();
    let mut b: Vec<u64> = vec![0; 100];
    slice_cast_mut_with(&a, &mut b, |&x| u64::from(x));
    assert_eq!(b, slice_cast_with(&a, |&x| u64::from(x)));
}

/// random test for `slice_cast_mut_with`
#[test]
fn test_slice_cast_mut_random() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let a: Vec<u32> = (0..100).map(|_| rng.gen()).collect();
    let mut b: Vec<Curve25519Scalar> = vec![Curve25519Scalar::default(); 100];
    slice_cast_mut(&a, &mut b);
    assert_eq!(b, slice_cast(&a));
}
