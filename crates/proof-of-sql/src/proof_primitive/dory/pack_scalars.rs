use super::{G1Affine, F};
use crate::{
    base::{commitment::CommittableColumn, database::ColumnType},
    proof_primitive::dory::offset_to_bytes::OffsetToBytes,
};
use ark_ec::CurveGroup;
use ark_ff::MontFp;
use ark_std::ops::Mul;
use rayon::prelude::*;

const BYTE_SIZE: usize = 8;

/// Returns a bit table vector related to each of the committable columns data size.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
fn output_bit_table<'a>(
    committable_columns: &'a [CommittableColumn],
) -> impl Iterator<Item = u32> + 'a {
    committable_columns
        .iter()
        .map(|column| column.column_type().bit_size())
}

/// Returns the size of the largest committable column.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
fn max_committable_column_length(committable_columns: &[CommittableColumn]) -> usize {
    committable_columns
        .iter()
        .map(|column| column.len())
        .max()
        .unwrap_or(0)
}

/// Returns the minimum value of a column as F.
///
/// # Arguments
///
/// * `column_type` - The type of a committable column.
const fn min_as_f(column_type: ColumnType) -> F {
    match column_type {
        ColumnType::SmallInt => MontFp!("-32768"),
        ColumnType::Int => MontFp!("-2147483648"),
        ColumnType::BigInt | ColumnType::TimestampTZ(_, _) => MontFp!("-9223372036854775808"),
        ColumnType::Int128 => MontFp!("-170141183460469231731687303715884105728"),
        ColumnType::Decimal75(_, _)
        | ColumnType::Scalar
        | ColumnType::VarChar
        | ColumnType::Boolean => MontFp!("0"),
    }
}

/// Returns a repeated bit table vector that duplicated the
/// bit table for each element by the num_sub_commits_per_full_commit.
///
/// # Arguments
///
/// * `bit_table` - A iterable bit table.
/// * `num_sub_commits_per_full_commit` - The number of sub commitments needed for each full commit.
fn repeat_bit_table(
    bit_table: impl Iterator<Item = u32>,
    num_sub_commits_per_full_commit: usize,
) -> Vec<u32> {
    bit_table
        .flat_map(|value| itertools::repeat_n(value, num_sub_commits_per_full_commit))
        .collect()
}

/// Returns the number of sub commitments needed for
/// each full commitment in the packed_msm function.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
/// * `num_matrix_commitment_columns` - The number of generators used for msm.
pub fn sub_commits_per_full_commit(
    committable_columns: &[CommittableColumn],
    offset: usize,
    num_matrix_commitment_columns: usize,
) -> usize {
    // Committable columns may be different sizes, get the max size and add offset.
    let max_column_length = max_committable_column_length(committable_columns) + offset;

    // Number of scalar vectors that the size of the generators n.
    // Each scalar will be used to call the packed_msm function.
    (max_column_length + num_matrix_commitment_columns - 1) / num_matrix_commitment_columns
}

/// Modifies the signed matrix commitment columns by adding the offset to the matrix commitment columns.
///
/// # Arguments
///
/// * `sub_commits` - A reference to the signed sub-commits.
/// * `committable_columns` - A reference to the committable columns.
/// * `num_sub_commits_per_full_commit` - The number of sub-commits needed for
///                                       each full commit for the packed_msm function.
#[tracing::instrument(name = "pack_scalars::modify_commits (gpu)", level = "debug", skip_all)]
pub fn modify_commits(
    sub_commits: &[G1Affine],
    committable_columns: &[CommittableColumn],
    num_sub_commits_per_full_commit: usize,
) -> Vec<G1Affine> {
    let num_full_commits = committable_columns.len();
    assert_eq!(
        2 * num_full_commits * num_sub_commits_per_full_commit,
        sub_commits.len()
    );

    // Currently, the packed_scalars doubles the number of sub-commits to deal with
    // signed sub-commits. Sub-commit i is offset by the sub-commit at i + num_sub_commits_per_full_commit.
    // Spit the sub-commits into signed sub-commits and offset sub-commits.
    let num_signed_sub_commits = num_full_commits * num_sub_commits_per_full_commit;
    let (signed_sub_commits, offset_sub_commits) = sub_commits.split_at(num_signed_sub_commits);

    // Ensure the packed_scalars were split correctly
    assert_eq!(signed_sub_commits.len(), offset_sub_commits.len());

    // Add the offset sub-commits multiplied by the min value to the signed sub-commits
    signed_sub_commits
        .par_iter()
        .zip(offset_sub_commits.par_iter())
        .enumerate()
        .map(|(index, (first, second))| {
            let min = min_as_f(
                committable_columns[index / num_sub_commits_per_full_commit].column_type(),
            );
            let modified_second = second.mul(min).into_affine();
            *first + modified_second
        })
        .map(|point| point.into_affine())
        .collect::<Vec<_>>()
}

