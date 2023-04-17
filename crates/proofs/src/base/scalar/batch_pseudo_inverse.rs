use curve25519_dalek::scalar::Scalar;
use rayon::{iter::ParallelIterator, slice::ParallelSliceMut};

// These constants can likely be tuned. There is no reason for the current values other that that they are sane.
const MIN_CHUNKING_SIZE: usize = 64; // The minimum size for which we should actually split into chunks.

/// Given a slice of `input` scalars, compute their pseudo inverses in a batch and store the result in `res`.
///
/// Return:
/// - res[i] <- 0 if input[i] is zero
/// - res[i] <- input[i].invert() otherwise
///
/// Warning:
/// - both `input` and `res` must have the same length
#[tracing::instrument(
    name = "proofs.base.scalar.batch_pseudo_inverse.batch_pseudo_invert",
    level = "info",
    skip_all
)]
pub fn batch_pseudo_invert(res: &mut [Scalar], input: &[Scalar]) {
    assert_eq!(res.len(), input.len());

    // we copy the non-zero elements from input into res
    let mut count_non_zeros = 0_usize;
    for input_val in input.iter() {
        if *input_val != Scalar::zero() {
            res[count_non_zeros] = *input_val;
            count_non_zeros += 1;
        }
    }

    // we invert only the non-zero elements from input
    // note: this function can possibly allocate memory

    // we should break this up into chunks in order to parallelize it if there are enough non-zero elements
    let number_of_chunks = rayon::current_num_threads();
    if count_non_zeros >= MIN_CHUNKING_SIZE {
        res[0..count_non_zeros]
            .par_chunks_mut(1 + (count_non_zeros - 1) / number_of_chunks)
            .for_each(|c| {
                Scalar::batch_invert(c);
            });
    } else {
        Scalar::batch_invert(&mut res[0..count_non_zeros]);
    }

    // we can stop here in case all the elements from `res` are already non-zero
    if count_non_zeros == input.len() {
        return;
    }

    // we then copy the zero elements to res,
    // shifting the previous non-zero elements
    // from `res` to a higher index
    for index_rev_input in (0..input.len()).rev() {
        let input_val = &input[index_rev_input];

        if *input_val != Scalar::zero() {
            count_non_zeros -= 1;
            res[index_rev_input] = res[count_non_zeros];
        } else {
            res[index_rev_input] = Scalar::zero();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::to_scalar::ToScalar;

    #[test]
    fn we_can_pseudo_invert_empty_arrays() {
        let input = Vec::new();
        let mut res = Vec::new();
        batch_pseudo_invert(&mut res[..], &input[..]);
    }

    #[test]
    fn we_can_pseudo_invert_arrays_of_length_1_with_non_zero() {
        let input = vec![Scalar::from(2_u32)];
        let mut res = vec![Scalar::from(0_u32)];
        batch_pseudo_invert(&mut res[..], &input[..]);
        assert!(res == vec![input[0].invert()]);
    }

    #[test]
    fn we_can_pseudo_invert_arrays_of_length_1_with_zero() {
        let input = vec![Scalar::from(0_u32)];
        let mut res = vec![Scalar::from(0_u32)];
        batch_pseudo_invert(&mut res[..], &input[..]);
        assert!(res == vec![input[0]]);
    }

    #[test]
    fn we_can_pseudo_invert_arrays_of_length_bigger_than_1_with_zeros_and_non_zeros() {
        let input = vec![
            Scalar::from(0_u32),
            Scalar::from(2_u32),
            (-33_i32).to_scalar(),
            Scalar::from(0_u32),
            Scalar::from(45_u32),
            Scalar::from(0_u32),
            Scalar::from(47_u32),
        ];
        let mut res = vec![Scalar::from(0_u32); input.len()];
        batch_pseudo_invert(&mut res[..], &input[..]);

        for (input_val, res_val) in input.iter().zip(res) {
            if *input_val != Scalar::zero() {
                assert!(input_val.invert() == res_val);
            } else {
                assert!(Scalar::zero() == res_val);
            }
        }
    }

    #[test]
    fn we_can_pseudo_invert_arrays_with_nonzero_count_bigger_than_min_chunking_size_with_zeros_and_non_zeros(
    ) {
        let input: Vec<_> = vec![
            Scalar::from(0_u32),
            Scalar::from(2_u32),
            (-33_i32).to_scalar(),
            Scalar::from(0_u32),
            Scalar::from(45_u32),
            Scalar::from(0_u32),
            Scalar::from(47_u32),
        ]
        .into_iter()
        .cycle()
        .take(MIN_CHUNKING_SIZE * 10)
        .collect();

        let mut res = vec![Scalar::from(0_u32); input.len()];
        batch_pseudo_invert(&mut res[..], &input[..]);

        for (input_val, res_val) in input.iter().zip(res) {
            if *input_val != Scalar::zero() {
                assert!(input_val.invert() == res_val);
            } else {
                assert!(Scalar::zero() == res_val);
            }
        }
    }

    #[test]
    fn we_can_pseudo_invert_arrays_with_nonzero_count_smaller_than_min_chunking_size_with_zeros_and_non_zeros(
    ) {
        let input: Vec<_> = vec![
            Scalar::from(0_u32),
            Scalar::from(2_u32),
            (-33_i32).to_scalar(),
            Scalar::from(0_u32),
            Scalar::from(45_u32),
            Scalar::from(0_u32),
            Scalar::from(47_u32),
        ]
        .into_iter()
        .cycle()
        .take(MIN_CHUNKING_SIZE - 1)
        .collect();

        let mut res = vec![Scalar::from(0_u32); input.len()];
        batch_pseudo_invert(&mut res[..], &input[..]);

        for (input_val, res_val) in input.iter().zip(res) {
            if *input_val != Scalar::zero() {
                assert!(input_val.invert() == res_val);
            } else {
                assert!(Scalar::zero() == res_val);
            }
        }
    }
}
