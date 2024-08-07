use super::{
    pack_scalars, pairings, transpose, DoryCommitment, DoryProverPublicSetup, DoryScalar, G1Affine,
    G2Affine,
};
use crate::base::commitment::CommittableColumn;
use ark_bls12_381::Fr;
use ark_ec::CurveGroup;
use ark_std::ops::Mul;
use blitzar::{compute::ElementP2, sequence::Sequence};
use rayon::prelude::*;
use zerocopy::AsBytes;

#[tracing::instrument(name = "get_offset_commits (gpu)", level = "debug", skip_all)]
fn get_offset_commits(
    column_len: usize,
    offset: usize,
    num_columns: usize,
    num_of_commits: usize,
    scalar: Fr,
    setup: &DoryProverPublicSetup,
) -> Vec<G1Affine> {
    let first_row_offset = offset % num_columns;
    let first_row_len = column_len.min(num_columns - first_row_offset);
    let num_zero_commits = offset / num_columns;
    let data_size = 1;

    let ones = vec![1_u8; column_len];
    let (first_row, remaining_elements) = ones.split_at(first_row_len);

    let mut ones_blitzar_commits =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); num_of_commits];

    if num_zero_commits < num_of_commits {
        // Get the commit of the first non-zero row
        let first_row_offset = offset - (num_zero_commits * num_columns);
        let first_row_transpose = transpose::transpose_for_fixed_msm(
            first_row,
            first_row_offset,
            1,
            num_columns,
            data_size,
        );

        setup.prover_setup().blitzar_msm(
            &mut ones_blitzar_commits[num_zero_commits..num_zero_commits + 1],
            data_size as u32,
            first_row_transpose.as_slice(),
        );

        // If there are more rows, get the commits of the middle row and duplicate them
        let mut chunks = remaining_elements.chunks(num_columns);
        if chunks.len() > 1 {
            if let Some(middle_row) = chunks.next() {
                let middle_row_transpose =
                    transpose::transpose_for_fixed_msm(middle_row, 0, 1, num_columns, data_size);
                let mut middle_row_blitzar_commit =
                    vec![ElementP2::<ark_bls12_381::g1::Config>::default(); 1];

                setup.prover_setup().blitzar_msm(
                    &mut middle_row_blitzar_commit,
                    data_size as u32,
                    middle_row_transpose.as_slice(),
                );

                ones_blitzar_commits[num_zero_commits + 1..num_of_commits - 1]
                    .par_iter_mut()
                    .for_each(|commit| *commit = middle_row_blitzar_commit[0].clone());
            }
        }

        // Get the commit of the last row to handle an zero padding at the end of the column
        if let Some(last_row) = remaining_elements.chunks(num_columns).last() {
            let last_row_transpose =
                transpose::transpose_for_fixed_msm(last_row, 0, 1, num_columns, data_size);

            setup.prover_setup().blitzar_msm(
                &mut ones_blitzar_commits[num_of_commits - 1..num_of_commits],
                data_size as u32,
                last_row_transpose.as_slice(),
            );
        }
    }

    ones_blitzar_commits
        .par_iter()
        .map(Into::into)
        .map(|commit: G1Affine| commit.mul(scalar).into_affine())
        .collect()
}

#[tracing::instrument(name = "compute_dory_commitment_impl (gpu)", level = "debug", skip_all)]
fn compute_dory_commitment_impl<'a, T>(
    column: &'a [T],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment
where
    &'a T: Into<DoryScalar>,
    &'a [T]: Into<Sequence<'a>>,
    T: AsBytes + Copy + transpose::OffsetToBytes,
{
    let num_columns = 1 << setup.sigma();
    let data_size = std::mem::size_of::<T>();

    // Format column to match column major data layout required by blitzar's msm
    let num_of_commits = ((column.len() + offset) + num_columns - 1) / num_columns;
    let column_transpose =
        transpose::transpose_for_fixed_msm(column, offset, num_of_commits, num_columns, data_size);
    let gamma_2_slice = &setup.prover_setup().Gamma_2.last().unwrap()[0..num_of_commits];

    // Compute the commitment for the entire data set
    let mut blitzar_commits =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); num_of_commits];
    setup.prover_setup().blitzar_msm(
        &mut blitzar_commits,
        data_size as u32,
        column_transpose.as_slice(),
    );

    let commits: Vec<G1Affine> = blitzar_commits.par_iter().map(Into::into).collect();

    // Signed data requires offset commitments
    if T::IS_SIGNED {
        let offset_commits = get_offset_commits(
            column.len(),
            offset,
            num_columns,
            num_of_commits,
            T::min_as_fr(),
            setup,
        );

        DoryCommitment(
            pairings::multi_pairing(commits, gamma_2_slice)
                + pairings::multi_pairing(offset_commits, gamma_2_slice),
        )
    } else {
        DoryCommitment(pairings::multi_pairing(commits, gamma_2_slice))
    }
}

fn compute_dory_commitment(
    committable_column: &CommittableColumn,
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment {
    match committable_column {
        CommittableColumn::SmallInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::BigInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int128(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Decimal75(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
        CommittableColumn::Scalar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::VarChar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Boolean(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::TimestampTZ(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
    }
}

#[tracing::instrument(name = "modify_commits (gpu)", level = "debug", skip_all)]
fn modify_commits(
    commits: &Vec<G1Affine>,
    committable_columns: &[CommittableColumn],
    signed_commits_size: usize,
    num_of_commits: usize,
) -> Vec<G1Affine> {
    let (signed_commits, offset_commits) = commits.split_at(signed_commits_size);

    signed_commits
        .iter()
        .zip(offset_commits.iter())
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
    // if the offset is larger than the number of generators
    let gamma_2_offset = offset / num_of_generators;
    let offset = offset % num_of_generators;

    let num_of_commits =
        pack_scalars::get_num_of_commits(&committable_columns, offset, num_of_generators);
        
    let gamma_2_slice = &setup.prover_setup().Gamma_2.last().unwrap()
        [gamma_2_offset..gamma_2_offset + num_of_commits];

    let bit_table = pack_scalars::get_output_bit_table(committable_columns);

    let (bit_table_for_packed_msm, packed_scalars) =
        pack_scalars::get_bit_table_and_scalar_for_packed_msm(
            &bit_table,
            committable_columns,
            offset,
            num_of_generators,
            num_of_commits,
        );

    let signed_commits_size = num_of_outputs * num_of_commits;
    let mut blitzar_commits =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); 2 * signed_commits_size];

    if !bit_table_for_packed_msm.is_empty() {
        setup.prover_setup().blitzar_packed_msm(
            &mut blitzar_commits,
            &bit_table_for_packed_msm,
            &packed_scalars.as_slice(),
        );
    }

    let commits: Vec<G1Affine> = blitzar_commits.par_iter().map(Into::into).collect();

    let modified_commits = modify_commits(
        &commits,
        &committable_columns,
        signed_commits_size,
        num_of_commits,
    );

    (0..num_of_outputs)
        .map(|i| {
            let idx = i * num_of_commits;
            let individual_commits: Vec<G1Affine> =
                modified_commits[idx..idx + num_of_commits].to_vec();

            DoryCommitment(pairings::multi_pairing(&individual_commits, gamma_2_slice))
        })
        .collect()
}

pub(super) fn compute_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> Vec<DoryCommitment> {
    compute_dory_commitments_packed_impl(committable_columns, offset, setup)
}
