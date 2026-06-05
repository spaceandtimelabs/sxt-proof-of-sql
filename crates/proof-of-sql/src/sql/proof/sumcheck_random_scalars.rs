use crate::base::{polynomial::compute_evaluation_vector, scalar::Scalar};
use alloc::{vec, vec::Vec};

/// Accessor for the random scalars used to form the sumcheck polynomial of a query proof
pub struct SumcheckRandomScalars<'a, S: Scalar> {
    pub entrywise_point: &'a [S],
    pub subpolynomial_multipliers: &'a [S],
    pub table_length: usize,
}

impl<'a, S: Scalar> SumcheckRandomScalars<'a, S> {
    pub fn new(scalars: &'a [S], table_length: usize, num_sumcheck_variables: usize) -> Self {
        let num_subpolynomial_multipliers = scalars.len() - num_sumcheck_variables;
        let (subpolynomial_multipliers, entrywise_point) =
            scalars.split_at(num_subpolynomial_multipliers);
        Self {
            entrywise_point,
            subpolynomial_multipliers,
            table_length,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn compute_entrywise_multipliers(&self) -> Vec<S> {
        let mut v = vec![S::ZERO; self.table_length];
        compute_evaluation_vector(&mut v, self.entrywise_point);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::scalar::test_scalar::TestScalar;

    #[test]
    fn splits_scalars_into_subpolynomial_multipliers_and_entrywise_point() {
        let scalars = [
            TestScalar::from(2u64),
            TestScalar::from(3u64),
            TestScalar::from(5u64),
            TestScalar::from(7u64),
        ];

        let random_scalars = SumcheckRandomScalars::new(&scalars, 4, 2);

        assert_eq!(
            random_scalars.subpolynomial_multipliers,
            &[TestScalar::from(2u64), TestScalar::from(3u64)]
        );
        assert_eq!(
            random_scalars.entrywise_point,
            &[TestScalar::from(5u64), TestScalar::from(7u64)]
        );
        assert_eq!(random_scalars.table_length, 4);
    }

    #[test]
    fn computes_entrywise_multipliers_from_the_entrywise_point() {
        let scalars = [TestScalar::from(11u64), TestScalar::from(3u64)];
        let random_scalars = SumcheckRandomScalars::new(&scalars, 2, 1);

        assert_eq!(
            random_scalars.compute_entrywise_multipliers(),
            vec![
                TestScalar::ONE - TestScalar::from(3u64),
                TestScalar::from(3u64)
            ]
        );
    }
}
