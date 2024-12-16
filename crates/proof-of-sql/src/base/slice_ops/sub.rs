use crate::base::if_rayon;
#[cfg(feature = "rayon")]
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

/// This operation does `result[i] = lhs[i] - rhs[i]` for `i` in `0..lhs.len()`.
///
/// # Panics
/// Panics if the length of `lhs` and `rhs` are not equal.
pub fn sub<T>(result: &mut [T], lhs: &[T], rhs: &[T])
where
    T: Send + Sync + std::ops::Sub<Output = T> + Copy,
{
    assert!(
        lhs.len() == rhs.len(),
        "The length of lhs and rhs must be equal"
    );
    if_rayon!(
        result.par_iter_mut().with_min_len(super::MIN_RAYON_LEN),
        result.iter_mut()
    )
    .zip(lhs)
    .zip(rhs)
    .for_each(|((res_i, &lhs_i), &rhs_i)| {
        *res_i = lhs_i - rhs_i;
    });
}

#[cfg(test)]
mod tests {
    use super::sub;
    use crate::base::scalar::{test_scalar::TestScalar, Scalar};

    #[test]
    fn test_sub() {
        let lhs = vec![5, 10, 15, 20];
        let rhs = vec![1, 2, 3, 4];
        let mut result = vec![0; lhs.len()];
        sub(&mut result, &lhs, &rhs);
        assert_eq!(result, vec![4, 8, 12, 16]);
    }

    #[test]
    #[should_panic(expected = "The length of lhs and rhs must be equal")]
    fn test_sub_panic() {
        let lhs = vec![5, 10, 15];
        let rhs = vec![1, 2, 3, 4];
        let mut result = vec![0; lhs.len()];
        sub(&mut result, &lhs, &rhs);
    }

    #[test]
    fn test_sub_with_scalars() {
        let lhs = vec![
            TestScalar::from(5),
            TestScalar::from(10),
            TestScalar::from(15),
        ];
        let rhs = vec![
            TestScalar::from(10),
            TestScalar::from(9),
            TestScalar::from(3),
        ];
        let mut result = vec![TestScalar::ZERO; lhs.len()];
        sub(&mut result, &lhs, &rhs);
        assert_eq!(
            result,
            vec![
                TestScalar::from(-5),
                TestScalar::from(1),
                TestScalar::from(12)
            ]
        );
    }
}
