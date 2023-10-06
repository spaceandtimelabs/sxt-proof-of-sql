//! These functions are adapted from arkworks. https://github.com/arkworks-rs/algebra/blob/ab13aa09ae3c11cde0224028dee7b878bbcf9246/ff/src/fields/mod.rs#L347-L410
//! See third_party/license/arkworks.LICENSE
//!
//! They differ in that they don't rely on the `Field` trait, but instead use `core::ops` and `crate::base::scalar` traits.
//! This results in minor modifications.
//!
//! Additionally, `num_elem_per_thread` rounds up instead of down.

use core::{
    cmp::max,
    ops::{Mul, MulAssign},
};
use num_traits::{Inv, One, Zero};
use rayon::prelude::*;

/**
 * Adapted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */

/// Given a vector of field elements {v_i}, compute the vector {v_i^(-1)} using Montgomery's trick.
/// The vector is modified in place.
/// Any zero elements in the vector are left unchanged.
pub fn batch_inversion<F>(v: &mut [F])
where
    F: One + Zero + MulAssign + Inv<Output = F> + Mul<Output = F> + Send + Sync + Copy,
{
    batch_inversion_and_mul(v, F::one());
}

pub fn batch_inversion_and_mul<F>(v: &mut [F], coeff: F)
where
    F: One + Zero + MulAssign + Inv<Output = F> + Mul<Output = F> + Send + Sync + Copy,
{
    // Divide the vector v evenly between all available cores, but make sure that each
    // core has at least MIN_RAYON_LEN elements to work on
    let num_cpus_available = max(1, rayon::current_num_threads());
    let num_elem_per_thread = max(
        (v.len() + num_cpus_available - 1) / num_cpus_available,
        super::MIN_RAYON_LEN,
    );

    // Batch invert in parallel, without copying the vector
    v.par_chunks_mut(num_elem_per_thread).for_each(|chunk| {
        serial_batch_inversion_and_mul(chunk, coeff);
    });
}

fn serial_batch_inversion_and_mul<F>(v: &mut [F], coeff: F)
where
    F: One + Zero + MulAssign + Inv<Output = F> + Mul<Output = F> + Copy,
{
    // Montgomery’s Trick and Fast Implementation of Masked AES
    // Genelle, Prouff and Quisquater
    // Section 3.2
    // but with an optimization to multiply every element in the returned vector by
    // coeff

    // First pass: compute [a, ab, abc, ...]
    let mut prod = Vec::with_capacity(v.len());
    let mut tmp = F::one();
    for &f in v.iter().filter(|f| !f.is_zero()) {
        tmp *= f;
        prod.push(tmp);
    }

    // Invert `tmp`.
    tmp = tmp.inv(); // Guaranteed to be nonzero.

    // Multiply product by coeff, so all inverses will be scaled by coeff
    tmp *= coeff;

    // Second pass: iterate backwards to compute inverses
    for (f, s) in v
        .iter_mut()
        // Backwards
        .rev()
        // Ignore normalized elements
        .filter(|f| !f.is_zero())
        // Backwards, skip last element, fill in one for last term.
        .zip(prod.into_iter().rev().skip(1).chain(Some(F::one())))
    {
        // tmp := tmp * f; f := tmp * s = 1/f
        let new_tmp = tmp * *f;
        *f = tmp * s;
        tmp = new_tmp;
    }
}