/// Packs bits of a committable column into the packed scalars array.
/// Will offset signed values by the minimum of the data type.
///
/// # Arguments
///
/// * `column` - A reference to the committable column to be packed.
/// * `packed_scalars` - A mutable reference to the array where the packed scalars will be stored.
/// * `current_bit_table_sum` - The current sum of the bit table up to the current sub commit.
/// * `offset` - The offset to the data.
/// * `current_byte_size` - The current byte size of the column.
/// * `bit_table_sum_in_bytes` - The full bit table size in bytes.
/// * `num_columns` - The number of columns in a matrix commitment.
fn pack_bit<const LEN: usize, T: OffsetToBytes<LEN>>(
    column: &[T],
    packed_scalars: &mut [u8],
    current_bit_table_sum: usize,
    offset: usize,
    current_byte_size: usize,
    bit_table_sum_in_bytes: usize,
    num_columns: usize,
) {
    let byte_offset = current_bit_table_sum / BYTE_SIZE;
    column.iter().enumerate().for_each(|(i, value)| {
        let row_offset = ((i + offset) % num_columns) * bit_table_sum_in_bytes;
        let col_offset = current_byte_size * ((i + offset) / num_columns);
        let offset_idx = row_offset + col_offset + byte_offset;

        packed_scalars[offset_idx..offset_idx + current_byte_size]
            .copy_from_slice(&value.offset_to_bytes()[..]);
    });
}

/// Packs the offset bits of a committable column into the packed scalars at the end of the array.
/// The offsets are 8-bit values used to handle the signed values.
///
/// # Arguments
///
/// * `column` -  A reference to a signed committable column that needs offsets calculated.
/// * `packed_scalars` - A mutable reference to the array where the packed scalars will be stored.
/// * `current_bit_table_sum` - The current sum of the bit table up to the current column.
/// * `offset` - The offset to the data.
/// * `bit_table_sum_in_bytes` - The full bit table size in bytes.
/// * `num_columns` - The number of columns in a matrix commitment.
fn pack_offset_bit<const LEN: usize, T: OffsetToBytes<LEN>>(
    column: &[T],
    packed_scalars: &mut [u8],
    current_bit_table_sum: usize,
    offset: usize,
    bit_table_sum_in_bytes: usize,
    num_columns: usize,
) {
    let byte_offset = current_bit_table_sum / BYTE_SIZE;
    column.iter().enumerate().for_each(|(i, _)| {
        let row_offset = ((i + offset) % num_columns) * bit_table_sum_in_bytes;
        let col_offset = (i + offset) / num_columns;
        let offset_idx = row_offset + col_offset + byte_offset;

        packed_scalars[offset_idx] = 1_u8;
    });
}

