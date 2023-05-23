use core::ops::{Mul, MulAssign};

use crate::base::slice_ops;

use super::{Inverse, One, Zero};

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
pub fn batch_pseudo_invert<F>(res: &mut [F], input: &[F])
where
    F: One + Zero + MulAssign + Inverse + Mul<Output = F> + Send + Sync + Copy,
{
    assert_eq!(res.len(), input.len());
    res.copy_from_slice(input);
    slice_ops::batch_inversion(res);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{polynomial::Scalar, scalar::to_scalar::ToScalar};

    #[test]
    fn we_can_pseudo_invert_empty_arrays() {
        let input: Vec<Scalar> = Vec::new();
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
        .take(slice_ops::MIN_RAYON_LEN * 10)
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
        .take(slice_ops::MIN_RAYON_LEN - 1)
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
