use super::{pack_scalars, pairings, DoryCommitment, DoryProverPublicSetup, G1Affine};
use crate::{
    base::{commitment::CommittableColumn, if_rayon, slice_ops::slice_cast},
    utils::log,
};
use blitzar::compute::ElementP2;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use tracing::{span, Level};

#[tracing::instrument(
    name = "compute_dory_commitments_packed_impl (gpu)",
    level = "debug",
    skip_all
)]
/// # Panics
///
/// Will panic if:
/// - `Gamma_2.last()` returns `None` during the computation of the gamma_2 slice.
/// - The slice indexing in `gamma_2.last().unwrap()` is out of bounds, which can happen if `gamma_2_offset + num_sub_commits` exceeds the length of `Gamma_2`.
fn compute_dory_commitments_packed_impl(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> Vec<DoryCommitment> {
    log::log_memory_usage("Start");

    // Make sure that the committable columns are not empty.
    if committable_columns.is_empty() {
        return vec![];
    }

    // Set the parameters.
    let num_columns = 1 << setup.sigma();
    let gamma_2 = &setup.prover_setup().Gamma_2;

    // If the offset is larger than the number of columns, we compute an
    // offset for the gamma_2 table to avoid performing msm on all zeros.
    let gamma_2_offset = offset / num_columns;
    let offset = offset % num_columns;

    // Pack the scalars and create the bit table.
    let (bit_table, packed_scalars) = pack_scalars::bit_table_and_scalars_for_packed_msm(
        committable_columns,
        offset,
        num_columns,
    );

    // Get sub commits by computing the packed msm.
    let mut sub_commits_from_blitzar =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); bit_table.len()];

    if !bit_table.is_empty() {
        setup.prover_setup().blitzar_packed_msm(
            &mut sub_commits_from_blitzar,
            &bit_table,
            packed_scalars.as_slice(),
        );
    }

    // Convert the sub-commits to G1Affine.
    let all_sub_commits: Vec<G1Affine> = slice_cast(&sub_commits_from_blitzar);

    // Modify the sub-commits to account for signed values that were offset.
    let modified_sub_commits_update = pack_scalars::modify_commits(
        &all_sub_commits,
        &bit_table,
        committable_columns,
        offset,
        num_columns,
    );

    // All columns are not guaranteed to have the same number of sub-commits.
    // Create a vector that stores the number of sub-commits per full commit.
    let num_sub_commits_per_full_commit: Vec<usize> = committable_columns
        .iter()
        .map(|column| pack_scalars::num_sub_commits(column, offset, num_columns))
        .collect();

    // Compute the cumulative sum of the number of sub-commits per full commit.
    // This is used to index into the modified sub-commits in the Doris commitment loop.
    let cumulative_sub_commit_sums: Vec<usize> = num_sub_commits_per_full_commit
        .iter()
        .scan(0, |acc, &x| {
            let prev = *acc;
            *acc += x;
            Some(prev)
        })
        .collect();

    // Compute the Dory commitments using multi pairing of sub-commits.
    let span = span!(Level::DEBUG, "multi_pairing").entered();
    let dc: Vec<DoryCommitment> = if_rayon!(
        cumulative_sub_commit_sums.par_iter(),
        cumulative_sub_commit_sums.iter()
    )
    .zip(&num_sub_commits_per_full_commit)
    .map(|(&idx, &num_sub_commits)| {
        let sub_commits = &modified_sub_commits_update[idx..idx + num_sub_commits];
        let gamma_2_slice =
            &gamma_2.last().unwrap()[gamma_2_offset..gamma_2_offset + num_sub_commits];

        DoryCommitment(pairings::multi_pairing(sub_commits, gamma_2_slice))
    })
    .collect();
    span.exit();

    log::log_memory_usage("End");

    dc
}

pub(super) fn compute_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> Vec<DoryCommitment> {
    compute_dory_commitments_packed_impl(committable_columns, offset, setup)
}
