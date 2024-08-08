use super::{pack_scalars, pairings, DoryCommitment, DoryProverPublicSetup, G1Affine};
use crate::base::commitment::CommittableColumn;
use ark_ec::CurveGroup;
use ark_std::ops::Mul;
use blitzar::compute::ElementP2;
use rayon::prelude::*;

#[tracing::instrument(name = "modify_commits (gpu)", level = "debug", skip_all)]
fn modify_commits(
    commits: &[G1Affine],
    committable_columns: &[CommittableColumn],
    num_of_outputs: usize,
    num_of_commits: usize,
) -> Vec<G1Affine> {
    let signed_commits_size = num_of_outputs * num_of_commits;

    let (signed_commits, offset_commits) = commits.split_at(signed_commits_size);

    // Currently, the packed_scalars doubles the number of commits
    // to deal with signed commits. Commit i is offset by commit i + num_of_commits.
    if signed_commits.len() != offset_commits.len() {
        return vec![];
    }

    signed_commits
        .par_iter()
        .zip(offset_commits.par_iter())
        .enumerate()
        .map(|(index, (first, second))| {
            let min = pack_scalars::get_min_as_fr(&committable_columns[index / num_of_commits]);
            let modified_second = second.mul(min).into_affine();
            *first + modified_second
        })
        .map(|point| point.into_affine())
        .collect()
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

    let num_of_outputs = committable_columns.len();
    let num_of_generators = 1 << setup.sigma();

    // Scale the offset to avoid adding columns of zero to the packed scalar
    // if the offset is larger than the number of generators.
    let gamma_2_offset = offset / num_of_generators;
    let offset = offset % num_of_generators;

    let num_of_commits =
        pack_scalars::get_num_of_commits(committable_columns, offset, num_of_generators);

    let gamma_2_slice = &setup.prover_setup().Gamma_2.last().unwrap()
        [gamma_2_offset..gamma_2_offset + num_of_commits];

    let (bit_table, packed_scalars) = pack_scalars::get_bit_table_and_scalars_for_packed_msm(
        committable_columns,
        offset,
        num_of_generators,
        num_of_commits,
    );

    let mut blitzar_commits =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); bit_table.len()];

    if !bit_table.is_empty() {
        setup.prover_setup().blitzar_packed_msm(
            &mut blitzar_commits,
            &bit_table,
            packed_scalars.as_slice(),
        );
    }

    let commits: Vec<G1Affine> = blitzar_commits.par_iter().map(Into::into).collect();

    let modified_commits = modify_commits(
        &commits,
        committable_columns,
        num_of_outputs,
        num_of_commits,
    );

    (0..num_of_outputs)
        .into_par_iter()
        .map(|i| {
            let idx = i * num_of_commits;
            let individual_commits: Vec<G1Affine> =
                modified_commits[idx..idx + num_of_commits].to_vec();

            DoryCommitment(pairings::multi_pairing(individual_commits, gamma_2_slice))
        })
        .collect::<Vec<_>>()
}

pub(super) fn compute_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> Vec<DoryCommitment> {
    compute_dory_commitments_packed_impl(committable_columns, offset, setup)
}