/// Returns the bit table and packed scalar array to be used in Blitzar's packed_msm function.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
/// * `num_columns` - The number of columns in a matrix commitment.
/// * `num_sub_commits_per_full_commit` - The number of sub commits needed for
///                                       each full commit for the packed_msm function.
///
/// # Example
///
/// ```ignore
/// let committable_columns = [
///     CommittableColumn::SmallInt(&[0, 1, 2]),
///     CommittableColumn::SmallInt(&[3, 4, 5, 6, 7]),
/// ];
/// let offset = 1;
/// let num_columns = 3;
///
/// let num_sub_commits_per_full_commit = sub_commits_per_full_commit(&committable_columns, offset, num_columns);
///
/// let (bit_table, packed_scalars) = bit_table_and_scalars_for_packed_msm(
///     &committable_columns,
///     offset,
///     num_columns,
///     num_sub_commits_per_full_commit,
/// );
///
/// assert_eq!(num_of_commits, 2);
/// assert_eq!(bit_table, [16, 16, 16, 16, 8, 8, 8, 8]);
/// assert_eq!(packed_scalars.len(), 36); // num_columns * bit_table_sum / BYTE_SIZE
/// assert_eq!(packed_scalars, [0,   0, 2, 128, 0,   0, 5, 128, 0, 1, 0, 1,
///                             0, 128, 0,   0, 3, 128, 6, 128, 1, 0, 1, 1,
///                             1, 128, 0,   0, 4, 128, 7, 128, 1, 0, 1, 1]);
/// ```
#[tracing::instrument(
    name = "pack_scalars::bit_table_and_scalars_for_packed_msm (gpu)",
    level = "debug",
    skip_all
)]
pub fn bit_table_and_scalars_for_packed_msm(
    committable_columns: &[CommittableColumn],
    offset: usize,
    num_columns: usize,
    num_sub_commits_per_full_commit: usize,
) -> (Vec<u32>, Vec<u8>) {
    // Get a bit table that represented each of the committable columns bit size.
    let bit_table_full_commits = output_bit_table(committable_columns);

    // Repeat the bit table to account for the appropriate number of sub commitments per full commit.
    let mut bit_table = repeat_bit_table(bit_table_full_commits, num_sub_commits_per_full_commit);
    let bit_table_sub_commits_sum = bit_table.iter().sum::<u32>() as usize;

    // Double the bit table to handle handle the BYTE_SIZE offsets.
    bit_table.extend(std::iter::repeat(BYTE_SIZE as u32).take(bit_table.len()));
    let bit_table_sum_in_bytes = bit_table.iter().sum::<u32>() as usize / BYTE_SIZE;

    // Create the packed_scalar vector.
    let mut packed_scalars = vec![0_u8; bit_table_sum_in_bytes * num_columns];

    // For each committable column, pack the data into the packed_scalar array.
    committable_columns
        .iter()
        .enumerate()
        .for_each(|(i, column)| {
            // Get the running sum of the bit table for the signed values.
            let current_bit_table_sum = if i > 0 {
                bit_table
                    .iter()
                    .take(i * num_sub_commits_per_full_commit)
                    .sum::<u32>() as usize
            } else {
                0
            };

            // Get the running sum of the bit table for the offsets.
            let bit_table_offset_sum =
                bit_table_sub_commits_sum + i * BYTE_SIZE * num_sub_commits_per_full_commit;

            // Get the byte size of the column of data.
            let byte_size = committable_columns[i].column_type().byte_size();

            // Pack the signed bits and offset bits into the packed_scalars array.
            match column {
                CommittableColumn::SmallInt(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                    pack_offset_bit(
                        column,
                        &mut packed_scalars,
                        bit_table_offset_sum,
                        offset,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Int(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                    pack_offset_bit(
                        column,
                        &mut packed_scalars,
                        bit_table_offset_sum,
                        offset,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::BigInt(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                    pack_offset_bit(
                        column,
                        &mut packed_scalars,
                        bit_table_offset_sum,
                        offset,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Int128(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                    pack_offset_bit(
                        column,
                        &mut packed_scalars,
                        bit_table_offset_sum,
                        offset,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::TimestampTZ(_, _, column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                    pack_offset_bit(
                        column,
                        &mut packed_scalars,
                        bit_table_offset_sum,
                        offset,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Boolean(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Decimal75(_, _, column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::Scalar(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
                CommittableColumn::VarChar(column) => {
                    pack_bit(
                        column,
                        &mut packed_scalars,
                        current_bit_table_sum,
                        offset,
                        byte_size,
                        bit_table_sum_in_bytes,
                        num_columns,
                    );
                }
            }
        });

    (bit_table, packed_scalars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::math::decimal::Precision;
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    #[test]
    fn we_can_get_max_committable_column_length_of_the_same_type() {
        let committable_columns = [
            CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::Scalar(vec![[1, 2, 3, 4]]),
        ];

        let max_column_length = max_committable_column_length(&committable_columns);
        assert_eq!(max_column_length, 2);
    }

    #[test]
    fn we_can_get_max_committable_column_length_of_different_types() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1, 2, 3]),
            CommittableColumn::Int(&[1, 2, 3]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3, 4, 5, 6]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![[1, 0, 0, 0], [2, 0, 0, 0]],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false, true]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1, 2, 3]),
        ];

        let max_column_length = max_committable_column_length(&committable_columns);
        assert_eq!(max_column_length, 6);
    }

    #[test]
    fn we_can_get_a_bit_table() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1, 2, 3]),
            CommittableColumn::Int(&[1, 2, 3]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![[1, 0, 0, 0], [2, 0, 0, 0]],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false, true]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1, 2, 3]),
        ];

        let bit_table: Vec<u32> = output_bit_table(&committable_columns).collect();
        let expected = [16, 32, 64, 128, 64 * 4, 64 * 4, 64 * 4, 8, 64];
        assert_eq!(bit_table, expected);
    }

    #[test]
    fn we_can_get_a_repeated_bit_table() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1, 2, 3]),
            CommittableColumn::Int(&[1, 2, 3]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![[1, 0, 0, 0], [2, 0, 0, 0]],
            ),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::Boolean(&[true, false, true]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1, 2, 3]),
        ];

        let bit_table = output_bit_table(&committable_columns);
        let repeated_bit_table = repeat_bit_table(bit_table, 3);
        let expected_bit_table = [
            16,
            16,
            16,
            32,
            32,
            32,
            64,
            64,
            64,
            128,
            128,
            128,
            64 * 4,
            64 * 4,
            64 * 4,
            64 * 4,
            64 * 4,
            64 * 4,
            64 * 4,
            64 * 4,
            64 * 4,
            8,
            8,
            8,
            64,
            64,
            64,
        ];
        assert_eq!(repeated_bit_table, expected_bit_table);
    }

    #[test]
    fn we_can_get_sub_commits_per_full_commit_with_less_rows_than_columns() {
        let committable_columns = [
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0]]),
        ];

        let offset = 0;
        let num_columns = 1 << 2;
        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 1);
    }

    #[test]
    fn we_can_get_sub_commits_per_full_commit_with_offset_and_less_rows_than_columns() {
        let committable_columns = [
            CommittableColumn::Scalar(vec![[1, 0, 0, 0], [2, 0, 0, 0]]),
            CommittableColumn::Scalar(vec![[1, 0, 0, 0]]),
        ];

        let offset = 5;
        let num_columns = 1 << 2;
        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 2);
    }

    #[test]
    fn we_can_get_sub_commits_per_full_commit_with_more_rows_than_generators() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]),
            CommittableColumn::Int(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
            CommittableColumn::SmallInt(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
        ];

        let offset = 0;
        let num_columns = 1 << 2;
        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 5);
    }

    #[test]
    fn we_can_get_sub_commits_per_full_commit_with_offset_and_more_rows_than_generators() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]),
            CommittableColumn::Int(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
            CommittableColumn::SmallInt(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
        ];

        let offset = 1;
        let num_columns = 1 << 2;
        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 6);
    }

    #[test]
    fn we_can_create_a_mixed_packed_scalar_with_more_rows_than_columns() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            ]),
            CommittableColumn::Int(&[
                19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
            ]),
            CommittableColumn::SmallInt(&[
                38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56,
            ]),
        ];

        let num_columns = 1 << 2;
        let offset = 0;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 5);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_bit_table = [
            16, 16, 16, 16, 16, 32, 32, 32, 32, 32, 16, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8,
        ];

        let expected_packed_scalar = [
            0, 128, 4, 128, 8, 128, 12, 128, 16, 128, 19, 0, 0, 128, 23, 0, 0, 128, 27, 0, 0, 128,
            31, 0, 0, 128, 35, 0, 0, 128, 38, 128, 42, 128, 46, 128, 50, 128, 54, 128, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 128, 5, 128, 9, 128, 13, 128, 17, 128, 20, 0, 0,
            128, 24, 0, 0, 128, 28, 0, 0, 128, 32, 0, 0, 128, 36, 0, 0, 128, 39, 128, 43, 128, 47,
            128, 51, 128, 55, 128, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 128, 6, 128, 10,
            128, 14, 128, 18, 128, 21, 0, 0, 128, 25, 0, 0, 128, 29, 0, 0, 128, 33, 0, 0, 128, 37,
            0, 0, 128, 40, 128, 44, 128, 48, 128, 52, 128, 56, 128, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 3, 128, 7, 128, 11, 128, 15, 128, 0, 0, 22, 0, 0, 128, 26, 0, 0, 128,
            30, 0, 0, 128, 34, 0, 0, 128, 0, 0, 0, 0, 41, 128, 45, 128, 49, 128, 53, 128, 0, 0, 1,
            1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0,
        ];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }

    #[test]
    fn we_can_create_a_mixed_packed_scalar_with_offset_and_same_num_of_rows_and_columns() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[0, 1, 2, 3]),
            CommittableColumn::Int(&[4, 5, 6, 7]),
            CommittableColumn::SmallInt(&[8, 9, 10, 11]),
        ];

        let num_columns = 1 << 2;
        let offset = 5;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 3);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_bit_table = [
            16, 16, 16, 32, 32, 32, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8,
        ];

        let expected_packed_scalar = [
            0, 0, 0, 0, 3, 128, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 128, 0, 0, 0, 0, 11, 128, 0, 0, 1,
            0, 0, 1, 0, 0, 1, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 4, 0, 0, 128, 0, 0, 0, 0, 0, 0, 8,
            128, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 128, 0, 0, 0, 0, 0, 0, 5, 0, 0, 128, 0,
            0, 0, 0, 0, 0, 9, 128, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 2, 128, 0, 0, 0, 0, 0, 0,
            6, 0, 0, 128, 0, 0, 0, 0, 0, 0, 10, 128, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0,
        ];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }

    #[test]
    fn we_can_pack_empty_scalars() {
        let committable_columns = [];

        let (bit_table, packed_scalar) =
            bit_table_and_scalars_for_packed_msm(&committable_columns, 0, 1, 0);

        assert!(bit_table.is_empty());
        assert!(packed_scalar.is_empty());
    }

    #[test]
    fn we_can_pack_scalars_with_one_full_row() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 0;
        let num_columns = 1 << 1;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 1);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_packed_scalar = [
            1, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0, 0, 0, 0, 128, 1, 1, 2, 0, 0, 0, 0, 0, 0, 128, 4,
            0, 0, 0, 0, 0, 0, 128, 1, 1,
        ];

        let expected_bit_table = [64, 64, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }

    #[test]
    fn we_can_pack_scalars_with_more_than_one_row() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 0;
        let num_columns = 1 << 0;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 2);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_packed_scalar = [
            1, 0, 0, 0, 0, 0, 0, 128, 2, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0, 0, 0, 0, 128, 4, 0, 0,
            0, 0, 0, 0, 128, 1, 1, 1, 1,
        ];

        let expected_bit_table = [64, 64, 64, 64, 8, 8, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }

    #[test]
    fn we_can_pack_scalars_with_one_full_row_with_offset() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 1;
        let num_columns = 1 << 1;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 2);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_packed_scalar = [
            0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
            0, 0, 0, 128, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
            0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0,
        ];

        let expected_bit_table = [64, 64, 64, 64, 8, 8, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }

    #[test]
    fn we_can_pack_scalars_with_offset_and_more_rows_than_columns() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let offset = 1;
        let num_columns = 1 << 0;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 3);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_packed_scalar = [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 128, 2, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0,
            0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 128, 4, 0, 0, 0, 0, 0, 0, 128, 0, 1, 1, 0, 1, 1,
        ];

        let expected_bit_table = [64, 64, 64, 64, 64, 64, 8, 8, 8, 8, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }

    #[test]
    fn we_can_create_a_mixed_packed_scalar_with_offset_and_more_rows_than_columns() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[0, 1, 2, 3, 4, 5]),
            CommittableColumn::Int(&[6, 7, 8, 9]),
            CommittableColumn::Scalar(vec![[10, 0, 0, 0], [11, 0, 0, 0], [12, 0, 0, 0]]),
        ];

        let offset = 0;
        let num_columns = 3;

        let num_sub_commits_per_full_commit =
            sub_commits_per_full_commit(&committable_columns, offset, num_columns);
        assert_eq!(num_sub_commits_per_full_commit, 2);

        let (bit_table, packed_scalar) = bit_table_and_scalars_for_packed_msm(
            &committable_columns,
            offset,
            num_columns,
            num_sub_commits_per_full_commit,
        );

        let expected_packed_scalar = [
            0, 128, 3, 128, 6, 0, 0, 128, 9, 0, 0, 128, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1,
            128, 4, 128, 7, 0, 0, 128, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 2, 128, 5,
            128, 8, 0, 0, 128, 0, 0, 0, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0,
        ];

        let expected_bit_table = [16, 16, 32, 32, 64 * 4, 64 * 4, 8, 8, 8, 8, 8, 8];

        assert_eq!(bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_packed_scalar);
    }
}
