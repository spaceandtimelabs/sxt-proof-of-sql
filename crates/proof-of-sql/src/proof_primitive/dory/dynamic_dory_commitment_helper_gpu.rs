use super::{
    dynamic_dory_structure::{full_width_of_row, matrix_size, row_and_column_from_index},
    pairings, DoryScalar, DynamicDoryCommitment, G1Affine, G1Projective, ProverSetup,
};
use crate::{base::commitment::CommittableColumn, proof_primitive::dory::pack_scalars::min_as_f};
use alloc::{vec, vec::Vec};
use ark_ec::CurveGroup;
use ark_std::ops::Mul;
use rayon::prelude::*;

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

/// Returns a single element worth of bit values for the bit table with offsets needed to handle the
/// signed columns. Note, the signed bits are handled naively. For each committable column,
/// a signed bit offset entry is added to the bit table. This means every signed and unsigned
/// committable column will have an additional byte for the signed offset. Also, multiple redundant
/// commitments of `1`'s are calculated.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `signed_offset_length` - The length of the signed offset.
///
/// # Returns
///
/// A vector containing the bit sizes of each committable column with a corresponding offset for
/// a single entry in the bit table.
fn populate_single_bit_array_with_offsets(
    committable_columns: &[CommittableColumn],
    signed_offset_length: usize,
) -> Vec<u32> {
    let mut bit_sizes: Vec<u32> = committable_columns
        .iter()
        .map(|column| column.column_type().bit_size())
        .collect();

    bit_sizes.extend(std::iter::repeat(BYTE_SIZE).take(signed_offset_length));

    bit_sizes
}

/// Returns a bit table to be used by the `vlen_msm` algorithm in Blitzar.
///
/// # Arguments
///
/// * `single_bit_table_entry` - A reference to the single bit table entry.
/// * `max_height` - The maximum height of the dynamic Dory matrix.
///
/// # Returns
///
/// A vector containing the bit sizes needed by Blitzar's `vlen_msm` algorithm.
fn populate_bit_table(single_bit_table_entry: &[u32], max_height: usize) -> Vec<u32> {
    single_bit_table_entry
        .iter()
        .copied()
        .cycle()
        .take(single_bit_table_entry.len() * max_height)
        .collect()
}

/// Returns a bit table to be used by the `vlen_msm` algorithm in Blitzar.
///
/// # Arguments
///
/// * `bit_table_len` - The length of the bit table used to call Blitzar's `vlen_msm` algorithm.
/// * `single_bit_table_entry_len` - The length of a single bit table entry.
///
/// # Returns
///
/// A vector containing the length of entries from the dynamic Dory structure that
/// are being used in the commitment computation.
///
/// # Panics
///
/// Panics if `bit_table_len` is not a multiple of `single_bit_table_entry_len`.
fn populate_length_table(bit_table_len: usize, single_bit_table_entry_len: usize) -> Vec<u32> {
    assert!(
        bit_table_len % single_bit_table_entry_len == 0,
        "bit_table_len must be a multiple of single_bit_table_entry_len"
    );

    (0..bit_table_len / single_bit_table_entry_len)
        .flat_map(|i| {
            std::iter::repeat(full_width_of_row(i) as u32).take(single_bit_table_entry_len)
        })
        .collect()
}

/// Returns a cumulative byte length table to be used when packing the scalar vector.
///
/// # Arguments
///
/// * `bit_table` - A reference to the bit table.
///
/// # Returns
///
/// A vector containing the cumulative byte length of the bit table.
fn cumulative_byte_length_table(bit_table: &[u32]) -> Vec<usize> {
    std::iter::once(0)
        .chain(bit_table.iter().scan(0usize, |acc, &x| {
            *acc += (x / BYTE_SIZE) as usize;
            Some(*acc)
        }))
        .collect()
}

/// Modifies the sub commits by adding the signed offset to the signed sub commits.
///
/// # Arguments
///
/// * `all_sub_commits` - A reference to the sub commits.
/// * `committable_columns` - A reference to the committable columns.
///
/// # Returns
///
/// A vector containing the modified sub commits to be used by the dynamic Dory commitment computation.
#[tracing::instrument(name = "modify_commits", level = "debug", skip_all)]
fn modify_commits(
    all_sub_commits: &Vec<G1Affine>,
    committable_columns: &[CommittableColumn],
) -> Vec<G1Affine> {
    let mut signed_sub_commits: Vec<G1Affine> = Vec::new();
    let mut offset_sub_commits: Vec<G1Affine> = Vec::new();
    let mut counter = 0;

    // Every sub_commit has a corresponding offset sub_commit committable_columns.len() away.
    // The commits and respective signed offset commits are interleaved in the all_sub_commits vector.
    for commit in all_sub_commits {
        if counter < committable_columns.len() {
            signed_sub_commits.push(*commit);
        } else {
            let min =
                min_as_f(committable_columns[counter - committable_columns.len()].column_type());
            offset_sub_commits.push(commit.mul(min).into_affine());
        }
        counter += 1;
        if counter == 2 * committable_columns.len() {
            counter = 0;
        }
    }

    signed_sub_commits
        .into_par_iter()
        .zip(offset_sub_commits.into_par_iter())
        .map(|(signed, offset)| (signed + offset).into())
        .collect()
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

    #[test]
    fn we_can_populate_single_bit_array_with_offsets() {
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

        let signed_offset_length = committable_columns.len();
        let single_bit_table_entry =
            populate_single_bit_array_with_offsets(&committable_columns, signed_offset_length);
        assert_eq!(
            single_bit_table_entry,
            vec![8, 16, 32, 64, 128, 256, 256, 256, 8, 64, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8]
        );
    }

    #[test]
    fn we_can_populate_a_bit_table() {
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

        let max_height = 4;
        let signed_offset_length = committable_columns.len();
        let single_bit_table_entry =
            populate_single_bit_array_with_offsets(&committable_columns, signed_offset_length);
        let bit_table = populate_bit_table(&single_bit_table_entry, max_height);
        assert_eq!(
            bit_table,
            vec![
                8, 16, 32, 64, 128, 256, 256, 256, 8, 64, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 16, 32,
                64, 128, 256, 256, 256, 8, 64, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 16, 32, 64, 128,
                256, 256, 256, 8, 64, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 16, 32, 64, 128, 256, 256,
                256, 8, 64, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8
            ]
        );
    }

    #[test]
    fn we_can_populate_a_length_table() {
        let committable_column_len = 3;
        let signed_value_offset = committable_column_len;
        let num_of_rows = 7;

        let bit_table_len = (committable_column_len + signed_value_offset) * num_of_rows;
        let single_bit_table_entry_len = committable_column_len + signed_value_offset;

        let length_table = populate_length_table(bit_table_len, single_bit_table_entry_len);

        assert_eq!(
            length_table,
            vec![
                1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4, 8, 8, 8, 8, 8, 8
            ]
        );
    }

    #[test]
    fn we_can_create_a_cumulative_byte_table() {
        assert_eq!(
            cumulative_byte_length_table(&Vec::new()),
            vec![0],
            "Empty bit table returned incorrect value"
        );
        assert_eq!(
            cumulative_byte_length_table(&[8, 8, 8, 8, 8]),
            vec![0, 1, 2, 3, 4, 5],
            "Simple bit table returned incorrect value"
        );
        assert_eq!(
            cumulative_byte_length_table(&[256, 128, 64, 32, 16, 8]),
            vec![0, 32, 48, 56, 60, 62, 63],
            "Complex bit table returned incorrect value"
        );
    }
}
