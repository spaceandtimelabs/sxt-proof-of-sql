use super::{
    dynamic_dory_structure::{full_width_of_row, index_from_row_and_column, matrix_size},
    pairings, DynamicDoryCommitment, G1Affine, ProverSetup,
};
use crate::{
    base::{commitment::CommittableColumn, slice_ops::slice_cast},
    proof_primitive::dory::{offset_to_bytes::OffsetToBytes, pack_scalars::min_as_f},
};
use ark_ec::CurveGroup;
use ark_std::ops::Mul;
use blitzar::compute::ElementP2;
use core::iter;
use tracing::{span, Level};

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
        .map(|column| column.column_type().byte_size())
        .sum()
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
///
/// # Panics
///
/// Panics if the committable columns are empty.
fn max_matrix_size(committable_columns: &[CommittableColumn], offset: usize) -> (usize, usize) {
    let max_column_len = committable_columns
        .iter()
        .map(CommittableColumn::len)
        .max()
        .unwrap();
    matrix_size(max_column_len, offset)
}

/// Returns a single element worth of bit values for the bit table with entries needed to handle the
/// signed columns. Note, the signed bits are handled naively. Each committable column will have an
/// additional byte to handle signed values, regardless if the committable column is singed or unsigned.
/// Additionally, multiple redundant sub commitments of `1`'s are calculated.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `signed_ones_length` - The length of the ones entry to handle signed columns.
///
/// # Returns
///
/// A vector containing the bit sizes of each committable column with a corresponding entry for
/// handling signed values in the bit table.
fn populate_single_bit_array_with_ones(
    committable_columns: &[CommittableColumn],
    signed_ones_length: usize,
) -> Vec<u32> {
    committable_columns
        .iter()
        .map(|column| column.column_type().bit_size())
        .chain(iter::repeat(BYTE_SIZE).take(signed_ones_length))
        .collect()
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
/// * `ones_index` - The index of the one used for signed columns.
fn copy_column_data_to_slice(
    column: &CommittableColumn,
    scalar_row_slice: &mut [u8],
    start: usize,
    end: usize,
    index: usize,
    ones_index: usize,
) {
    match column {
        CommittableColumn::Boolean(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());
        }
        CommittableColumn::TinyInt(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());

            scalar_row_slice[ones_index] = 1_u8;
        }
        CommittableColumn::SmallInt(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());

            scalar_row_slice[ones_index] = 1_u8;
        }
        CommittableColumn::Int(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());

            scalar_row_slice[ones_index] = 1_u8;
        }
        CommittableColumn::BigInt(column) | CommittableColumn::TimestampTZ(_, _, column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());

            scalar_row_slice[ones_index] = 1_u8;
        }
        CommittableColumn::Int128(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());

            scalar_row_slice[ones_index] = 1_u8;
        }
        CommittableColumn::Scalar(column)
        | CommittableColumn::Decimal75(_, _, column)
        | CommittableColumn::VarChar(column) => {
            scalar_row_slice[start..end].copy_from_slice(&column[index].offset_to_bytes());
        }
        CommittableColumn::RangeCheckWord(_) => todo!(),
    }
}

