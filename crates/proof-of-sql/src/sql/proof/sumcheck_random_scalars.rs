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
        let (entrywise_point, subpolynomial_multipliers) = scalars.split_at(num_sumcheck_variables);
        Self {
            entrywise_point,
            subpolynomial_multipliers,
            table_length,
        }
    }

    pub fn compute_entrywise_multipliers(&self) -> Vec<S> {
        let mut v = vec![Default::default(); self.table_length];
        compute_evaluation_vector(&mut v, self.entrywise_point);
        v
    }
}
