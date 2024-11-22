use super::{
    hyrax_commitment::HyraxCommitment,
    hyrax_configuration::HyraxConfiguration,
    hyrax_helpers::{compute_dynamic_vecs, matrix_size, row_and_column_from_index},
    hyrax_public_setup::HyraxPublicSetup,
    hyrax_scalar::HyraxScalarWrapper,
};
use crate::base::{
    commitment::CommitmentEvaluationProof,
    if_rayon,
    proof::{ProofError, Transcript},
    scalar::Scalar,
};
use alloc::{vec, vec::Vec};
#[cfg(feature = "rayon")]
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use serde::{Deserialize, Serialize};

/// The Hyrax evaluation proof scheme
#[derive(Serialize, Deserialize)]
pub struct HyraxCommitmentEvaluationProof<C: HyraxConfiguration> {
    /// Represents the matrix cross the high vector.
    /// Verified against the product with the low vector.
    /// Verified against the commitment and high vector using the generators.
    pub witness: Vec<C::OperableScalar>,
}

impl<C: HyraxConfiguration> CommitmentEvaluationProof for HyraxCommitmentEvaluationProof<C>
where
    for<'b> C::OperableGroup: 'b,
{
    type Scalar = HyraxScalarWrapper<C::OperableScalar>;

    type Commitment = HyraxCommitment<C>;

    type Error = ProofError;
    type ProverPublicSetup<'a> = HyraxPublicSetup<'a, C::OperableGroup>;
    type VerifierPublicSetup<'a> = HyraxPublicSetup<'a, C::OperableGroup>;

    fn new(
        _transcript: &mut impl Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        _generators_offset: u64,
        _setup: &Self::ProverPublicSetup<'_>,
    ) -> Self {
        let (_lo_vec, high_vec) = compute_dynamic_vecs(b_point);
        let empty_column_scalars = vec![C::OperableScalar::ZERO; matrix_size(a.len(), 0).0];
        let witness: Vec<_> = if_rayon!((0..a.len()).into_par_iter(), (0..a.len()).into_iter())
            .map(|index| {
                let (row, column) = row_and_column_from_index(index);
                (column, a[index].0 * high_vec[row].0)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .fold(
                empty_column_scalars,
                |mut acc: Vec<C::OperableScalar>, (column, scalar)| {
                    acc[column] += scalar;
                    acc
                },
            )
            .into_iter()
            .collect();
        Self { witness }
    }

    fn verify_batched_proof(
        &self,
        _transcript: &mut impl Transcript,
        commit_batch: &[Self::Commitment],
        batching_factors: &[Self::Scalar],
        product: &Self::Scalar,
        b_point: &[Self::Scalar],
        _generators_offset: u64,
        _table_length: usize,
        setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        let (lo_vec, high_vec) = compute_dynamic_vecs(b_point);
        let expected_product = if_rayon!(self.witness.par_iter(), self.witness.iter())
            .zip(lo_vec)
            .map(|(s, l)| *s * l.0)
            .sum::<C::OperableScalar>();
        assert_eq!(product.0, expected_product);
        let generators_by_witness = if_rayon!(setup.generators.par_iter(), setup.generators.iter())
            .zip(self.witness.clone())
            .map(|(g, w)| *g * w)
            .sum::<C::OperableGroup>();
        let row_commits_by_high = if_rayon!(commit_batch.par_iter(), commit_batch.iter())
            .zip(batching_factors)
            .map(|(hc, bf)| {
                let row_commits_iter = if_rayon!(hc.row_commits.par_iter(), hc.row_commits.iter());
                row_commits_iter
                    .zip(high_vec.clone())
                    .map(|(rc, hs)| C::from_compressed_to_operable(rc) * (hs.0 * bf.0))
                    .sum::<C::OperableGroup>()
            })
            .sum::<C::OperableGroup>();

        assert_eq!(generators_by_witness, row_commits_by_high);
        Ok(())
    }
}
