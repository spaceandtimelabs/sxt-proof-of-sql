/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use curve25519_dalek::scalar::Scalar;

/// compute the factorial(a) = 1 * 2 * ... * a
#[inline]
pub fn scalar_factorial(a: usize) -> Scalar {
    let mut res = Scalar::one();
    for i in 1..=a {
        res *= Scalar::from(i as u64);
    }
    res
}

/// compute the factorial(a) = 1 * 2 * ... * a
#[inline]
pub fn u128_factorial(a: usize) -> u128 {
    let mut res = 1u128;
    for i in 1..=a {
        res *= i as u128;
    }
    res
}

/// compute the factorial(a) = 1 * 2 * ... * a
#[inline]
pub fn u64_factorial(a: usize) -> u64 {
    let mut res = 1u64;
    for i in 1..=a {
        res *= i as u64;
    }
    res
}
