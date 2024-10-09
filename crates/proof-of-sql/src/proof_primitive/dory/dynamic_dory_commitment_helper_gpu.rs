use super::{
    dynamic_dory_structure::{full_width_of_row, index_from_row_and_column, matrix_size},
    DynamicDoryCommitment, G1Affine, ProverSetup,
};
use crate::{
    base::commitment::CommittableColumn,
    proof_primitive::dory::{offset_to_bytes::OffsetToBytes, pack_scalars::min_as_f},
};
use ark_ec::CurveGroup;
use ark_std::ops::Mul;
use core::iter;
use itertools::Itertools;
use tracing::{span, Level};

const BYTE_SIZE: u32 = 8;

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
fn create_blitzar_metadata_tables(
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
        .take(single_entry_in_blitzar_output_bit_table.len() * max_height)
        .collect();

    // Create the full length vector to be used by Blitzar's vlen_msm algorithm.
    let blitzar_output_length_table: Vec<u32> = (0..blitzar_output_bit_table.len()
        / single_entry_in_blitzar_output_bit_table.len())
        .flat_map(|i| {
            itertools::repeat_n(
                full_width_of_row(i) as u32,
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
        (num_of_bytes_in_committable_columns + ones_columns_lengths.len()) * max_height;
    let mut blitzar_scalars = vec![0u8; num_scalar_rows * num_scalar_columns];

    // Populate the scalars array.
    let span = span!(Level::INFO, "pack_blitzar_scalars").entered();
    if !blitzar_scalars.is_empty() {
        blitzar_scalars
            .chunks_exact_mut(num_scalar_columns)
            .enumerate()
            .for_each(|(scalar_row, scalar_row_slice)| {
                // Iterate over the columns and populate the scalars array.
                for scalar_col in 0..max_height {
                    // Find index in the committable columns. Note, the scalar is in
                    // column major order, that is why the (row, col) arguments are flipped.
                    if let Some(index) = index_from_row_and_column(scalar_col, scalar_row).and_then(
                        |committable_column_idx| committable_column_idx.checked_sub(offset),
                    ) {
                        for (i, committable_column) in committable_columns
                            .iter()
                            .enumerate()
                            .filter(|(_, committable_column)| index < committable_column.len())
                        {
                            let start = cumulative_byte_length_table
                                [i + scalar_col * single_entry_in_blitzar_output_bit_table.len()];
                            let end = start
                                + (single_entry_in_blitzar_output_bit_table[i] / BYTE_SIZE)
                                    as usize;

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

    (
        blitzar_output_bit_table,
        blitzar_output_length_table,
        blitzar_scalars,
    )
}

pub(super) fn compute_dynamic_dory_commitments(
    _committable_columns: &[CommittableColumn],
    _offset: usize,
    _setup: &ProverSetup,
) -> Vec<DynamicDoryCommitment> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::math::decimal::Precision;
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_empty_columns() {
        let committable_columns = [CommittableColumn::BigInt(&[0; 0])];
        let offset = 0;
        let (bit_table, length_table, scalars) =
            create_blitzar_metadata_tables(&committable_columns, offset);

        assert!(bit_table.is_empty());
        assert!(length_table.is_empty());
        assert!(scalars.is_empty());
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_empty_columns_and_an_offset() {
        let committable_columns = [CommittableColumn::BigInt(&[0; 0])];
        let offset = 1;
        let (bit_table, length_table, scalars) =
            create_blitzar_metadata_tables(&committable_columns, offset);

        assert_eq!(bit_table, vec![64, 8]);
        assert_eq!(length_table, vec![1, 1]);
        assert_eq!(scalars, vec![0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_simple_column() {
        let committable_columns = [CommittableColumn::BigInt(&[1])];
        let offset = 0;
        let (bit_table, length_table, scalars) =
            create_blitzar_metadata_tables(&committable_columns, offset);

        assert_eq!(bit_table, vec![64, 8]);
        assert_eq!(length_table, vec![1, 1]);
        assert_eq!(scalars, vec![1, 0, 0, 0, 0, 0, 0, 128, 1]);
    }

    #[test]
    fn we_can_populate_blitzar_metadata_tables_with_simple_column_and_offset() {
        let committable_columns = [CommittableColumn::BigInt(&[1])];
        let offset = 1;
        let (bit_table, length_table, scalars) =
            create_blitzar_metadata_tables(&committable_columns, offset);

        assert_eq!(bit_table, vec![64, 8, 64, 8]);
        assert_eq!(length_table, vec![1, 1, 2, 2]);
        assert_eq!(
            scalars,
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                0, 0, 0, 0, 0, 0, 128, 1
            ]
        );
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
        let (bit_table, length_table, scalars) =
            create_blitzar_metadata_tables(&committable_columns, offset);
        assert_eq!(
            bit_table,
            vec![8, 16, 32, 64, 128, 256, 256, 256, 64, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8]
        );

        assert_eq!(
            length_table,
            vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
        );
        assert_eq!(
            scalars,
            vec![
                129, 2, 128, 3, 0, 0, 128, 4, 0, 0, 0, 0, 0, 0, 128, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 128, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 128,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1
            ]
        );
    }
}
