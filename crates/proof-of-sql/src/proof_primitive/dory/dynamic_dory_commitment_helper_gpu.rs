use super::{
    dynamic_dory_structure::{matrix_size, row_and_column_from_index},
    pairings, DoryScalar, DynamicDoryCommitment, G1Affine, G1Projective, ProverSetup,
};
use crate::base::commitment::CommittableColumn;
use alloc::{vec, vec::Vec};

const BYTE_SIZE: u32 = 8;

/// Finds the single packed byte size of all committable columns.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
///
/// # Returns
///
/// The single packed byte size of all committable columns.
fn single_packed_byte_size(committable_columns: &[CommittableColumn]) -> usize {
    committable_columns
        .iter()
        .fold(0, |acc, x| acc + x.column_type().byte_size())
}

/// Returns the size of the matrix that can hold the longest committable column in a dynamic Dory structure.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
///
/// # Returns
///
/// A tuple containing the maximum (height, width) needed to store the longest committable column in
/// a dynamic Dory structure.
fn max_matrix_size(committable_columns: &[CommittableColumn], offset: usize) -> (usize, usize) {
    committable_columns
        .iter()
        .map(|column| matrix_size(column.len(), offset))
        .fold((0, 0), |(acc_height, acc_width), (height, width)| {
            (acc_height.max(height), acc_width.max(width))
        })
}

#[tracing::instrument(name = "compute_dory_commitment_impl (cpu)", level = "debug", skip_all)]
/// # Panics
///
/// Will panic if:
/// - `setup.Gamma_1.last()` returns `None`, indicating that `Gamma_1` is empty.
/// - `setup.Gamma_2.last()` returns `None`, indicating that `Gamma_2` is empty.
/// - The indexing for `Gamma_2` with `first_row..=last_row` goes out of bounds.
fn compute_dory_commitment_impl<'a, T>(
    column: &'a [T],
    offset: usize,
    setup: &ProverSetup,
) -> DynamicDoryCommitment
where
    &'a T: Into<DoryScalar>,
    T: Sync,
{
    let Gamma_1 = setup.Gamma_1.last().unwrap();
    let Gamma_2 = setup.Gamma_2.last().unwrap();
    let (first_row, _) = row_and_column_from_index(offset);
    let (last_row, _) = row_and_column_from_index(offset + column.len() - 1);
    let row_commits = column.iter().enumerate().fold(
        vec![G1Projective::from(G1Affine::identity()); last_row - first_row + 1],
        |mut row_commits, (i, v)| {
            let (row, col) = row_and_column_from_index(i + offset);
            row_commits[row - first_row] += Gamma_1[col] * v.into().0;
            row_commits
        },
    );
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
        CommittableColumn::TinyInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::SmallInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::BigInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int128(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::VarChar(column) | CommittableColumn::Decimal75(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
        CommittableColumn::Boolean(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::TimestampTZ(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
        CommittableColumn::RangeCheckWord(column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
    }
}

pub(super) fn compute_dynamic_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &ProverSetup,
) -> Vec<DynamicDoryCommitment> {
    committable_columns
        .iter()
        .map(|column| compute_dory_commitment(column, offset, setup))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::math::decimal::Precision;
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    #[test]
    fn we_can_get_a_single_packed_bit_size() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1]),
            CommittableColumn::BigInt(&[1, 2, 3]),
        ];

        let single_packed_byte_size = single_packed_byte_size(&committable_columns);
        let full_byte_size = ((64 + 64) / BYTE_SIZE) as usize;
        assert_eq!(single_packed_byte_size, full_byte_size);
    }

    #[test]
    fn we_can_get_a_single_packed_bit_size_with_mixed_columns() {
        let committable_columns = [
            CommittableColumn::Boolean(&[true, false]),
            CommittableColumn::TinyInt(&[1, 2, 3]),
            CommittableColumn::SmallInt(&[1]),
            CommittableColumn::Int(&[1, 2]),
            CommittableColumn::Int128(&[1, 2, 3, 4]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0], [4, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1]),
        ];

        let single_packed_byte_size = single_packed_byte_size(&committable_columns);
        let full_byte_size = (8 + 16 + 32 + 64 + 128 + 256 + 256 + 256 + 8 + 64) / 8;
        assert_eq!(single_packed_byte_size, full_byte_size);
    }

    #[test]
    fn we_can_get_max_matrix_size() {
        let committable_columns = [
            CommittableColumn::BigInt(&[0]),
            CommittableColumn::BigInt(&[0, 1, 2, 3]),
        ];

        let offset = 0;
        assert_eq!(max_matrix_size(&committable_columns, offset), (3, 2));
    }

    #[test]
    fn we_can_get_max_matrix_size_mixed_columns() {
        let committable_columns = [
            CommittableColumn::TinyInt(&[0]),
            CommittableColumn::SmallInt(&[0, 1]),
            CommittableColumn::Int(&[0, 1, 2]),
            CommittableColumn::BigInt(&[0, 1, 2, 3]),
            CommittableColumn::Int128(&[0, 1, 2, 3, 4]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [0, 0, 0, 0],
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![
                [0, 0, 0, 0],
                [1, 0, 0, 0],
                [2, 0, 0, 0],
                [3, 0, 0, 0],
                [4, 0, 0, 0],
            ]),
            CommittableColumn::VarChar(vec![
                [0, 0, 0, 0],
                [1, 0, 0, 0],
                [2, 0, 0, 0],
                [3, 0, 0, 0],
            ]),
            CommittableColumn::Boolean(&[true, false, true]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[0, 1]),
        ];

        let offset = 0;
        assert_eq!(max_matrix_size(&committable_columns, offset), (4, 4));
    }

    #[test]
    fn we_can_get_max_matrix_size_with_offset() {
        let committable_columns = [
            CommittableColumn::BigInt(&[0]),
            CommittableColumn::BigInt(&[0, 1, 2, 3]),
        ];

        let offset = 15;
        assert_eq!(max_matrix_size(&committable_columns, offset), (7, 8));
    }

    #[test]
    fn we_can_get_max_matrix_size_mixed_columns_with_offset() {
        let committable_columns = [
            CommittableColumn::TinyInt(&[0]),
            CommittableColumn::SmallInt(&[0, 1]),
            CommittableColumn::Int(&[0, 1, 2]),
            CommittableColumn::BigInt(&[0, 1, 2, 3]),
            CommittableColumn::Int128(&[0, 1, 2, 3, 4]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![
                    [0, 0, 0, 0],
                    [1, 0, 0, 0],
                    [2, 0, 0, 0],
                    [3, 0, 0, 0],
                    [4, 0, 0, 0],
                    [5, 0, 0, 0],
                ],
            ),
            CommittableColumn::Scalar(vec![
                [0, 0, 0, 0],
                [1, 0, 0, 0],
                [2, 0, 0, 0],
                [3, 0, 0, 0],
                [4, 0, 0, 0],
            ]),
            CommittableColumn::VarChar(vec![
                [0, 0, 0, 0],
                [1, 0, 0, 0],
                [2, 0, 0, 0],
                [3, 0, 0, 0],
            ]),
            CommittableColumn::Boolean(&[true, false, true]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[0, 1]),
        ];

        let offset = 60;
        assert_eq!(max_matrix_size(&committable_columns, offset), (13, 16));
    }
}
