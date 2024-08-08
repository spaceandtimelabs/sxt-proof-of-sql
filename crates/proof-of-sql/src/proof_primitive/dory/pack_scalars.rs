use crate::base::commitment::CommittableColumn;
use ark_bls12_381::Fr;
use num_traits::ToBytes;
use zerocopy::AsBytes;

const BYTE_SIZE: usize = 8;

pub trait OffsetToBytes {
    fn offset_to_bytes(&self) -> Vec<u8>;
}

impl OffsetToBytes for u8 {
    fn offset_to_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl OffsetToBytes for i16 {
    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i16::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for i32 {
    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i32::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for i64 {
    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i64::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for i128 {
    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i128::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for bool {
    fn offset_to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl OffsetToBytes for u64 {
    fn offset_to_bytes(&self) -> Vec<u8> {
        let bytes = self.to_le_bytes();
        bytes.to_vec()
    }
}

impl OffsetToBytes for [u64; 4] {
    fn offset_to_bytes(&self) -> Vec<u8> {
        let slice = self.as_bytes();
        slice.to_vec()
    }
}

fn get_byte_size(column: &CommittableColumn) -> usize {
    match column {
        CommittableColumn::SmallInt(_) => std::mem::size_of::<i16>(),
        CommittableColumn::Int(_) => std::mem::size_of::<i32>(),
        CommittableColumn::BigInt(_) => std::mem::size_of::<i64>(),
        CommittableColumn::Int128(_) => std::mem::size_of::<i128>(),
        CommittableColumn::Decimal75(_, _, _) => std::mem::size_of::<[u64; 4]>(),
        CommittableColumn::Scalar(_) => std::mem::size_of::<[u64; 4]>(),
        CommittableColumn::VarChar(_) => std::mem::size_of::<[u64; 4]>(),
        CommittableColumn::Boolean(_) => std::mem::size_of::<bool>(),
        CommittableColumn::TimestampTZ(_, _, _) => std::mem::size_of::<i64>(),
    }
}

fn get_bit_size(column: &CommittableColumn) -> usize {
    get_byte_size(column) * BYTE_SIZE
}

fn get_max_column_length(committable_columns: &[CommittableColumn]) -> usize {
    committable_columns
        .iter()
        .map(|column| column.len())
        .max()
        .unwrap_or(0)
}

pub fn get_min_as_fr(column: &CommittableColumn) -> Fr {
    match column {
        CommittableColumn::SmallInt(_) => Fr::from(i16::MIN),
        CommittableColumn::Int(_) => Fr::from(i32::MIN),
        CommittableColumn::BigInt(_) => Fr::from(i64::MIN),
        CommittableColumn::Int128(_) => Fr::from(i128::MIN),
        CommittableColumn::Decimal75(_, _, _) => Fr::from(0),
        CommittableColumn::Scalar(_) => Fr::from(0),
        CommittableColumn::VarChar(_) => Fr::from(0),
        CommittableColumn::Boolean(_) => Fr::from(0),
        CommittableColumn::TimestampTZ(_, _, _) => Fr::from(i64::MIN),
    }
}

pub fn get_output_bit_table(committable_columns: &[CommittableColumn]) -> Vec<u32> {
    committable_columns
        .iter()
        .map(|column| get_bit_size(column) as u32)
        .collect()
}

fn get_repeated_bit_table(bit_table: &[u32], num_of_commits_per_column: usize) -> Vec<u32> {
    bit_table
        .iter()
        .flat_map(|&value| std::iter::repeat(value).take(num_of_commits_per_column))
        .collect()
}

pub fn get_num_of_commits(
    committable_columns: &[CommittableColumn],
    offset: usize,
    num_of_generators: usize,
) -> usize {
    // Committable columns may be different sizes, get the max size and add offset.
    let max_column_length = get_max_column_length(committable_columns) + offset;

    // Number of scalar vectors that the size of the generators n.
    // Each scalar will be used to call the packed_msm function.
    (max_column_length + num_of_generators - 1) / num_of_generators
}

fn pack_bit<T: OffsetToBytes>(
    column: &[T],
    packed_scalars: &mut [u8],
    current_bit_table_sum: usize,
    idx_offset: usize,
    current_byte_size: usize,
    full_row_byte_size: usize,
    num_of_generators: usize,
) {
    let byte_offset = current_bit_table_sum / BYTE_SIZE;
    for (idx, value) in column.iter().enumerate() {
        let row_offset = ((idx + idx_offset) % num_of_generators) * full_row_byte_size;
        let col_offset = current_byte_size * ((idx + idx_offset) / num_of_generators);
        let offset_idx = row_offset + col_offset + byte_offset;

        packed_scalars[offset_idx..offset_idx + current_byte_size]
            .copy_from_slice(&value.offset_to_bytes()[..]);
    }
}

fn pack_bit_offset<T: OffsetToBytes>(
    column: &[T],
    packed_scalars: &mut [u8],
    current_bit_table_sum: usize,
    idx_offset: usize,
    full_row_byte_size: usize,
    num_of_rows: usize,
) {
    let byte_offset = current_bit_table_sum / BYTE_SIZE;
    for (idx, _) in column.iter().enumerate() {
        let row_offset = ((idx + idx_offset) % num_of_rows) * full_row_byte_size;
        let col_offset = (idx + idx_offset) / num_of_rows;
        let offset_idx = row_offset + col_offset + byte_offset;

        packed_scalars[offset_idx] = 1_u8;
    }
}

#[tracing::instrument(
    name = "get_bit_table_and_scalar_for_packed_msm (gpu)",
    level = "debug",
    skip_all
)]
pub fn get_bit_table_and_scalar_for_packed_msm(
    bit_table: &[u32],
    committable_columns: &[CommittableColumn],
    offset: usize,
    num_of_generators: usize,
    num_of_commits: usize,
) -> (Vec<u32>, Vec<u8>) {
    // Extend the bit table for packed msm.
    let repeated_bit_table = get_repeated_bit_table(bit_table, num_of_commits);
    let repeated_bit_table_sum = repeated_bit_table.iter().sum::<u32>() as usize;

    // Extend for offsets
    let mut extended_bit_table = Vec::with_capacity(repeated_bit_table.len() * 2);
    extended_bit_table.extend_from_slice(&repeated_bit_table);
    extended_bit_table.extend(std::iter::repeat(8).take(repeated_bit_table.len()));

    let extended_bit_table_sum = extended_bit_table.iter().sum::<u32>() as usize;
    let full_row_byte_size = extended_bit_table_sum / BYTE_SIZE;
    let packed_scalar_size_extended = full_row_byte_size * num_of_generators;
    let mut packed_scalars_temp_extended = vec![0_u8; packed_scalar_size_extended];

    for (i, column) in committable_columns.iter().enumerate() {
        // Get the running sum of the bit table.
        let bit_table_sum = if i > 0 {
            extended_bit_table
                .iter()
                .take(i * num_of_commits)
                .sum::<u32>() as usize
        } else {
            0
        };
        let bit_table_offset_sum = repeated_bit_table_sum + i * BYTE_SIZE * num_of_commits;

        // Get the byte size of the column of data.
        let current_byte_size = get_byte_size(&committable_columns[i]);

        match column {
            CommittableColumn::SmallInt(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_sum,
                    offset,
                    current_byte_size,
                    full_row_byte_size,
                    num_of_generators,
                );
                pack_bit_offset(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_offset_sum,
                    offset,
                    full_row_byte_size,
                    num_of_generators,
                );
            }
            CommittableColumn::Int(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_sum,
                    offset,
                    current_byte_size,
                    full_row_byte_size,
                    num_of_generators,
                );
                pack_bit_offset(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_offset_sum,
                    offset,
                    full_row_byte_size,
                    num_of_generators,
                );
            }
            CommittableColumn::BigInt(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_sum,
                    offset,
                    current_byte_size,
                    full_row_byte_size,
                    num_of_generators,
                );
                pack_bit_offset(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_offset_sum,
                    offset,
                    full_row_byte_size,
                    num_of_generators,
                );
            }
            CommittableColumn::Int128(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_sum,
                    offset,
                    current_byte_size,
                    full_row_byte_size,
                    num_of_generators,
                );
                pack_bit_offset(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_offset_sum,
                    offset,
                    full_row_byte_size,
                    num_of_generators,
                );
            }
            CommittableColumn::TimestampTZ(_, _, column) => {
                pack_bit(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_sum,
                    offset,
                    current_byte_size,
                    full_row_byte_size,
                    num_of_generators,
                );
                pack_bit_offset(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_offset_sum,
                    offset,
                    full_row_byte_size,
                    num_of_generators,
                );
            }
            CommittableColumn::Boolean(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_sum,
                    offset,
                    current_byte_size,
                    full_row_byte_size,
                    num_of_generators,
                );
            }
            CommittableColumn::Decimal75(_, _, column) => {
                pack_bit(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_sum,
                    offset,
                    current_byte_size,
                    full_row_byte_size,
                    num_of_generators,
                );
            }
            CommittableColumn::Scalar(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_sum,
                    offset,
                    current_byte_size,
                    full_row_byte_size,
                    num_of_generators,
                );
            }
            CommittableColumn::VarChar(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars_temp_extended,
                    bit_table_sum,
                    offset,
                    current_byte_size,
                    full_row_byte_size,
                    num_of_generators,
                );
            }
        }
    }

    (extended_bit_table, packed_scalars_temp_extended)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::math::decimal::Precision;
    use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

    #[test]
    fn we_can_get_correct_data_sizes() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1, 2, 3]),
            CommittableColumn::Int(&[1, 2, 3]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![[1, 2, 3, 4], [5, 6, 7, 8]],
            ),
            CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::VarChar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::Boolean(&[true, false, true]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1, 2, 3]),
        ];

        let expected_bit_sizes = [16, 32, 64, 128, 64 * 4, 64 * 4, 64 * 4, 8, 64];
        let expected_byte_size = expected_bit_sizes
            .iter()
            .map(|&x| x / BYTE_SIZE)
            .collect::<Vec<usize>>();

        for (i, column) in committable_columns.iter().enumerate() {
            let bit_size = get_bit_size(column);
            let byte_size = get_byte_size(column);

            assert_eq!(bit_size, expected_bit_sizes[i]);
            assert_eq!(byte_size, expected_byte_size[i]);
        }
    }

    #[test]
    fn we_can_get_max_column_length_of_the_same_type() {
        let committable_columns = [
            CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::Scalar(vec![[1, 2, 3, 4]]),
        ];

        let max_column_length = get_max_column_length(&committable_columns);
        assert_eq!(max_column_length, 2);
    }

    #[test]
    fn we_can_get_max_column_length_of_different_types() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[1, 2, 3]),
            CommittableColumn::Int(&[1, 2, 3]),
            CommittableColumn::BigInt(&[1, 2, 3]),
            CommittableColumn::Int128(&[1, 2, 3, 4, 5, 6]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![[1, 2, 3, 4], [5, 6, 7, 8]],
            ),
            CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::VarChar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::Boolean(&[true, false, true]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1, 2, 3]),
        ];

        let max_column_length = get_max_column_length(&committable_columns);
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
                vec![[1, 2, 3, 4], [5, 6, 7, 8]],
            ),
            CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::VarChar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::Boolean(&[true, false, true]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1, 2, 3]),
        ];

        let bit_table = get_output_bit_table(&committable_columns);
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
                vec![[1, 2, 3, 4], [5, 6, 7, 8]],
            ),
            CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::VarChar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::Boolean(&[true, false, true]),
            CommittableColumn::TimestampTZ(PoSQLTimeUnit::Second, PoSQLTimeZone::Utc, &[1, 2, 3]),
        ];

        let bit_table = get_output_bit_table(&committable_columns);
        let repeated_bit_table = get_repeated_bit_table(&bit_table, 3);
        let expected = [
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
        assert_eq!(repeated_bit_table, expected);
    }

    #[test]
    fn we_can_get_num_of_columns_with_more_generators_than_columns() {
        let committable_columns = [
            CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::Scalar(vec![[1, 2, 3, 4]]),
        ];

        let offset = 0;
        let num_of_generators = 4;
        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 1);
    }

    #[test]
    fn we_can_get_num_of_columns_with_more_generators_than_columns_and_offset() {
        let committable_columns = [
            CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::Scalar(vec![[1, 2, 3, 4]]),
        ];

        let offset = 5;
        let num_of_generators = 4;
        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 2);
    }

    #[test]
    fn we_can_get_num_of_columns_with_more_columns_than_generators() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]),
            CommittableColumn::Int(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
            CommittableColumn::SmallInt(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
        ];

        let offset = 0;
        let num_of_generators = 4;
        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 5);
    }

    #[test]
    fn we_can_get_num_of_columns_with_more_columns_than_generators_and_offset() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]),
            CommittableColumn::Int(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
            CommittableColumn::SmallInt(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
        ];

        let offset = 1;
        let num_of_generators = 4;
        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 6);
    }

    #[test]
    fn we_can_create_a_mixed_packed_scalar() {
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

        let bit_table = get_output_bit_table(&committable_columns);
        assert_eq!(bit_table, [16, 32, 16]);

        let num_of_generators = 4;
        let offset = 0;

        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 5);

        let repeated_bit_table = get_repeated_bit_table(&bit_table, num_of_commits);
        assert_eq!(
            repeated_bit_table,
            [16, 16, 16, 16, 16, 32, 32, 32, 32, 32, 16, 16, 16, 16, 16]
        );

        let (extended_bit_table, packed_scalar) = get_bit_table_and_scalar_for_packed_msm(
            &bit_table,
            &committable_columns,
            offset,
            num_of_generators,
            num_of_commits,
        );

        let expected_bit_table = [
            16, 16, 16, 16, 16, 32, 32, 32, 32, 32, 16, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8,
        ];

        let expected_scalar = [
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

        assert_eq!(extended_bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_scalar);
    }

    #[test]
    fn we_can_create_mixed_packed_scalar_with_offset() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[0, 1, 2, 3]),
            CommittableColumn::Int(&[4, 5, 6, 7]),
            CommittableColumn::SmallInt(&[8, 9, 10, 11]),
        ];

        let bit_table = get_output_bit_table(&committable_columns);
        assert_eq!(bit_table, [16, 32, 16]);

        let num_of_generators = 4;
        let offset = 5;

        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 3);

        let (repeated_bit_table, packed_scalar) = get_bit_table_and_scalar_for_packed_msm(
            &bit_table,
            &committable_columns,
            offset,
            num_of_generators,
            num_of_commits,
        );

        let expected_bit_table = [
            16, 16, 16, 32, 32, 32, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8,
        ];

        let expected_scalar = [
            0, 0, 0, 0, 3, 128, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 128, 0, 0, 0, 0, 11, 128, 0, 0, 1,
            0, 0, 1, 0, 0, 1, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 4, 0, 0, 128, 0, 0, 0, 0, 0, 0, 8,
            128, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 1, 128, 0, 0, 0, 0, 0, 0, 5, 0, 0, 128, 0,
            0, 0, 0, 0, 0, 9, 128, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 2, 128, 0, 0, 0, 0, 0, 0,
            6, 0, 0, 128, 0, 0, 0, 0, 0, 0, 10, 128, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0,
        ];

        assert_eq!(repeated_bit_table, expected_bit_table);
        assert_eq!(packed_scalar, expected_scalar);
    }

    #[test]
    fn we_can_pack_empty_scalars() {
        let committable_columns = [];

        let bit_table = get_output_bit_table(&committable_columns);
        assert!(bit_table.is_empty());

        let (packed_scalar, packed_scalar_offsets) =
            get_bit_table_and_scalar_for_packed_msm(&bit_table, &committable_columns, 0, 1, 0);

        assert!(packed_scalar.is_empty());
        assert!(packed_scalar_offsets.is_empty());
    }

    #[test]
    fn we_can_pack_scalars_with_one_full_row() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];
        let bit_table = get_output_bit_table(&committable_columns);
        assert_eq!(bit_table, [64, 64]);

        let offset = 0;
        let num_of_generators = 2;

        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 1);

        let (repeated_bit_table, packed_scalar) = get_bit_table_and_scalar_for_packed_msm(
            &bit_table,
            &committable_columns,
            offset,
            num_of_generators,
            num_of_commits,
        );

        let expected_scalar = [
            1, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0, 0, 0, 0, 128, 1, 1, 2, 0, 0, 0, 0, 0, 0, 128, 4,
            0, 0, 0, 0, 0, 0, 128, 1, 1,
        ];

        let expected_bit_table = [64, 64, 8, 8];

        assert_eq!(packed_scalar, expected_scalar);
        assert_eq!(repeated_bit_table, expected_bit_table);
    }

    #[test]
    fn we_can_pack_scalars_with_more_than_one_row() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];
        let bit_table = get_output_bit_table(&committable_columns);
        assert_eq!(bit_table, [64, 64]);

        let offset = 0;
        let num_of_generators = 1;

        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 2);

        let (repeated_bit_table, packed_scalar) = get_bit_table_and_scalar_for_packed_msm(
            &bit_table,
            &committable_columns,
            offset,
            num_of_generators,
            num_of_commits,
        );

        let expected_scalar = [
            1, 0, 0, 0, 0, 0, 0, 128, 2, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0, 0, 0, 0, 128, 4, 0, 0,
            0, 0, 0, 0, 128, 1, 1, 1, 1,
        ];

        let expected_bit_table = [64, 64, 64, 64, 8, 8, 8, 8];

        assert_eq!(packed_scalar, expected_scalar);
        assert_eq!(repeated_bit_table, expected_bit_table);
    }

    #[test]
    fn we_can_pack_scalars_with_one_full_row_with_offset() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];
        let bit_table = get_output_bit_table(&committable_columns);
        assert_eq!(bit_table, [64, 64]);

        let offset = 1;
        let num_of_generators = 2;

        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 2);

        let (repeated_bit_table, packed_scalar) = get_bit_table_and_scalar_for_packed_msm(
            &bit_table,
            &committable_columns,
            offset,
            num_of_generators,
            num_of_commits,
        );

        let expected_scalar = [
            0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0,
            0, 0, 0, 128, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0,
            0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0,
        ];

        let expected_bit_table = [64, 64, 64, 64, 8, 8, 8, 8];

        assert_eq!(packed_scalar, expected_scalar);
        assert_eq!(repeated_bit_table, expected_bit_table);
    }

    #[test]
    fn we_can_pack_scalars_with_more_than_one_row_with_offset() {
        let committable_columns = [
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];

        let bit_table = get_output_bit_table(&committable_columns);
        assert_eq!(bit_table, [64, 64]);

        let offset = 1;
        let num_of_generators = 1;

        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 3);

        let (repeated_bit_table, packed_scalar) = get_bit_table_and_scalar_for_packed_msm(
            &bit_table,
            &committable_columns,
            offset,
            num_of_generators,
            num_of_commits,
        );

        let expected_scalar = [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 128, 2, 0, 0, 0, 0, 0, 0, 128, 0, 0, 0, 0,
            0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 128, 4, 0, 0, 0, 0, 0, 0, 128, 0, 1, 1, 0, 1, 1,
        ];

        let expected_bit_table = [64, 64, 64, 64, 64, 64, 8, 8, 8, 8, 8, 8];

        assert_eq!(packed_scalar, expected_scalar);
        assert_eq!(repeated_bit_table, expected_bit_table);
    }

    #[test]
    fn we_can_add_offsets() {
        let committable_columns = [
            CommittableColumn::SmallInt(&[0, 1, 2, 3, 4, 5]),
            CommittableColumn::Int(&[6, 7, 8, 9]),
            CommittableColumn::Scalar(vec![[10, 0, 0, 0], [11, 0, 0, 0], [12, 0, 0, 0]]),
        ];
        let bit_table = get_output_bit_table(&committable_columns);
        assert_eq!(bit_table, [16, 32, 64 * 4]);

        let offset = 0;
        let num_of_generators = 3;

        let num_of_commits = get_num_of_commits(&committable_columns, offset, num_of_generators);
        assert_eq!(num_of_commits, 2);

        let (repeated_bit_table, packed_scalar) = get_bit_table_and_scalar_for_packed_msm(
            &bit_table,
            &committable_columns,
            offset,
            num_of_generators,
            num_of_commits,
        );

        let expected_scalar = [
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

        assert_eq!(packed_scalar, expected_scalar);
        assert_eq!(repeated_bit_table, expected_bit_table);
    }
}
