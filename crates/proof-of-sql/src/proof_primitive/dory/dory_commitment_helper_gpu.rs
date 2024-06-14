use super::{pairings, transpose, DoryCommitment, DoryProverPublicSetup, DoryScalar, G1Affine};
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
        let first_row_transpose =
            transpose::transpose_for_fixed_msm(first_row, first_row_offset, num_columns, data_size);

        setup.public_parameters().blitzar_msm(
            &mut ones_blitzar_commits[num_zero_commits..num_zero_commits + 1],
            data_size as u32,
            first_row_transpose.as_slice(),
        );

        // If there are more rows, get the commits of the middle row and duplicate them
        let mut chunks = remaining_elements.chunks(num_columns);
        if chunks.len() > 1 {
            if let Some(middle_row) = chunks.next() {
                let middle_row_transpose =
                    transpose::transpose_for_fixed_msm(middle_row, 0, num_columns, data_size);
                let mut middle_row_blitzar_commit =
                    vec![ElementP2::<ark_bls12_381::g1::Config>::default(); 1];

                setup.public_parameters().blitzar_msm(
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
                transpose::transpose_for_fixed_msm(last_row, 0, num_columns, data_size);

            setup.public_parameters().blitzar_msm(
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
    let column_transpose =
        transpose::transpose_for_fixed_msm(column, offset, num_columns, data_size);
    let num_of_commits = column_transpose.len() / (data_size * num_columns);
    let gamma_2_slice = &setup.public_parameters().Gamma_2[0..num_of_commits];

    // Compute the commitment for the entire data set
    let mut blitzar_commits =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); num_of_commits];
    setup.public_parameters().blitzar_msm(
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
    }
}

pub(super) fn compute_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> Vec<DoryCommitment> {
    committable_columns
        .iter()
        .map(|column| compute_dory_commitment(column, offset, setup))
        .collect()
}
