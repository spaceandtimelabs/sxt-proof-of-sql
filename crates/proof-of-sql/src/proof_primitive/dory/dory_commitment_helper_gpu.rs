use super::{pack_scalars, pairings, DoryCommitment, DoryProverPublicSetup, G1Affine};
use crate::base::commitment::CommittableColumn;
use blitzar::compute::ElementP2;
use rayon::prelude::*;
use tracing::{span, Level};

#[tracing::instrument(
    name = "compute_dory_commitments_packed_impl (gpu)",
    level = "debug",
    skip_all
)]
fn compute_dory_commitments_packed_impl(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> Vec<DoryCommitment> {
    if committable_columns.is_empty() {
        return vec![];
    }

    let num_columns = 1 << setup.sigma();
    let gamma_2 = setup.prover_setup().Gamma_2.clone();

    // If the offset is larger than the number of columns, we compute an
    // offset for the gamma_2 table to avoid finding sub-commits of zero.
    let gamma_2_offset = offset / num_columns;
    let offset = offset % num_columns;

    let (bit_table, packed_scalars) = pack_scalars::bit_table_and_scalars_for_packed_msm(
        committable_columns,
        offset,
        num_columns,
    );

    let mut sub_commits_from_blitzar =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); bit_table.len()];

    // Get sub commits by computing the packed msm
    if !bit_table.is_empty() {
        setup.prover_setup().blitzar_packed_msm(
            &mut sub_commits_from_blitzar,
            &bit_table,
            packed_scalars.as_slice(),
        );
    }

    let all_sub_commits: Vec<G1Affine> = sub_commits_from_blitzar
        .par_iter()
        .map(Into::into)
        .collect();

    // Modify the sub-commits to account for signed values that were offset
    let modified_sub_commits_update = pack_scalars::modify_commits(
        &all_sub_commits,
        &bit_table,
        committable_columns,
        offset,
        num_columns,
    );

    // All columns are not guaranteed to have the same number of sub-commits
    let num_sub_commits_per_full_commit: Vec<usize> = committable_columns
        .par_iter()
        .map(|column| pack_scalars::num_sub_commits_update(column, offset, num_columns))
        .collect();

    // Compute the cumulative sum of the number of sub-commits per full commit
    let cumulative_sums: Vec<usize> = num_sub_commits_per_full_commit
        .iter()
        .scan(0, |acc, &x| {
            let prev = *acc;
            *acc += x;
            Some(prev)
        })
        .collect();

    // Compute the Dory commitments using multi pairing of sub-commits
    let span = span!(Level::INFO, "multi_pairing").entered();
    let dc: Vec<DoryCommitment> = (0..num_sub_commits_per_full_commit.len())
        .into_par_iter()
        .map(|i| {
            let idx = cumulative_sums[i];

            let sub_commits =
                &modified_sub_commits_update[idx..idx + num_sub_commits_per_full_commit[i]];

            let gamma_2_slice = &gamma_2.last().unwrap()
                [gamma_2_offset..gamma_2_offset + num_sub_commits_per_full_commit[i]];

            DoryCommitment(pairings::multi_pairing(sub_commits, gamma_2_slice))
        })
        .collect();
    span.exit();

    dc
}

pub(super) fn compute_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> Vec<DoryCommitment> {
    compute_dory_commitments_packed_impl(committable_columns, offset, setup)
}
