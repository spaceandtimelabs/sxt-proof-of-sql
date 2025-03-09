use super::{
    hyrax_commitment::HyraxCommitment, hyrax_configuration::HyraxConfiguration,
    hyrax_error::HyraxError, hyrax_public_setup::HyraxPublicSetup,
    hyrax_scalar::HyraxScalarWrapper,
};
use crate::{
    base::{commitment::CommitmentEvaluationProof, if_rayon, proof::Transcript, scalar::Scalar},
    proof_primitive::dynamic_matrix_utils::{
        matrix_structure::{matrix_size, row_and_column_from_index},
        standard_basis_helper::compute_dynamic_vecs,
    },
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
    witness: Vec<C::OperableScalar>,
    challenge: [u8; 32],
}

impl<C: HyraxConfiguration> CommitmentEvaluationProof for HyraxCommitmentEvaluationProof<C>
where
    for<'b> C::OperableGroup: 'b,
{
    type Scalar = HyraxScalarWrapper<C::OperableScalar>;

    type Commitment = HyraxCommitment<C>;

    type Error = HyraxError;
    type ProverPublicSetup<'a> = HyraxPublicSetup<'a, C::OperableGroup>;
    type VerifierPublicSetup<'a> = HyraxPublicSetup<'a, C::OperableGroup>;

    /// Creates a `HyraxCommitmentEvaluationProof`
    ///
    /// Given a column of data and a random `b_point`, the resulting proof is effectively the witness:
    /// the product of the `high_vec` (which is derved from `b_point`) and the column of data (transformed to a dynamic matrix).
    fn new(
        transcript: &mut impl Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _setup: &Self::ProverPublicSetup<'_>,
    ) -> Self {
        if generators_offset != 0 {
            return Self {
                witness: Vec::new(),
                challenge: [0u8; 32],
            };
        }
        let challenge = transcript.challenge_as_le();
        let (_, high_vec) = compute_dynamic_vecs(b_point);
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
        transcript.extend_scalars_as_be(witness.iter());
        Self { witness, challenge }
    }

    /// Verifies a batched hyrax proof
    ///
    /// Given a proof, commitment, inner product, and random `b_point`, the proof is verified against the commitment and `b_point` as follows:
    /// - The product of the witness and the the `lo_vec` (which is derved from `b_point`) should equal the inner product. `hi_vec` and `lo_vec` are
    ///   defined so that LMH is equal to M evaluated at `b_point`, which is why this constraint must hold (remember witness = MH).
    /// - The product of the vector of group elements in the commitment with the high vector should be equal to the product of the witness and the generators.
    ///   This must hold because commitment = GM, witness = MH, and (GM)H = G(MH).
    fn verify_batched_proof(
        &self,
        transcript: &mut impl Transcript,
        commit_batch: &[Self::Commitment],
        batching_factors: &[Self::Scalar],
        evaluations: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _table_length: usize,
        setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        if generators_offset != 0 {
            return Err(HyraxError::InvalidGeneratorsOffset {
                offset: generators_offset,
            });
        }
        if transcript.challenge_as_le() != self.challenge {
            return Err(HyraxError::VerificationError);
        }
        let (lo_vec, high_vec) = compute_dynamic_vecs(b_point);
        let expected_product = if_rayon!(self.witness.par_iter(), self.witness.iter())
            .zip(lo_vec)
            .map(|(s, l)| *s * l.0)
            .sum::<C::OperableScalar>();
        let product: Self::Scalar = evaluations
            .iter()
            .zip(batching_factors)
            .map(|(&e, &f)| e * f)
            .sum();
        if product.0 != expected_product {
            return Err(HyraxError::VerificationError);
        }
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

        if generators_by_witness != row_commits_by_high {
            return Err(HyraxError::VerificationError);
        }
        transcript.extend_scalars_as_be(self.witness.iter());
        Ok(())
    }
}
