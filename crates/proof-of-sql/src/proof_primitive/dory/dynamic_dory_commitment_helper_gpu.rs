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
use rayon::prelude::*;
use std::sync::Mutex;
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
    name = "compute_dory_commitment_impl_gpu (vlen_msm gpu)",
    level = "debug",
    skip_all
)]
fn compute_dory_commitment_impl_gpu(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &ProverSetup,
) -> Vec<DynamicDoryCommitment> {
    let Gamma_2 = setup.Gamma_2.last().unwrap();

    // The maximum matrix size will be used to create the scalars vector.
    let (max_height, max_width) = max_matrix_size(committable_columns, offset);

    // Find the single packed byte size of all committable columns.
    let single_packed_byte_size = single_packed_byte_size(committable_columns);

    // Get a single bit table entry with offsets of all committable columns.
    let signed_offset_length = committable_columns.len();
    let single_packed_byte_with_offset_size = single_packed_byte_size + signed_offset_length;
    let single_bit_table_entry =
        populate_single_bit_array_with_offsets(committable_columns, signed_offset_length);

    // Create the full bit table vector to be used by Blitzar's vlen_msm algorithm.
    let bit_table = populate_bit_table(&single_bit_table_entry, max_height);

    // Create the full length vector to be used by Blitzar's vlen_msm algorithm.
    let length_table = populate_length_table(bit_table.len(), single_bit_table_entry.len());

    // Create a cumulative length table to be used when packing the scalar vector.
    let cumulative_byte_length_table: Vec<usize> = cumulative_byte_length_table(&bit_table);

    // Create scalars array. Note, scalars need to be stored in a column-major order.
    let num_scalar_rows = max_width;
    let num_scalar_columns = single_packed_byte_with_offset_size * max_height;
    let scalars = vec![0u8; num_scalar_rows * num_scalar_columns];

    // Populate the scalars array.
    let span = span!(Level::INFO, "pack_vlen_scalars_array").entered();
    let scalars = Mutex::new(scalars);
    (0..num_scalar_rows).into_par_iter().for_each(|scalar_row| {
        // Get a mutable slice of the scalars array that represents one full row of the scalars array.
        let mut scalars = scalars.lock().unwrap();
        let scalar_row_slice =
            &mut scalars[scalar_row * num_scalar_columns..(scalar_row + 1) * num_scalar_columns];

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

                        // For signed offset
                        let offset_idx = i
                            + scalar_col * single_packed_byte_with_offset_size
                            + single_packed_byte_size;

                        let column = &committable_columns[i];
                        match column {
                            CommittableColumn::Boolean(column) => {
                                scalar_row_slice[start..end]
                                    .copy_from_slice(&column[index].offset_to_bytes());
                            }
                            CommittableColumn::TinyInt(column) => {
                                scalar_row_slice[start..end]
                                    .copy_from_slice(&column[index].offset_to_bytes());

                                scalar_row_slice[offset_idx] = 1_u8;
                            }
                            CommittableColumn::SmallInt(column) => {
                                scalar_row_slice[start..end]
                                    .copy_from_slice(&column[index].offset_to_bytes());

                                scalar_row_slice[offset_idx] = 1_u8;
                            }
                            CommittableColumn::Int(column) => {
                                scalar_row_slice[start..end]
                                    .copy_from_slice(&column[index].offset_to_bytes());

                                scalar_row_slice[offset_idx] = 1_u8;
                            }
                            CommittableColumn::BigInt(column)
                            | CommittableColumn::TimestampTZ(_, _, column) => {
                                scalar_row_slice[start..end]
                                    .copy_from_slice(&column[index].offset_to_bytes());

                                scalar_row_slice[offset_idx] = 1_u8;
                            }
                            CommittableColumn::Int128(column) => {
                                scalar_row_slice[start..end]
                                    .copy_from_slice(&column[index].offset_to_bytes());

                                scalar_row_slice[offset_idx] = 1_u8;
                            }
                            CommittableColumn::Scalar(column)
                            | CommittableColumn::Decimal75(_, _, column)
                            | CommittableColumn::VarChar(column) => {
                                scalar_row_slice[start..end]
                                    .copy_from_slice(&column[index].offset_to_bytes());
                            }
                            CommittableColumn::RangeCheckWord(_) => todo!(),
                        }
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
        let scalars_guard = scalars.lock().unwrap();
        setup.blitzar_vlen_msm(
            &mut sub_commits_from_blitzar,
            &bit_table,
            &length_table,
            scalars_guard.as_slice(),
        );
    }

    // Modify the sub commits to include the signed offset.
    let all_sub_commits: Vec<G1Affine> = slice_cast(&sub_commits_from_blitzar);
    let sub_commits = modify_commits(&all_sub_commits, committable_columns);

    // Calculate the dynamic Dory commitments.
    assert!(
        sub_commits.len() % committable_columns.len() == 0,
        "Invalid number of sub commits"
    );
    let num_commits = sub_commits.len() / committable_columns.len();

    let span = span!(Level::INFO, "multi_pairing").entered();
    let ddc: Vec<DynamicDoryCommitment> = (0..committable_columns.len())
        .into_par_iter()
        .map(|i| {
            let sub_slice = sub_commits[i..]
                .iter()
                .step_by(committable_columns.len())
                .take(num_commits);
            DynamicDoryCommitment(pairings::multi_pairing(sub_slice, &Gamma_2[..num_commits]))
        })
        .collect();
    span.exit();

    ddc
}

/// Computes the dynamic Dory commitments using the GPU implementation of the `vlen_msm` algorithm.
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
pub(super) fn compute_dynamic_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &ProverSetup,
) -> Vec<DynamicDoryCommitment> {
    compute_dory_commitment_impl_gpu(committable_columns, offset, setup)
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
