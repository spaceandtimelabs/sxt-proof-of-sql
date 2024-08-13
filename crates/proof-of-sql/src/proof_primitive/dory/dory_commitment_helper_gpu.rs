use super::{pack_scalars, pairings, DoryCommitment, DoryProverPublicSetup, G1Affine};
use crate::base::commitment::CommittableColumn;
use ark_ec::CurveGroup;
use ark_std::ops::Mul;
use blitzar::compute::ElementP2;
use rayon::prelude::*;
use tracing::{span, Level};

#[tracing::instrument(name = "modify_commits (gpu)", level = "debug", skip_all)]
fn modify_commits(
    commits: &[G1Affine],
    committable_columns: &[CommittableColumn],
    num_of_full_commits: usize,
    num_of_sub_commits_per_full_commit: usize,
) -> Vec<G1Affine> {
    let num_of_signed_sub_commits = num_of_full_commits * num_of_sub_commits_per_full_commit;

    let (signed_sub_commits, offset_sub_commits) = commits.split_at(num_of_signed_sub_commits);

    // Currently, the packed_scalars doubles the number of commits
    // to deal with signed sub commits. Commit i is offset by commit i + num_of_sub_commits_per_full_commit.
    if signed_sub_commits.len() != offset_sub_commits.len() {
        return vec![];
    }

    signed_sub_commits
        .par_iter()
        .zip(offset_sub_commits.par_iter())
        .enumerate()
        .map(|(index, (first, second))| {
            let min = pack_scalars::get_min_as_fr(
                &committable_columns[index / num_of_sub_commits_per_full_commit],
            );
            let modified_second = second.mul(min).into_affine();
            *first + modified_second
        })
        .map(|point| point.into_affine())
        .collect::<Vec<_>>()
}

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

    let num_of_full_commits = committable_columns.len();
    let num_of_generators = 1 << setup.sigma();

    // Scale the offset to avoid adding columns of zero to the packed scalar
    // if the offset is larger than the number of generators.
    let gamma_2_offset = offset / num_of_generators;
    let offset = offset % num_of_generators;

    let num_of_commits_in_a_full_commit =
        pack_scalars::get_num_of_commits(committable_columns, offset, num_of_generators);

    let gamma_2_slice = &setup.prover_setup().Gamma_2.last().unwrap()
        [gamma_2_offset..gamma_2_offset + num_of_commits_in_a_full_commit];

    let (full_bit_table, packed_scalars) = pack_scalars::get_bit_table_and_scalars_for_packed_msm(
        committable_columns,
        offset,
        num_of_generators,
        num_of_commits_in_a_full_commit,
    );

    let mut sub_commits_from_blitzar =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); full_bit_table.len()];

    if !full_bit_table.is_empty() {
        setup.prover_setup().blitzar_packed_msm(
            &mut sub_commits_from_blitzar,
            &full_bit_table,
            packed_scalars.as_slice(),
        );
    }

    let sub_commits: Vec<G1Affine> = sub_commits_from_blitzar
        .par_iter()
        .map(Into::into)
        .collect();

    let modified_sub_commits = modify_commits(
        &sub_commits,
        committable_columns,
        num_of_full_commits,
        num_of_commits_in_a_full_commit,
    );

    let span = span!(Level::INFO, "multi_pairing").entered();
    let dc: Vec<DoryCommitment> = (0..num_of_full_commits)
        .into_par_iter()
        .map(|i| {
            let idx = i * num_of_commits_in_a_full_commit;
            let sub_commits_of_full_commit: Vec<G1Affine> =
                modified_sub_commits[idx..idx + num_of_commits_in_a_full_commit].to_vec();

            DoryCommitment(pairings::multi_pairing(
                sub_commits_of_full_commit,
                gamma_2_slice,
            ))
        })
        .collect::<Vec<_>>();
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
