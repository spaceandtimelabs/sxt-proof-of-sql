use super::{G1Affine, F};
use crate::{
    base::{commitment::CommittableColumn, database::ColumnType, if_rayon},
    proof_primitive::{
        dory::offset_to_bytes::OffsetToBytes,
        dynamic_matrix_utils::matrix_structure::{
            full_width_of_row, index_from_row_and_column, matrix_size, row_and_column_from_index,
        },
    },
    utils::log,
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
    prelude::{ParallelSlice, ParallelSliceMut},
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
        | ColumnType::Uint8
        | ColumnType::FixedSizeBinary(_)
        | ColumnType::Scalar
        | ColumnType::VarChar
        | ColumnType::VarBinary
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
    if committable_columns.is_empty() {
        return vec![];
    }

    log::log_memory_usage("Start");

    let res = if_rayon!(
        all_sub_commits.par_chunks_exact(committable_columns.len() * 2),
        all_sub_commits.chunks_exact(committable_columns.len() * 2)
    )
    .flat_map(|chunk| {
        let (first_half, second_half) = chunk.split_at(committable_columns.len());

        if_rayon!(
            first_half.into_par_iter().zip(second_half.into_par_iter()),
            first_half.iter().zip(second_half.iter())
        )
        .enumerate()
        .map(|(i, (first, second))| {
            let min = min_as_f(committable_columns[i].column_type());
            let combined = *first + second.mul(min);
            combined.into_affine()
        })
        .collect::<Vec<_>>()
    })
    .collect();

    log::log_memory_usage("End");

    res
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
        CommittableColumn::Uint8(column) => {
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
        | CommittableColumn::VarChar(column)
        | CommittableColumn::VarBinary(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());
        }
        CommittableColumn::FixedSizeBinary(bw, items) => {
            let width = usize::from(*bw);
            scalar_row_slice[start..end].copy_from_slice(&items[index * width..][..width]);
        }
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
///
/// # Panics
///
/// Panics if the row of a column exceeds `u32::MAX`.
#[tracing::instrument(name = "create_blitzar_metadata_tables", level = "debug", skip_all)]
pub fn create_blitzar_metadata_tables(
    committable_columns: &[CommittableColumn],
    offset: usize,
) -> (Vec<u32>, Vec<u32>, Vec<u8>) {
    if committable_columns.is_empty() {
        return (vec![], vec![], vec![]);
    }

    log::log_memory_usage("Start");

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
                u32::try_from(full_width_of_row(i + offset_row))
                    .expect("row lengths should never exceed u32::MAX"),
                single_entry_in_blitzar_output_bit_table.len(),
            )
        })
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
    let span = span!(Level::DEBUG, "pack_blitzar_scalars").entered();
    if !blitzar_scalars.is_empty() {
        if_rayon!(
            blitzar_scalars.par_chunks_exact_mut(num_scalar_columns),
            blitzar_scalars.chunks_exact_mut(num_scalar_columns)
        )
        .enumerate()
        .for_each(|(scalar_row, scalar_row_slice)| {
            for scalar_col in 0..offset_height {
                // Find index in the committable columns. Note, the scalar is in
                // column major order, that is why the (row, col) arguments are flipped.
                if let Some(index) = index_from_row_and_column(scalar_col + offset_row, scalar_row)
                    .and_then(|committable_column_idx| committable_column_idx.checked_sub(offset))
                {
                    for (i, committable_column) in committable_columns
                        .iter()
                        .enumerate()
                        .filter(|(_, committable_column)| index < committable_column.len())
                    {
                        let start = cumulative_byte_length_table
                            [i + scalar_col * single_entry_in_blitzar_output_bit_table.len()];
                        let end = start
                            + (single_entry_in_blitzar_output_bit_table[i] / BYTE_SIZE) as usize;

                        copy_column_data_to_slice(
                            committable_column,
                            scalar_row_slice,
                            start,
                            end,
                            index,
                        );
                    }

                    ones_columns_lengths
                        .iter()
                        .positions(|ones_columns_length| index < *ones_columns_length)
                        .for_each(|i| {
                            let ones_index = i
                                + scalar_col
                                    * (num_of_bytes_in_committable_columns
                                        + ones_columns_lengths.len())
                                + num_of_bytes_in_committable_columns;

                            scalar_row_slice[ones_index] = 1_u8;
                        });
                }
            }
        });
    }
    span.exit();

    log::log_memory_usage("End");

    (
        blitzar_output_bit_table,
        blitzar_output_length_table,
        blitzar_scalars,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::{
        math::decimal::Precision,
        posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
    };

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
    fn we_can_handle_empty_committable_columns_in_blitzar_metadata_tables() {
        assert_blitzar_metadata(&[], 0, &[], &[], &[]);
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
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[9]),
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
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[9]),
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
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::utc(), &[9]),
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
