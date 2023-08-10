use crate::base::bit::make_abs_bit_mask;
use crate::base::bit::BitDistribution;
use crate::base::scalar::ArkScalar;
use bumpalo::Bump;

/// Let x1, ..., xn denote the values of a data column. Let
/// b1, ..., bk denote the bit positions of abs(x1), ..., abs(xn)
/// that vary.
///
/// compute_varying_bit_matrix returns the matrix M where
///   M_ij = abs(xi) & (1 << bj) == 1
/// The last column of M corresponds to the sign bit if it varies.
pub fn compute_varying_bit_matrix<'a>(
    alloc: &'a Bump,
    vals: &[ArkScalar],
    dist: &BitDistribution,
) -> Vec<&'a [bool]> {
    let n = vals.len();
    let num_varying_bits = dist.num_varying_bits();
    let data: &'a mut [bool] = alloc.alloc_slice_fill_default(n * num_varying_bits);

    // decompose
    for (i, val) in vals.iter().enumerate() {
        let mask = make_abs_bit_mask(*val);
        let mut offset = i;
        dist.for_each_varying_bit(|int_index: usize, bit_index: usize| {
            data[offset] = (mask[int_index] & (1u64 << bit_index)) != 0;
            offset += n;
        });
    }

    // make result
    let mut res = Vec::with_capacity(num_varying_bits);
    for bit_index in 0..num_varying_bits {
        let first = n * bit_index;
        let last = n * (bit_index + 1);
        res.push(&data[first..last]);
    }
    res
}
