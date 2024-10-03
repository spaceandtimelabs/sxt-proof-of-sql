use alloc::{vec, vec::Vec};
/**
 * Adapted from arkworks
 *
 * See `third_party/license/arkworks.LICENSE`
 */
use core::ops::{Add, AddAssign, Mul, MulAssign, SubAssign};
use core::{cmp::PartialEq, iter::Product};
use num_traits::{Inv, One, Zero};

/// Interpolate a uni-variate degree-`polynomial.len()-1` polynomial and evaluate this
/// polynomial at `x`:
///
/// For any polynomial, `f(x)`, with degree less than or equal to `d`, we have that:
/// `f(x) = sum_{i=0}^{d} (-1)^(d-i) * (f(i) / (i! * (d-i)! * (x-i))) * prod_{i=0}^{d} (x-i)`
/// unless x is one of 0,1,...,d, in which case, f(x) is already known.
#[allow(dead_code)]
pub fn interpolate_uni_poly<F>(polynomial: &[F], x: F) -> F
where
    F: Copy
        + Inv<Output = Option<F>>
        + One
        + Zero
        + AddAssign
        + Mul<Output = F>
        + MulAssign
        + SubAssign
        + PartialEq,
{
    if polynomial.is_empty() {
        return F::zero();
    }
    let degree = polynomial.len() - 1;

    // Construct a vector of factorials, where `factorials[i] = i!`.
    let mut factorials: Vec<F> = Vec::with_capacity(degree + 1);
    let mut factorial = F::one();
    let mut i = F::zero();
    for eval in polynomial {
        factorials.push(factorial);
        if i == x {
            return *eval;
        }
        i += F::one();
        factorial *= i;
    }

    // This will become `sum_{i=0}^{d} (-1)^(d-i) * (f(i) / (i! * (d-i)! * (x-i)))`.
    let mut sum = F::zero();
    // This will become `prod_{i=0}^{d} (x-i)`.
    let mut product = F::one();
    // This will be `x-i`.
    let mut x_minus_i = x;
    for i in 0..=degree {
        // This is `f(i) / (i! * (d-i)! * (x-i))`
        let new_term = polynomial[i]
            * (factorials[i] * factorials[degree - i] * x_minus_i)
                .inv()
                .unwrap(); // This unwrap is safe because we are guarenteed that x-i is not zero, and factorials are never zero.

        // This handles the (-1)^(d-i) sign.
        if (degree - i) % 2 == 0 {
            sum += new_term;
        } else {
            sum -= new_term;
        }
        product *= x_minus_i;
        x_minus_i -= F::one();
    }
    sum * product
}

/// Let `d` be `evals.len() - 1` and let `f` be the polynomial such that `f(i) = evals[i]`.
/// The output of this function is the vector of coefficients of `f`, with the leading coefficient first.
/// That is, `f(x) = evals[j] * x^(d - j)`.
pub fn interpolate_evaluations_to_reverse_coefficients<S>(evals: &[S]) -> Vec<S>
where
    S: Zero
        + Copy
        + From<i32>
        + Mul<Output = S>
        + Add<Output = S>
        + Inv<Output = Option<S>>
        + Product,
{
    let n = evals.len().max(1) - 1;
    evals
        .iter()
        .enumerate()
        .map(|(idx, &eval_i)| {
            let i = idx as i32;
            let mut scaled_lagrange_basis = vec![S::zero(); n + 1];
            // First compute the constant factor of this lagrange basis polynomial:
            scaled_lagrange_basis[0] = (i - n as i32..0)
                .chain(1..=i)
                .map(S::from)
                .product::<S>()
                .inv()
                .unwrap()
                * eval_i;
            // Then multiply by the appropriate linear terms:
            // for j in 0..=n if j != i {
            for neg_j in (-(n as i32)..-i).chain(1 - i..=0).map(S::from) {
                for k in (0..n).rev() {
                    scaled_lagrange_basis[k + 1] =
                        scaled_lagrange_basis[k + 1] + neg_j * scaled_lagrange_basis[k];
                }
            }
            scaled_lagrange_basis
        })
        // Finally, sum up all the resulting polynomials
        .reduce(|mut acc, b| {
            acc.iter_mut().zip(b).for_each(|(a, b)| *a = *a + b);
            acc
        })
        .unwrap_or(vec![])
}
