use crate::base::commitment::CommittableColumn;
use ark_bls12_381::Fr;
use num_traits::ToBytes;
use zerocopy::AsBytes;

const BYTE_SIZE: usize = 8;

pub trait OffsetToBytes {
    const IS_SIGNED: bool;
    fn min_as_fr() -> Fr;
    fn offset_to_bytes(&self) -> Vec<u8>;
}

impl OffsetToBytes for u8 {
    const IS_SIGNED: bool = false;

    fn min_as_fr() -> Fr {
        Fr::from(0)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl OffsetToBytes for i16 {
    const IS_SIGNED: bool = true;

    fn min_as_fr() -> Fr {
        Fr::from(i16::MIN)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i16::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for i32 {
    const IS_SIGNED: bool = true;

    fn min_as_fr() -> Fr {
        Fr::from(i32::MIN)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i32::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for i64 {
    const IS_SIGNED: bool = true;

    fn min_as_fr() -> Fr {
        Fr::from(i64::MIN)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i64::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for i128 {
    const IS_SIGNED: bool = true;

    fn min_as_fr() -> Fr {
        Fr::from(i128::MIN)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let shifted = self.wrapping_sub(i128::MIN);
        shifted.to_le_bytes().to_vec()
    }
}

impl OffsetToBytes for bool {
    const IS_SIGNED: bool = false;

    fn min_as_fr() -> Fr {
        Fr::from(false)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl OffsetToBytes for u64 {
    const IS_SIGNED: bool = false;

    fn min_as_fr() -> Fr {
        Fr::from(0)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let bytes = self.to_le_bytes();
        bytes.to_vec()
    }
}

impl OffsetToBytes for [u64; 4] {
    const IS_SIGNED: bool = false;

    fn min_as_fr() -> Fr {
        Fr::from(0)
    }

    fn offset_to_bytes(&self) -> Vec<u8> {
        let slice = self.as_bytes();
        slice.to_vec()
    }
}

#[tracing::instrument(name = "transpose_for_fixed_msm (gpu)", level = "debug", skip_all)]
pub fn transpose_for_fixed_msm<T: AsBytes + Copy + OffsetToBytes>(
    column: &[T],
    offset: usize,
    rows: usize,
    cols: usize,
    data_size: usize,
) -> Vec<u8> {
    let total_length_bytes = data_size * rows * cols;
    let mut transpose = vec![0_u8; total_length_bytes];
    for n in offset..(column.len() + offset) {
        let i = n / cols;
        let j = n % cols;
        let t_idx = (j * rows + i) * data_size;
        let p_idx = (i * cols + j) - offset;

        transpose[t_idx..t_idx + data_size]
            .copy_from_slice(column[p_idx].offset_to_bytes().as_slice());
    }
    transpose
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

fn get_single_rounded_packed_scalar_bit_size(bit_table: &[u32]) -> usize {
    let total_bits = bit_table.iter().sum::<u32>() as usize;
    (total_bits + BYTE_SIZE - 1) / BYTE_SIZE * BYTE_SIZE
}

fn get_max_column_length(committable_columns: &[CommittableColumn]) -> usize {
    committable_columns
        .iter()
        .map(|column| column.len())
        .max()
        .unwrap_or(0)
}

fn pack_bit<T: OffsetToBytes>(
    column: &[T],
    packed_scalars: &mut [Vec<u8>],
    scalar_byte_size: usize,
    byte_offset: usize,
    single_packed_scalar_bit_size: usize,
    current_bit: &mut usize,
) {
    for value in column.iter() {
        let byte_idx = *current_bit / BYTE_SIZE;
        let scalar_idx = byte_idx / scalar_byte_size;
        let local_byte_idx = byte_idx % scalar_byte_size;
        packed_scalars[scalar_idx][local_byte_idx..local_byte_idx + byte_offset]
            .copy_from_slice(&value.offset_to_bytes()[..]);
        *current_bit += single_packed_scalar_bit_size;
    }
}

fn add_offset_bit<T>(
    column: &[T],
    packed_scalars: &mut [Vec<u8>],
    scalar_byte_size: usize,
    single_packed_scalar_bit_size: usize,
    current_bit: &mut usize,
) {
    for _ in column.iter() {
        let byte_idx = *current_bit / BYTE_SIZE;
        let scalar_idx = byte_idx / scalar_byte_size;
        let local_byte_idx = byte_idx % scalar_byte_size;
        packed_scalars[scalar_idx][local_byte_idx] |= 1;
        *current_bit += single_packed_scalar_bit_size;
    }
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

#[tracing::instrument(name = "get_output_bit_table (gpu)", level = "debug", skip_all)]
pub fn get_output_bit_table(committable_columns: &[CommittableColumn]) -> Vec<u32> {
    committable_columns
        .iter()
        .map(|column| get_bit_size(column) as u32)
        .collect()
}

#[tracing::instrument(
    name = "get_packed_scalar_and_offset_scalar_offset (gpu)",
    level = "debug",
    skip_all
)]
pub fn get_packed_scalar_and_offset_scalar_offset(
    bit_table: &[u32],
    committable_columns: &[CommittableColumn],
    offset: usize,
    n: usize,
) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    // Get the total bit size needed for a single entry in the scalar vector.
    let single_packed_scalar_bit_size = get_single_rounded_packed_scalar_bit_size(bit_table);
    assert!(single_packed_scalar_bit_size % BYTE_SIZE == 0);

    // The offset adds bits to the beginning of the scalar vector.
    let bit_offset = offset * single_packed_scalar_bit_size;

    // Committable columns may be different sizes, get the max size and add offset.
    let max_column_length = get_max_column_length(committable_columns) + offset;

    // Number of scalar vectors that the size of the generators n.
    // Each scalar will be used to call the packed_msm function.
    let scalar_vec_size = (max_column_length + n - 1) / n;

    // Get the total number of bits needed for the packed scalar vector.
    let full_packed_scalar_bit_size = if scalar_vec_size > 0 {
        single_packed_scalar_bit_size * n
    } else {
        single_packed_scalar_bit_size * max_column_length
    };

    // Convert to bytes
    let scalar_byte_size = full_packed_scalar_bit_size / BYTE_SIZE;

    // Create a vector of scalar vectors
    let mut packed_scalars = vec![vec![0_u8; scalar_byte_size]; scalar_vec_size];
    let mut packed_scalar_offsets = vec![vec![0_u8; scalar_byte_size]; scalar_vec_size];

    for (i, column) in committable_columns.iter().enumerate() {
        let mut current_bit = if i > 0 {
            bit_offset + bit_table.iter().take(i).sum::<u32>() as usize
        } else {
            bit_offset
        };
        let mut current_offset_bit = if i > 0 {
            bit_offset + bit_table.iter().take(i).sum::<u32>() as usize
        } else {
            bit_offset
        };
        let byte_offset = get_byte_size(&committable_columns[i]);

        match column {
            CommittableColumn::SmallInt(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars,
                    scalar_byte_size,
                    byte_offset,
                    single_packed_scalar_bit_size,
                    &mut current_bit,
                );
                add_offset_bit(
                    column,
                    &mut packed_scalar_offsets,
                    scalar_byte_size,
                    single_packed_scalar_bit_size,
                    &mut current_offset_bit,
                );
            }
            CommittableColumn::Int(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars,
                    scalar_byte_size,
                    byte_offset,
                    single_packed_scalar_bit_size,
                    &mut current_bit,
                );
                add_offset_bit(
                    column,
                    &mut packed_scalar_offsets,
                    scalar_byte_size,
                    single_packed_scalar_bit_size,
                    &mut current_offset_bit,
                );
            }
            CommittableColumn::BigInt(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars,
                    scalar_byte_size,
                    byte_offset,
                    single_packed_scalar_bit_size,
                    &mut current_bit,
                );
                add_offset_bit(
                    column,
                    &mut packed_scalar_offsets,
                    scalar_byte_size,
                    single_packed_scalar_bit_size,
                    &mut current_offset_bit,
                );
            }
            CommittableColumn::Int128(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars,
                    scalar_byte_size,
                    byte_offset,
                    single_packed_scalar_bit_size,
                    &mut current_bit,
                );
                add_offset_bit(
                    column,
                    &mut packed_scalar_offsets,
                    scalar_byte_size,
                    single_packed_scalar_bit_size,
                    &mut current_offset_bit,
                );
            }
            CommittableColumn::TimestampTZ(_, _, column) => {
                pack_bit(
                    column,
                    &mut packed_scalars,
                    scalar_byte_size,
                    byte_offset,
                    single_packed_scalar_bit_size,
                    &mut current_bit,
                );
                add_offset_bit(
                    column,
                    &mut packed_scalar_offsets,
                    scalar_byte_size,
                    single_packed_scalar_bit_size,
                    &mut current_offset_bit,
                );
            }
            CommittableColumn::Boolean(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars,
                    scalar_byte_size,
                    byte_offset,
                    single_packed_scalar_bit_size,
                    &mut current_bit,
                );
            }
            CommittableColumn::Decimal75(_, _, column) => {
                pack_bit(
                    column,
                    &mut packed_scalars,
                    scalar_byte_size,
                    byte_offset,
                    single_packed_scalar_bit_size,
                    &mut current_bit,
                );
            }
            CommittableColumn::Scalar(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars,
                    scalar_byte_size,
                    byte_offset,
                    single_packed_scalar_bit_size,
                    &mut current_bit,
                );
            }
            CommittableColumn::VarChar(column) => {
                pack_bit(
                    column,
                    &mut packed_scalars,
                    scalar_byte_size,
                    byte_offset,
                    single_packed_scalar_bit_size,
                    &mut current_bit,
                );
            }
        }
    }

    (packed_scalars, packed_scalar_offsets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::math::decimal::Precision;
    use proof_of_sql_parser::posql_time::{timezone::PoSQLTimeZone, unit::PoSQLTimeUnit};

    #[test]
    fn we_can_transpose_empty_column() {
        type T = u64;
        let column: Vec<T> = vec![];
        let offset = 0;
        let rows = 0;
        let cols = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);
        assert!(transpose.is_empty());
    }

    #[test]
    fn we_can_transpose_u64_column() {
        type T = u64;
        let column: Vec<T> = vec![0, 1, 2, 3];
        let offset = 0;
        let rows = 2;
        let cols = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(&transpose[0..data_size], column[0].as_bytes());
        assert_eq!(&transpose[data_size..2 * data_size], column[2].as_bytes());
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[1].as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[3].as_bytes()
        );
    }

    #[test]
    fn we_can_transpose_u64_column_with_offset() {
        type T = u64;
        let column: Vec<T> = vec![1, 2, 3];
        let offset = 2;
        let rows = 2;
        let cols = 3;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset + 1);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(&transpose[0..data_size], 0_u64.as_bytes());
        assert_eq!(&transpose[data_size..2 * data_size], column[1].as_bytes());
        assert_eq!(&transpose[2 * data_size..3 * data_size], 0_u64.as_bytes());
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[2].as_bytes()
        );
        assert_eq!(
            &transpose[4 * data_size..5 * data_size],
            column[0].as_bytes()
        );
        assert_eq!(&transpose[5 * data_size..6 * data_size], 0_u64.as_bytes());
    }

