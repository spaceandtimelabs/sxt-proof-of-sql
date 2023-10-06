use super::compute_evaluation_vector;
use crate::base::scalar::ArkScalar;

/// Accessor for the random scalars used to form the sumcheck polynomial of a query proof
pub struct SumcheckRandomScalars<'a> {
    pub entrywise_point: &'a [ArkScalar],
    pub subpolynomial_multipliers: &'a [ArkScalar],
    pub table_length: usize,
}

impl<'a> SumcheckRandomScalars<'a> {
    pub fn new(
        scalars: &'a [ArkScalar],
        table_length: usize,
        num_sumcheck_variables: usize,
    ) -> Self {
        let (entrywise_point, subpolynomial_multipliers) = scalars.split_at(num_sumcheck_variables);
        Self {
            entrywise_point,
            subpolynomial_multipliers,
            table_length,
        }
    }

    pub fn compute_entrywise_multipliers(&self) -> Vec<ArkScalar> {
        let mut v = vec![Default::default(); self.table_length];
        compute_evaluation_vector(&mut v, self.entrywise_point);
        v
    }
}
