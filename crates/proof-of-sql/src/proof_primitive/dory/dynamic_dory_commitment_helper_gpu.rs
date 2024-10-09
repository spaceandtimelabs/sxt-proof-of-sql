use super::{DynamicDoryCommitment, G1Affine, ProverSetup};
use crate::{
    base::commitment::CommittableColumn,
    proof_primitive::dory::{offset_to_bytes::OffsetToBytes, pack_scalars::min_as_f},
};
use ark_ec::CurveGroup;
use ark_std::ops::Mul;

/// Modifies the sub commits by adding the minimum commitment of the column type to the signed sub commits.
///
/// # Arguments
///
/// * `all_sub_commits` - A reference to the sub commits.
/// * `committable_columns` - A reference to the committable columns.
///
/// # Returns
///
/// A vector containing the modified sub commits to be used by the dynamic Dory commitment computation.
#[tracing::instrument(name = "signed_commits", level = "debug", skip_all)]
fn signed_commits(
    all_sub_commits: &Vec<G1Affine>,
    committable_columns: &[CommittableColumn],
) -> Vec<G1Affine> {
    let mut unsigned_sub_commits: Vec<G1Affine> = Vec::new();
    let mut min_sub_commits: Vec<G1Affine> = Vec::new();
    let mut counter = 0;

    // Every sub_commit has a corresponding offset sub_commit committable_columns.len() away.
    // The commits and respective ones commits are interleaved in the all_sub_commits vector.
    for commit in all_sub_commits {
        if counter < committable_columns.len() {
            unsigned_sub_commits.push(*commit);
        } else {
            let min =
                min_as_f(committable_columns[counter - committable_columns.len()].column_type());
            min_sub_commits.push(commit.mul(min).into_affine());
        }
        counter += 1;
        if counter == 2 * committable_columns.len() {
            counter = 0;
        }
    }

    unsigned_sub_commits
        .into_iter()
        .zip(min_sub_commits.into_iter())
        .map(|(unsigned, min)| (unsigned + min).into())
        .collect()
}

/// Copies the column data to the scalar row slice.
///
/// # Arguments
///
/// * `column` - A reference to the committable column.
/// * `scalar_row_slice` - A mutable reference to the scalar row slice.
/// * `start` - The start index of the slice.
/// * `end` - The end index of the slice.
/// * `index` - The index of the column.
fn copy_column_data_to_slice(
    column: &CommittableColumn,
    scalar_row_slice: &mut [u8],
    start: usize,
    end: usize,
    index: usize,
) {
    match column {
        CommittableColumn::Boolean(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());
        }
        CommittableColumn::TinyInt(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());
        }
        CommittableColumn::SmallInt(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());
        }
        CommittableColumn::Int(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());
        }
        CommittableColumn::BigInt(column) | CommittableColumn::TimestampTZ(_, _, column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());
        }
        CommittableColumn::Int128(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());
        }
        CommittableColumn::Scalar(column)
        | CommittableColumn::Decimal75(_, _, column)
        | CommittableColumn::VarChar(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());
        }
        CommittableColumn::RangeCheckWord(_) => todo!(),
    }
}

pub(super) fn compute_dynamic_dory_commitments(
    _committable_columns: &[CommittableColumn],
    _offset: usize,
    _setup: &ProverSetup,
) -> Vec<DynamicDoryCommitment> {
    todo!()
}
