use super::{pairings, DoryScalar, DynamicDoryCommitment, G1Projective, ProverSetup, GT};
use crate::{
    base::{commitment::CommittableColumn, if_rayon, slice_ops::slice_cast},
    proof_primitive::dynamic_matrix_utils::matrix_structure::{
        full_width_of_row, row_and_column_from_index, row_start_index,
    },
};
use alloc::vec::Vec;
use ark_ec::VariableBaseMSM;
use bytemuck::TransparentWrapper;
use num_traits::Zero;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};

#[tracing::instrument(name = "compute_dory_commitment_impl (cpu)", level = "debug", skip_all)]
/// # Panics
///
/// Will panic if:
/// - `setup.Gamma_1.last()` returns `None`, indicating that `Gamma_1` is empty.
/// - `setup.Gamma_2.last()` returns `None`, indicating that `Gamma_2` is empty.
/// - The indexing for `Gamma_2` with `first_row..=last_row` goes out of bounds.
#[allow(clippy::range_plus_one)]
fn compute_dory_commitment_impl<'a, T>(
    column: &'a [T],
    offset: usize,
    setup: &ProverSetup,
) -> DynamicDoryCommitment
where
    &'a T: Into<DoryScalar>,
    T: Sync,
{
    if column.is_empty() {
        return DynamicDoryCommitment::default();
    }
    let Gamma_1 = setup.Gamma_1.last().unwrap();
    let Gamma_2 = setup.Gamma_2.last().unwrap();
    let (first_row, first_col) = row_and_column_from_index(offset);
    let (last_row, last_col) = row_and_column_from_index(offset + column.len() - 1);

    let row_commits: Vec<_> = if_rayon!(
        (first_row..=last_row).into_par_iter(),
        (first_row..=last_row)
    )
    .map(|row| {
        let width = full_width_of_row(row);
        let row_start = row_start_index(row);
        let (gamma_range, column_range) = if first_row == last_row {
            (first_col..last_col + 1, 0..column.len())
        } else if row == 1 {
            (1..2, (1 - offset)..(2 - offset))
        } else if row == first_row {
            (first_col..width, 0..width - first_col)
        } else if row == last_row {
            (0..last_col + 1, column.len() - last_col - 1..column.len())
        } else {
            (0..width, row_start - offset..width + row_start - offset)
        };
        G1Projective::msm_unchecked(
            &Gamma_1[gamma_range],
            TransparentWrapper::peel_slice(&slice_cast::<_, DoryScalar>(&column[column_range])),
        )
    })
    .collect();

    DynamicDoryCommitment(pairings::multi_pairing(
        row_commits,
        &Gamma_2[first_row..=last_row],
    ))
}

fn compute_dory_commitment(
    committable_column: &CommittableColumn,
    offset: usize,
    setup: &ProverSetup,
) -> DynamicDoryCommitment {
    match committable_column {
        CommittableColumn::Scalar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Uint8(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::TinyInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::SmallInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::BigInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int128(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::VarChar(column)
        | CommittableColumn::VarBinary(column)
        | CommittableColumn::Decimal75(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
        CommittableColumn::Boolean(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::TimestampTZ(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
    }
}

pub(super) fn compute_dynamic_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &ProverSetup,
) -> Vec<DynamicDoryCommitment> {
    if_rayon!(committable_columns.par_iter(), committable_columns.iter())
        .map(|column| {
            column
                .is_empty()
                .then(|| DynamicDoryCommitment(GT::zero()))
                .unwrap_or_else(|| compute_dory_commitment(column, offset, setup))
        })
        .collect()
}
