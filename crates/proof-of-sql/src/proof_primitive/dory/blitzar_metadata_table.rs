use super::{
    dynamic_dory_structure::{
        full_width_of_row, index_from_row_and_column, matrix_size, row_and_column_from_index,
    },
    G1Affine, F,
};
use crate::{
    base::{commitment::CommittableColumn, database::ColumnType, if_rayon},
    proof_primitive::dory::offset_to_bytes::OffsetToBytes,
};
use alloc::{vec, vec::Vec};
use ark_ec::CurveGroup;
use ark_ff::MontFp;
use ark_std::ops::Mul;
use core::iter;
use itertools::Itertools;
#[cfg(feature = "rayon")]
use rayon::{
    iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator},
    prelude::ParallelSliceMut,
};
use tracing::{span, Level};

const BYTE_SIZE: u32 = 8;

/// Returns the minimum value of a column as F.
///
/// # Arguments
///
/// * `column_type` - The type of a committable column.
pub const fn min_as_f(column_type: ColumnType) -> F {
    match column_type {
        ColumnType::TinyInt => MontFp!("-128"),
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
pub fn signed_commits(
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

    if_rayon!(
        unsigned_sub_commits
            .into_par_iter()
            .zip(min_sub_commits.into_par_iter())
            .map(|(signed, offset)| (signed + offset).into())
            .collect(),
        unsigned_sub_commits
            .into_iter()
            .zip(min_sub_commits.into_iter())
            .map(|(unsigned, min)| (unsigned + min).into())
            .collect()
    )
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

/// Populates a slice of the scalar array with committable column data
/// while adding the ones entry to handle signed data columns.
///
/// # Arguments
///
/// * `scalar_col` - The scalar column index.
/// * `scalar_row` - The scalar row index.
/// * `scalar_row_slice` - A mutable reference to the scalar row slice.
/// * `offset` - The offset to the data.
/// * `committable_columns` - A reference to the committable columns.
/// * `cumulative_byte_length_table` - A reference to the cumulative byte length table.
/// * `single_entry_in_blitzar_output_bit_table` - A reference to the single entry in the Blitzar output bit table.
/// * `ones_columns_lengths` - A reference to the ones columns lengths.
/// * `num_of_bytes_in_committable_columns` - The number of bytes in the committable columns.
#[allow(clippy::too_many_arguments)]
fn populate_scalar_array_with_column(
    scalar_col: usize,
    scalar_row: usize,
    scalar_row_slice: &mut [u8],
    offset: usize,
    committable_columns: &[CommittableColumn],
    cumulative_byte_length_table: &[usize],
    single_entry_in_blitzar_output_bit_table: &[u32],
    ones_columns_lengths: &[usize],
    num_of_bytes_in_committable_columns: usize,
) {
    if let Some(index) = index_from_row_and_column(scalar_col, scalar_row)
        .and_then(|committable_column_idx| committable_column_idx.checked_sub(offset))
    {
        for (i, committable_column) in committable_columns
            .iter()
            .enumerate()
            .filter(|(_, committable_column)| index < committable_column.len())
        {
            let start = cumulative_byte_length_table
                [i + scalar_col * single_entry_in_blitzar_output_bit_table.len()];
            let end = start + (single_entry_in_blitzar_output_bit_table[i] / BYTE_SIZE) as usize;

            copy_column_data_to_slice(committable_column, scalar_row_slice, start, end, index);
        }

        ones_columns_lengths
            .iter()
            .positions(|ones_columns_length| index < *ones_columns_length)
            .for_each(|i| {
                let ones_index = i
                    + scalar_col
                        * (num_of_bytes_in_committable_columns + ones_columns_lengths.len())
                    + num_of_bytes_in_committable_columns;

                scalar_row_slice[ones_index] = 1_u8;
            });
    }
}

/// Creates the metadata tables for Blitzar's `vlen_msm` algorithm.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
///
/// # Returns
///
/// A tuple containing the output bit table, output length table,
/// and scalars required to call Blitzar's `vlen_msm` function.
#[tracing::instrument(name = "create_blitzar_metadata_tables", level = "debug", skip_all)]
pub fn create_blitzar_metadata_tables(
    committable_columns: &[CommittableColumn],
    offset: usize,
) -> (Vec<u32>, Vec<u32>, Vec<u8>) {
    // Keep track of the lengths of the columns to handled signed data columns.
    let ones_columns_lengths = committable_columns
        .iter()
        .map(CommittableColumn::len)
        .collect_vec();

    // The maximum matrix size will be used to create the scalars vector.
    let (max_height, max_width) = if let Some(max_column_len) =
        committable_columns.iter().map(CommittableColumn::len).max()
    {
        matrix_size(max_column_len, offset)
    } else {
        (0, 0)
    };

    // We will ignore the rows that are zero from the offsets.
    let (offset_row, _) = row_and_column_from_index(offset);
    let offset_height = max_height - offset_row;

    // Find the single packed byte size of all committable columns.
    let num_of_bytes_in_committable_columns: usize = committable_columns
        .iter()
        .map(|column| column.column_type().byte_size())
        .sum();

    // Get a single bit table entry with ones added for all committable columns that are signed.
    let single_entry_in_blitzar_output_bit_table: Vec<u32> = committable_columns
        .iter()
        .map(|column| column.column_type().bit_size())
        .chain(iter::repeat(BYTE_SIZE).take(ones_columns_lengths.len()))
        .collect();

    // Create the full bit table vector to be used by Blitzar's vlen_msm algorithm.
    let blitzar_output_bit_table: Vec<u32> = single_entry_in_blitzar_output_bit_table
        .iter()
        .copied()
        .cycle()
        .take(single_entry_in_blitzar_output_bit_table.len() * offset_height)
        .collect();

    // Create the full length vector to be used by Blitzar's vlen_msm algorithm.
    let blitzar_output_length_table: Vec<u32> = (0..blitzar_output_bit_table.len()
        / single_entry_in_blitzar_output_bit_table.len())
        .flat_map(|i| {
            itertools::repeat_n(
                u32::try_from(full_width_of_row(i)),
                single_entry_in_blitzar_output_bit_table.len(),
            )
        })
        .flatten()
        .collect();

    // Create a cumulative length table to be used when packing the scalar vector.
    let cumulative_byte_length_table: Vec<usize> = iter::once(0)
        .chain(blitzar_output_bit_table.iter().scan(0usize, |acc, &x| {
            *acc += (x / BYTE_SIZE) as usize;
            Some(*acc)
        }))
        .collect();

    // Create scalars array. Note, scalars need to be stored in a column-major order.
    let num_scalar_rows = max_width;
    let num_scalar_columns =
        (num_of_bytes_in_committable_columns + ones_columns_lengths.len()) * offset_height;
    let mut blitzar_scalars = vec![0u8; num_scalar_rows * num_scalar_columns];

    // Populate the scalars array.
    let span = span!(Level::INFO, "pack_blitzar_scalars").entered();
    if !blitzar_scalars.is_empty() {
        if_rayon!(
            blitzar_scalars
                .par_chunks_exact_mut(num_scalar_columns)
                .enumerate()
                .for_each(|(scalar_row, scalar_row_slice)| {
                    for scalar_col in 0..max_height {
                        populate_scalar_array_with_column(
                            scalar_col,
                            scalar_row,
                            scalar_row_slice,
                            offset,
                            committable_columns,
                            &cumulative_byte_length_table,
                            &single_entry_in_blitzar_output_bit_table,
                            &ones_columns_lengths,
                            num_of_bytes_in_committable_columns,
                        );
                    }
                }),
            blitzar_scalars
                .chunks_exact_mut(num_scalar_columns)
                .enumerate()
                .for_each(|(scalar_row, scalar_row_slice)| {
                    for scalar_col in 0..max_height {
                        populate_scalar_array_with_column(
                            scalar_col,
                            scalar_row,
                            scalar_row_slice,
                            offset,
                            committable_columns,
                            &cumulative_byte_length_table,
                            &single_entry_in_blitzar_output_bit_table,
                            &ones_columns_lengths,
                            num_of_bytes_in_committable_columns,
                        );
                    }
                })
        );
    }
    span.exit();

    (
        blitzar_output_bit_table,
        blitzar_output_length_table,
        blitzar_scalars,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::math::decimal::Precision;
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    fn assert_blitzar_metadata(
        committable_columns: &[CommittableColumn],
        offset: usize,
        expected_bit_table: &[u32],
        expected_length_table: &[u32],
        expected_scalars: &[u8],
    ) {
        let (bit_table, length_table, scalars) =
            create_blitzar_metadata_tables(committable_columns, offset);

        assert_eq!(
            bit_table, expected_bit_table,
            "Bit table mismatch for offset {offset}"
        );
        assert_eq!(
            length_table, expected_length_table,
            "Length table mismatch for offset {offset}"
        );
        assert_eq!(
            scalars, expected_scalars,
            "Scalars mismatch for offset {offset}"
        );
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_empty_columns_and_offset_that_fills_row() {
        let committable_columns = [CommittableColumn::BigInt(&[0; 0])];
        let offsets = vec![
            0, 1, 2, 4, 8, 12, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128,
        ];
        for &offset in &offsets {
            assert_blitzar_metadata(&committable_columns, offset, &[], &[], &[]);
        }
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_empty_columns_and_offset_that_does_not_fill_row(
    ) {
        let committable_columns = [CommittableColumn::BigInt(&[0; 0])];

        let offset = 3;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &[64, 8],
            &[2, 2],
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        );

        let offset = 5;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &[64, 8],
            &[4, 4],
            &[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
            ],
        );

        let offset = 17;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &[64, 8],
            &[8, 8],
            &[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        );

        let offset = 65;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &[64, 8],
            &[16, 16],
            &[
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
        );
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_simple_column() {
        let committable_columns = [CommittableColumn::BigInt(&[1])];

        let offset = 0;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &[64, 8],
            &[1, 1],
            &[1, 0, 0, 0, 0, 0, 0, 128, 1],
        );
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_simple_column_and_offset() {
        let committable_columns = [CommittableColumn::BigInt(&[1])];

        let offset = 1;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &[64, 8],
            &[2, 2],
            &[0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 128, 1],
        );
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_simple_column_and_non_trivial_offsets() {
        let committable_columns = [CommittableColumn::TinyInt(&[1])];

        let expected_bit_table = vec![8, 8];

        let offset = 0;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &expected_bit_table,
            &[1, 1],
            &[129, 1],
        );

        let offset = 1;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &expected_bit_table,
            &[2, 2],
            &[0, 0, 129, 1],
        );

        let offset = 2;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &expected_bit_table,
            &[2, 2],
            &[129, 1, 0, 0],
        );

        let offset = 3;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &expected_bit_table,
            &[2, 2],
            &[0, 0, 129, 1],
        );
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_simple_column_and_offsets_with_4_columns() {
        let committable_columns = [CommittableColumn::TinyInt(&[1])];

        let expected_bit_table = vec![8, 8];
        let expected_length_table = vec![4, 4];

        let offsets = vec![4, 8, 12];
        for &offset in &offsets {
            assert_blitzar_metadata(
                &committable_columns,
                offset,
                &expected_bit_table,
                &expected_length_table,
                &[129, 1, 0, 0, 0, 0, 0, 0],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 1,
                &expected_bit_table,
                &expected_length_table,
                &[0, 0, 129, 1, 0, 0, 0, 0],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 2,
                &expected_bit_table,
                &expected_length_table,
                &[0, 0, 0, 0, 129, 1, 0, 0],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 3,
                &expected_bit_table,
                &expected_length_table,
                &[0, 0, 0, 0, 0, 0, 129, 1],
            );
        }
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_simple_column_and_offsets_with_8_columns() {
        let committable_columns = [CommittableColumn::TinyInt(&[1])];

        let expected_bit_table = vec![8, 8];
        let expected_length_table = vec![8, 8];

        let offsets = vec![16, 24, 32, 40, 48, 56];
        for &offset in &offsets {
            assert_blitzar_metadata(
                &committable_columns,
                offset,
                &expected_bit_table,
                &expected_length_table,
                &[129, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 1,
                &expected_bit_table,
                &expected_length_table,
                &[0, 0, 129, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 2,
                &expected_bit_table,
                &expected_length_table,
                &[0, 0, 0, 0, 129, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 3,
                &expected_bit_table,
                &expected_length_table,
                &[0, 0, 0, 0, 0, 0, 129, 1, 0, 0, 0, 0, 0, 0, 0, 0],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 4,
                &expected_bit_table,
                &expected_length_table,
                &[0, 0, 0, 0, 0, 0, 0, 0, 129, 1, 0, 0, 0, 0, 0, 0],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 5,
                &expected_bit_table,
                &expected_length_table,
                &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 129, 1, 0, 0, 0, 0],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 6,
                &expected_bit_table,
                &expected_length_table,
                &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 129, 1, 0, 0],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 7,
                &expected_bit_table,
                &expected_length_table,
                &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 129, 1],
            );
        }
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_simple_column_and_offsets_with_16_columns() {
        let committable_columns = [CommittableColumn::TinyInt(&[1])];

        let expected_bit_table = vec![8, 8];
        let expected_length_table = vec![16, 16];

        let offsets = vec![64, 80, 96, 112];
        for &offset in &offsets {
            assert_blitzar_metadata(
                &committable_columns,
                offset,
                &expected_bit_table,
                &expected_length_table,
                &[
                    129, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 8,
                &expected_bit_table,
                &expected_length_table,
                &[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 129, 1, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0,
                ],
            );

            assert_blitzar_metadata(
                &committable_columns,
                offset + 15,
                &expected_bit_table,
                &expected_length_table,
                &[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 129, 1,
                ],
            );
        }
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_mixed_columns() {
        let committable_columns = [
            CommittableColumn::TinyInt(&[1]),
            CommittableColumn::SmallInt(&[2]),
            CommittableColumn::Int(&[3]),
            CommittableColumn::BigInt(&[4]),
            CommittableColumn::Int128(&[5]),
            CommittableColumn::Decimal75(Precision::new(1).unwrap(), 0, vec![[6, 0, 0, 0]]),
            CommittableColumn::Scalar(vec![[7, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[8, 0, 0, 0]]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[9]),
            CommittableColumn::Boolean(&[true]),
        ];

        let offset = 0;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &[
                8, 16, 32, 64, 128, 256, 256, 256, 64, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            ],
            &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            &[
                129, 2, 128, 3, 0, 0, 128, 4, 0, 0, 0, 0, 0, 0, 128, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 128, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 128,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            ],
        );
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_mixed_columns_and_partial_column_offset() {
        let committable_columns = [
            CommittableColumn::TinyInt(&[1]),
            CommittableColumn::SmallInt(&[2]),
            CommittableColumn::Int(&[3]),
            CommittableColumn::BigInt(&[4]),
            CommittableColumn::Int128(&[5]),
            CommittableColumn::Decimal75(Precision::new(1).unwrap(), 0, vec![[6, 0, 0, 0]]),
            CommittableColumn::Scalar(vec![[7, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[8, 0, 0, 0]]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[9]),
            CommittableColumn::Boolean(&[true]),
        ];

        let offsets = vec![1, 3];
        for &offset in &offsets {
            assert_blitzar_metadata(
                &committable_columns,
                offset,
                &[
                    8, 16, 32, 64, 128, 256, 256, 256, 64, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
                ],
                &[2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
                &[
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 129, 2, 128, 3, 0, 0, 128, 4,
                    0, 0, 0, 0, 0, 0, 128, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 6, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 128, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1,
                ],
            );
        }
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_mixed_columns_and_full_column_offset() {
        let committable_columns = [
            CommittableColumn::TinyInt(&[1]),
            CommittableColumn::SmallInt(&[2]),
            CommittableColumn::Int(&[3]),
            CommittableColumn::BigInt(&[4]),
            CommittableColumn::Int128(&[5]),
            CommittableColumn::Decimal75(Precision::new(1).unwrap(), 0, vec![[6, 0, 0, 0]]),
            CommittableColumn::Scalar(vec![[7, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[8, 0, 0, 0]]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[9]),
            CommittableColumn::Boolean(&[true]),
        ];

        let offset = 2;
        assert_blitzar_metadata(
            &committable_columns,
            offset,
            &[
                8, 16, 32, 64, 128, 256, 256, 256, 64, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            ],
            &[2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2],
            &[
                129, 2, 128, 3, 0, 0, 128, 4, 0, 0, 0, 0, 0, 0, 128, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 128, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 128,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        );
    }
}