/// Computes the dynamic Dory commitment using the GPU implementation of the `vlen_msm` algorithm.
///
/// # Arguments
///
/// * `committable_columns` - A reference to the committable columns.
/// * `offset` - The offset to the data.
/// * `setup` - A reference to the prover setup.
///
/// # Returns
///
/// A vector containing the dynamic Dory commitments.
///
/// # Panics
///
/// Panics if the number of sub commits is not a multiple of the number of committable columns.
#[tracing::instrument(
    name = "compute_dynamic_dory_commitments (gpu)",
    level = "debug",
    skip_all
)]
pub(super) fn compute_dynamic_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &ProverSetup,
) -> Vec<DynamicDoryCommitment> {
    let Gamma_2 = setup.Gamma_2.last().unwrap();

    // The maximum matrix size will be used to create the scalars vector.
    let (max_height, max_width) = max_matrix_size(committable_columns, offset);

    // Find the single packed byte size of all committable columns.
    let single_packed_byte_size = single_packed_byte_size(committable_columns);

    // Get a single bit table entry with ones added for all committable columns that are signed.
    let signed_ones_length = committable_columns.len();
    let single_packed_byte_with_ones_size = single_packed_byte_size + signed_ones_length;
    let single_bit_table_entry =
        populate_single_bit_array_with_ones(committable_columns, signed_ones_length);

    // Create the full bit table vector to be used by Blitzar's vlen_msm algorithm.
    let bit_table = populate_bit_table(&single_bit_table_entry, max_height);

    // Create the full length vector to be used by Blitzar's vlen_msm algorithm.
    let length_table = populate_length_table(bit_table.len(), single_bit_table_entry.len());

    // Create a cumulative length table to be used when packing the scalar vector.
    let cumulative_byte_length_table: Vec<usize> = cumulative_byte_length_table(&bit_table);

    // Create scalars array. Note, scalars need to be stored in a column-major order.
    let num_scalar_rows = max_width;
    let num_scalar_columns = single_packed_byte_with_ones_size * max_height;
    let mut scalars = vec![0u8; num_scalar_rows * num_scalar_columns];

    // Populate the scalars array.
    let span = span!(Level::INFO, "pack_vlen_scalars_array").entered();
    scalars
        .chunks_exact_mut(num_scalar_columns)
        .enumerate()
        .for_each(|(scalar_row, scalar_row_slice)| {
            // Iterate over the columns and populate the scalars array.
            for scalar_col in 0..max_height {
                // Find index in the committable columns. Note, the scalar is in
                // column major order, that is why the (row, col) arguments are flipped.
                let committable_column_idx = index_from_row_and_column(scalar_col, scalar_row);

                // If the index is in the committable columns and above the offset, populate the scalars array.
                if committable_column_idx.is_some() && committable_column_idx.unwrap() >= offset {
                    let index: usize = committable_column_idx.unwrap() - offset;

                    // Iterate over each committable column.
                    for i in 0..committable_columns.len() {
                        if index < committable_columns[i].len() {
                            let start = cumulative_byte_length_table
                                [i + scalar_col * single_bit_table_entry.len()];
                            let end = start + (single_bit_table_entry[i] / BYTE_SIZE) as usize;

                            // For signed ones
                            let ones_index = i
                                + scalar_col * single_packed_byte_with_ones_size
                                + single_packed_byte_size;

                            copy_column_data_to_slice(
                                &committable_columns[i],
                                scalar_row_slice,
                                start,
                                end,
                                index,
                                ones_index,
                            );
                        }
                    }
                }
            }
        });
    span.exit();

    // Initialize sub commits.
    let mut sub_commits_from_blitzar =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); bit_table.len()];

    // Get sub commits from Blitzar's vlen_msm algorithm.
    if !bit_table.is_empty() {
        setup.blitzar_vlen_msm(
            &mut sub_commits_from_blitzar,
            &bit_table,
            &length_table,
            scalars.as_slice(),
        );
    }

    // Modify the sub commits to include the signed offset.
    let all_sub_commits: Vec<G1Affine> = slice_cast(&sub_commits_from_blitzar);
    let signed_sub_commits = signed_commits(&all_sub_commits, committable_columns);

    // Calculate the dynamic Dory commitments.
    assert!(
        signed_sub_commits.len() % committable_columns.len() == 0,
        "Invalid number of sub commits"
    );
    let num_commits = signed_sub_commits.len() / committable_columns.len();

    let span = span!(Level::INFO, "multi_pairing").entered();
    let ddc: Vec<DynamicDoryCommitment> = (0..committable_columns.len())
        .map(|i| {
            let sub_slice = signed_sub_commits[i..]
                .iter()
                .step_by(committable_columns.len())
                .take(num_commits);
            DynamicDoryCommitment(pairings::multi_pairing(sub_slice, &Gamma_2[..num_commits]))
        })
        .collect();
    span.exit();

    ddc
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
    fn we_can_populate_single_bit_array_with_ones() {
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

        let signed_ones_length = committable_columns.len();
        let single_bit_table_entry =
            populate_single_bit_array_with_ones(&committable_columns, signed_ones_length);
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
        let signed_ones_length = committable_columns.len();
        let single_bit_table_entry =
            populate_single_bit_array_with_ones(&committable_columns, signed_ones_length);
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
        let ones = committable_column_len;
        let num_of_rows = 7;

        let bit_table_len = (committable_column_len + ones) * num_of_rows;
        let single_bit_table_entry_len = committable_column_len + ones;

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
