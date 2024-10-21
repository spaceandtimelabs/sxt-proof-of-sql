use super::{
    blitzar_metadata_table::{create_blitzar_metadata_tables, signed_commits},
    dynamic_dory_structure::row_and_column_from_index,
    pairings, DynamicDoryCommitment, G1Affine, ProverSetup,
};
use crate::base::{commitment::CommittableColumn, if_rayon, slice_ops::slice_cast};
use blitzar::compute::ElementP2;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tracing::{span, Level};

/// Computes the dynamic Dory commitment using the GPU implementation of the `vlen_msm` algorithm.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
/// * `setup` - A reference to the prover setup.
///
/// # Returns
///
/// A vector containing the dynamic Dory commitments.
///
/// # Panics
///
/// Panics if the number of sub commits is not a multiple of the number of committable columns.
#[tracing::instrument(
    name = "compute_dynamic_dory_commitments (gpu)",
    level = "debug",
    skip_all
)]
pub(super) fn compute_dynamic_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &ProverSetup,
) -> Vec<DynamicDoryCommitment> {
    let Gamma_2 = setup.Gamma_2.last().unwrap();
    let (gamma_2_offset, _) = row_and_column_from_index(offset);

    // Get metadata tables for Blitzar's vlen_msm algorithm.
    let (blitzar_output_bit_table, blitzar_output_length_table, blitzar_scalars) =
        create_blitzar_metadata_tables(committable_columns, offset);

    // Initialize sub commits.
    let mut blitzar_sub_commits =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); blitzar_output_bit_table.len()];

    // Get sub commits from Blitzar's vlen_msm algorithm.
    setup.blitzar_vlen_msm(
        &mut blitzar_sub_commits,
        &blitzar_output_bit_table,
        &blitzar_output_length_table,
        blitzar_scalars.as_slice(),
    );

    // Modify the sub commits to include the signed offset.
    let all_sub_commits: Vec<G1Affine> = slice_cast(&blitzar_sub_commits);
    let signed_sub_commits = signed_commits(&all_sub_commits, committable_columns);
    assert!(
        signed_sub_commits.len() % committable_columns.len() == 0,
        "Invalid number of sub commits"
    );
    let num_commits = signed_sub_commits.len() / committable_columns.len();

    // Calculate the dynamic Dory commitments.
    let span = span!(Level::INFO, "multi_pairing").entered();
    let ddc: Vec<DynamicDoryCommitment> = signed_sub_commits
        .is_empty()
        .then_some(vec![
            DynamicDoryCommitment::default();
            committable_columns.len()
        ])
        .unwrap_or_else(|| {
            if_rayon!(
                (0..committable_columns.len())
                    .into_par_iter()
                    .map(|i| {
                        let sub_slice = signed_sub_commits[i..]
                            .iter()
                            .step_by(committable_columns.len())
                            .take(num_commits);
                        DynamicDoryCommitment(pairings::multi_pairing(
                            sub_slice,
                            &Gamma_2[..num_commits],
                        ))
                    })
                    .collect(),
                (0..committable_columns.len())
                    .map(|i| {
                        let sub_slice = signed_sub_commits[i..]
                            .iter()
                            .step_by(committable_columns.len())
                            .take(num_commits);
                        DynamicDoryCommitment(pairings::multi_pairing(
                            sub_slice,
                            &Gamma_2[..num_commits],
                        ))
                    })
                    .collect()
            )
        });
    span.exit();

    ddc
}