    #[test]
    fn we_can_transpose_boolean_column_with_offset() {
        type T = bool;
        let column: Vec<T> = vec![true, false, true];
        let offset = 1;
        let rows = 2;
        let cols = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(&transpose[0..data_size], 0_u8.as_bytes());
        assert_eq!(&transpose[data_size..2 * data_size], column[1].as_bytes());
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[0].as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[2].as_bytes()
        );
    }

    #[test]
    fn we_can_transpose_i64_column() {
        type T = i64;
        let column: Vec<T> = vec![0, 1, 2, 3];
        let offset = 0;
        let rows = 2;
        let cols = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(
            &transpose[0..data_size],
            column[0].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[data_size..2 * data_size],
            column[2].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[1].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[3].wrapping_sub(T::MIN).as_bytes()
        );
    }

    #[test]
    fn we_can_transpose_i128_column() {
        type T = i128;
        let column: Vec<T> = vec![0, 1, 2, 3];
        let offset = 0;
        let rows = 2;
        let cols = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(
            &transpose[0..data_size],
            column[0].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[data_size..2 * data_size],
            column[2].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[1].wrapping_sub(T::MIN).as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[3].wrapping_sub(T::MIN).as_bytes()
        );
    }

    #[test]
    fn we_can_transpose_u64_array_column() {
        type T = [u64; 4];
        let column: Vec<T> = vec![[0, 0, 0, 0], [1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]];
        let offset = 0;
        let rows = 2;
        let cols = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(&transpose[0..data_size], column[0].as_bytes());
        assert_eq!(&transpose[data_size..2 * data_size], column[2].as_bytes());
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[1].as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[3].as_bytes()
        );
    }

    #[test]
    fn we_can_transpose_u64_array_column_update() {
        type T = [u64; 4];
        let column: Vec<T> = vec![[0, 0, 0, 0], [1, 0, 0, 0], [2, 0, 0, 0], [3, 0, 0, 0]];
        let offset = 0;
        let rows = 2;
        let cols = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_for_fixed_msm(&column, offset, rows, cols, data_size);

        assert_eq!(transpose.len(), expected_len);

        assert_eq!(&transpose[0..data_size], column[0].as_bytes());
        assert_eq!(&transpose[data_size..2 * data_size], column[2].as_bytes());
        assert_eq!(
            &transpose[2 * data_size..3 * data_size],
            column[1].as_bytes()
        );
        assert_eq!(
            &transpose[3 * data_size..4 * data_size],
            column[3].as_bytes()
        );
    }

    #[test]
    fn we_can_get_a_bit_table() {
        let committable_columns = vec![
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

        let expected = vec![16, 32, 64, 128, 64 * 4, 64 * 4, 64 * 4, 8, 64];

        assert_eq!(bit_table, expected);
    }

    #[test]
    fn we_can_get_min_as_fr() {
        let fr_mins = vec![
            get_min_as_fr(&CommittableColumn::SmallInt(&[1, 2, 3])),
            get_min_as_fr(&CommittableColumn::Int(&[1, 2, 3])),
            get_min_as_fr(&CommittableColumn::BigInt(&[1, 2, 3])),
            get_min_as_fr(&CommittableColumn::Int128(&[1, 2, 3])),
            get_min_as_fr(&CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![[1, 2, 3, 4], [5, 6, 7, 8]],
            )),
            get_min_as_fr(&CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]])),
            get_min_as_fr(&CommittableColumn::VarChar(vec![
                [1, 2, 3, 4],
                [5, 6, 7, 8],
            ])),
            get_min_as_fr(&CommittableColumn::Boolean(&[true, false, true])),
            get_min_as_fr(&CommittableColumn::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::Utc,
                &[1, 2, 3],
            )),
        ];

        let expected = vec![
            Fr::from(i16::MIN),
            Fr::from(i32::MIN),
            Fr::from(i64::MIN),
            Fr::from(i128::MIN),
            Fr::from(0),
            Fr::from(0),
            Fr::from(0),
            Fr::from(false),
            Fr::from(i64::MIN),
        ];

        assert_eq!(fr_mins, expected);
    }

    #[test]
    fn we_can_get_single_rounded_packed_scalar_bit_size() {
        let bit_table = vec![1];
        let res = get_single_rounded_packed_scalar_bit_size(&bit_table);
        assert_eq!(res, 8);

        let bit_table = vec![8, 1];
        let res = get_single_rounded_packed_scalar_bit_size(&bit_table);
        assert_eq!(res, 16);

        let bit_table = vec![8, 1, 7];
        let res = get_single_rounded_packed_scalar_bit_size(&bit_table);
        assert_eq!(res, 16);

        let bit_table = vec![8, 1, 7, 15, 1, 3];
        let res = get_single_rounded_packed_scalar_bit_size(&bit_table);
        assert_eq!(res, 40);
    }

    #[test]
    fn we_can_get_max_column_length_of_same_type() {
        let committable_columns = vec![
            CommittableColumn::Scalar(vec![[1, 2, 3, 4], [5, 6, 7, 8]]),
            CommittableColumn::Scalar(vec![[1, 2, 3, 4]]),
        ];

        let max_column_length = get_max_column_length(&committable_columns);

        assert_eq!(max_column_length, 2);
    }

    #[test]
    fn we_can_get_max_column_length_of_different_types() {
        let committable_columns = vec![
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
    fn we_can_pack_empty_scalars() {
        let bit_table = vec![];
        let committable_columns = vec![];
        let (packed_scalar, packed_scalar_offsets) =
            get_packed_scalar_and_offset_scalar_offset(&bit_table, &committable_columns, 0, 1);

        assert!(packed_scalar.is_empty());
        assert!(packed_scalar_offsets.is_empty());
    }

    #[test]
    fn we_can_pack_scalars_with_one_full_row() {
        let bit_table = vec![64, 64];
        let committable_columns = vec![
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];
        let (packed_scalar, packed_scalar_offsets) =
            get_packed_scalar_and_offset_scalar_offset(&bit_table, &committable_columns, 0, 2);

        let expected_scalar: Vec<Vec<u8>> = vec![vec![
            1, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0, 0, 0, 0, 128, 2, 0, 0, 0, 0, 0, 0, 128, 4, 0, 0,
            0, 0, 0, 0, 128,
        ]];
        let expected_scalar_offsets: Vec<Vec<u8>> = vec![vec![
            1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0,
        ]];

        assert_eq!(packed_scalar, expected_scalar);
        assert_eq!(packed_scalar_offsets, expected_scalar_offsets);
    }

    #[test]
    fn we_can_pack_scalars_with_more_than_one_row() {
        let bit_table = vec![64, 64];
        let committable_columns = vec![
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];
        let (packed_scalar, packed_scalar_offsets) =
            get_packed_scalar_and_offset_scalar_offset(&bit_table, &committable_columns, 0, 1);

        let expected_scalar: Vec<Vec<u8>> = vec![
            vec![1, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0, 0, 0, 0, 128],
            vec![2, 0, 0, 0, 0, 0, 0, 128, 4, 0, 0, 0, 0, 0, 0, 128],
        ];
        let expected_scalar_offsets: Vec<Vec<u8>> = vec![
            vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
        ];

        assert_eq!(packed_scalar, expected_scalar);
        assert_eq!(packed_scalar_offsets, expected_scalar_offsets);
    }

    #[test]
    fn we_can_pack_scalars_with_one_full_row_with_offset() {
        let bit_table = vec![64, 64];
        let committable_columns = vec![
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];
        let (packed_scalar, packed_scalar_offsets) =
            get_packed_scalar_and_offset_scalar_offset(&bit_table, &committable_columns, 1, 3);

        let expected_scalar: Vec<Vec<u8>> = vec![vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0,
            0, 0, 0, 128, 2, 0, 0, 0, 0, 0, 0, 128, 4, 0, 0, 0, 0, 0, 0, 128,
        ]];
        let expected_scalar_offsets: Vec<Vec<u8>> = vec![vec![
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
        ]];

        assert_eq!(packed_scalar, expected_scalar);
        assert_eq!(packed_scalar_offsets, expected_scalar_offsets);
    }

    #[test]
    fn we_can_pack_scalars_with_more_than_one_row_with_offset() {
        let bit_table = vec![64, 64];
        let committable_columns = vec![
            CommittableColumn::BigInt(&[1, 2]),
            CommittableColumn::BigInt(&[3, 4]),
        ];
        let (packed_scalar, packed_scalar_offsets) =
            get_packed_scalar_and_offset_scalar_offset(&bit_table, &committable_columns, 1, 1);

        let expected_scalar: Vec<Vec<u8>> = vec![
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 0, 0, 0, 0, 0, 128, 3, 0, 0, 0, 0, 0, 0, 128],
            vec![2, 0, 0, 0, 0, 0, 0, 128, 4, 0, 0, 0, 0, 0, 0, 128],
        ];
        let expected_scalar_offsets: Vec<Vec<u8>> = vec![
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
            vec![1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
        ];

        assert_eq!(packed_scalar, expected_scalar);
        assert_eq!(packed_scalar_offsets, expected_scalar_offsets);
    }
}
