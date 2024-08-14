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

    // If the offset is larger than the number of columns, we compute an
    // offset for the gamma_2 table to avoid finding sub-commits of zero.
    let gamma_2_offset = offset / num_columns;
    let offset = offset % num_columns;

    // Get the number of sub-commits for each full commit
    let num_matrix_commitment_columns =
        pack_scalars::num_matrix_commitment_columns(committable_columns, offset, num_columns);

    // Get the bit table and packed scalars for the packed msm
    let (bit_table, packed_scalars) = pack_scalars::bit_table_and_scalars_for_packed_msm(
        committable_columns,
        offset,
        num_columns,
        num_matrix_commitment_columns,
    );

    let mut sub_commits_from_blitzar =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); bit_table.len()];

    // Compute packed msm
    if !bit_table.is_empty() {
        setup.prover_setup().blitzar_packed_msm(
            &mut sub_commits_from_blitzar,
            &bit_table,
            packed_scalars.as_slice(),
        );
    }

    // Convert the sub-commits to G1Affine
    let all_sub_commits: Vec<G1Affine> = sub_commits_from_blitzar
        .par_iter()
        .map(Into::into)
        .collect();

    // Modify the signed sub-commits by adding the offset
    let modified_sub_commits = pack_scalars::modify_commits(
        &all_sub_commits,
        committable_columns,
        num_matrix_commitment_columns,
    );

    let gamma_2_slice = &setup.prover_setup().Gamma_2.last().unwrap()
        [gamma_2_offset..gamma_2_offset + num_matrix_commitment_columns];

    // Compute the Dory commitments using multi pairing of sub-commits
    let span = span!(Level::INFO, "multi_pairing").entered();
    let dc = modified_sub_commits
        .par_chunks_exact(num_matrix_commitment_columns)
        .map(|sub_commits| DoryCommitment(pairings::multi_pairing(sub_commits, gamma_2_slice)))
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
