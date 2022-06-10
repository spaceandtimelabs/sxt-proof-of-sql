/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use curve25519_dalek::scalar::Scalar;

use crate::base::math::{scalar_factorial, u128_factorial, u64_factorial};

/// interpolate a uni-variate degree-`p_i.len()-1` polynomial and evaluate this
/// polynomial at `eval_at`:
///   \sum_{i=0}^len p_i * (\prod_{j!=i} (eval_at - j)/(i-j))
pub fn interpolate_uni_poly(p_i: &[Scalar], eval_at: Scalar) -> Scalar {
    let len = p_i.len();

    let mut evals = vec![];

    let mut prod = eval_at;
    evals.push(eval_at);

    // `prod = \prod_{j} (eval_at - j)`
    for e in 1..len {
        let tmp = eval_at - Scalar::from(e as u64);
        evals.push(tmp);
        prod *= tmp;
    }
    let mut res = Scalar::zero();
    // we want to compute \prod (j!=i) (i-j) for a given i
    //
    // we start from the last step, which is
    //  denom[len-1] = (len-1) * (len-2) *... * 2 * 1
    // the step before that is
    //  denom[len-2] = (len-2) * (len-3) * ... * 2 * 1 * -1
    // and the step before that is
    //  denom[len-3] = (len-3) * (len-4) * ... * 2 * 1 * -1 * -2
    //
    // i.e., for any i, the one before this will be derived from
    //  denom[i-1] = - denom[i] * (len-i) / i
    //
    // that is, we only need to store
    // - the last denom for i = len-1, and
    // - the ratio between the current step and the last step, which is the
    //   product of -(len-i) / i from all previous steps and we store
    //   this product as a fraction number to reduce field divisions.

    // We know
    //  - 2^61 < factorial(20) < 2^62
    //  - 2^122 < factorial(33) < 2^123
    // so we will be able to compute the ratio
    //  - for len <= 20 with i64
    //  - for len <= 33 with i128
    //  - for len >  33 with BigInt
    if p_i.len() <= 20 {
        let last_denom = Scalar::from(u64_factorial(len - 1));
        let mut ratio_numerator = 1i64;
        let mut ratio_enumerator = 1u64;

        for i in (0..len).rev() {
            let ratio_numerator_f = if ratio_numerator < 0 {
                -Scalar::from((-ratio_numerator) as u64)
            } else {
                Scalar::from(ratio_numerator as u64)
            };

            res += p_i[i]
                * prod
                * Scalar::from(ratio_enumerator)
                * (last_denom * ratio_numerator_f * evals[i]).invert();

            // compute ratio for the next step which is current_ratio * -(len-i)/i
            if i != 0 {
                ratio_numerator *= -(len as i64 - i as i64);
                ratio_enumerator *= i as u64;
            }
        }
    } else if p_i.len() <= 33 {
        let last_denom = Scalar::from(u128_factorial(len - 1));
        let mut ratio_numerator = 1i128;
        let mut ratio_enumerator = 1u128;

        for i in (0..len).rev() {
            let ratio_numerator_f = if ratio_numerator < 0 {
                -Scalar::from((-ratio_numerator) as u128)
            } else {
                Scalar::from(ratio_numerator as u128)
            };

            res += p_i[i]
                * prod
                * Scalar::from(ratio_enumerator)
                * (last_denom * ratio_numerator_f * evals[i]).invert();

            // compute ratio for the next step which is current_ratio * -(len-i)/i
            if i != 0 {
                ratio_numerator *= -(len as i128 - i as i128);
                ratio_enumerator *= i as u128;
            }
        }
    } else {
        // since we are using field operations, we can merge
        // `last_denom` and `ratio_numerator` into a single field element.
        let mut denom_up = scalar_factorial(len - 1);
        let mut denom_down = Scalar::one();

        for i in (0..len).rev() {
            res += p_i[i] * prod * denom_down * (denom_up * evals[i]).invert();

            // compute denom for the next step is -current_denom * (len-i)/i
            if i != 0 {
                denom_up *= -Scalar::from((len - i) as u64);
                denom_down *= Scalar::from(i as u64);
            }
        }
    }

    res
}
