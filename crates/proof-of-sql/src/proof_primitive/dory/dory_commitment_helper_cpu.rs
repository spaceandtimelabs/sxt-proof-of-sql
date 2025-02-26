use super::{pairings, DoryCommitment, DoryProverPublicSetup, DoryScalar, G1Projective};
use crate::{base::commitment::CommittableColumn, utils::log};
use alloc::vec::Vec;
use ark_ec::VariableBaseMSM;
use core::iter::once;

#[tracing::instrument(name = "compute_dory_commitment_impl (cpu)", level = "debug", skip_all)]
/// # Panics
///
/// Will panic if:
/// - `Gamma_1.last()` returns `None` when computing the first row commitment.
/// - `Gamma_1.last()` returns `None` when computing remaining row commitments.
/// - `Gamma_2.last()` returns `None` when computing the commitment for the entire matrix.
/// - The slices accessed in `Gamma_1.last().unwrap()` or `Gamma_2.last().unwrap()` are out of bounds.
fn compute_dory_commitment_impl<'a, T>(
    column: &'a [T],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment
where
    &'a T: Into<DoryScalar>,
    T: Sync,
{
    log::log_memory_usage("Start");

    // Compute offsets for the matrix.
    let num_columns = 1 << setup.sigma();
    let first_row_offset = offset % num_columns;
    let rows_offset = offset / num_columns;
    let first_row_len = column.len().min(num_columns - first_row_offset);
    let remaining_elements_len = column.len() - first_row_len;
    let remaining_row_count = (remaining_elements_len + num_columns - 1) / num_columns;

    // Break column into rows.
    let (first_row, remaining_elements) = column.split_at(first_row_len);
    let remaining_rows = remaining_elements.chunks(num_columns);

    // Compute commitments for the rows.
    let first_row_commit = G1Projective::msm_unchecked(
        &setup.prover_setup().Gamma_1.last().unwrap()[first_row_offset..num_columns],
        &Vec::from_iter(first_row.iter().map(|s| s.into().0)),
    );
    let remaining_row_commits = remaining_rows.map(|row| {
        G1Projective::msm_unchecked(
            &setup.prover_setup().Gamma_1.last().unwrap()[..num_columns],
            &Vec::from_iter(row.iter().map(|s| s.into().0)),
        )
    });

    // Compute the commitment for the entire matrix.
    let res = DoryCommitment(pairings::multi_pairing(
        once(first_row_commit).chain(remaining_row_commits),
        &setup.prover_setup().Gamma_2.last().unwrap()
            [rows_offset..(rows_offset + remaining_row_count + 1)],
    ));

    log::log_memory_usage("End");

    res
}

fn compute_dory_commitment(
    committable_column: &CommittableColumn,
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment {
    match committable_column {
        CommittableColumn::Scalar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Uint8(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Uint16(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::TinyInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::SmallInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::BigInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int128(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Decimal75(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
        CommittableColumn::VarChar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::VarBinary(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Boolean(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::TimestampTZ(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
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
