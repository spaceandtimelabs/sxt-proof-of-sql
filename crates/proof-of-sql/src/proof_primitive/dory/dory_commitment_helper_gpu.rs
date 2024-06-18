use super::{pairings, DoryCommitment, DoryProverPublicSetup, DoryScalar, G1Affine};
use crate::base::commitment::CommittableColumn;
use ark_bls12_381::Fr;
use ark_ec::CurveGroup;
use ark_std::ops::Mul;
use blitzar::{compute::ElementP2, sequence::Sequence};
use num_traits::ToBytes;
use rayon::prelude::*;
use zerocopy::AsBytes;

trait OffsetToBytes {
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

#[tracing::instrument(name = "transpose_column (gpu)", level = "debug", skip_all)]
fn transpose_column<T: AsBytes + Copy + OffsetToBytes>(
    column: &[T],
    offset: usize,
    num_columns: usize,
    data_size: usize,
) -> Vec<u8> {
    let column_len_with_offset = column.len() + offset;
    let total_length_bytes =
        data_size * (((column_len_with_offset + num_columns - 1) / num_columns) * num_columns);
    let cols = num_columns;
    let rows = total_length_bytes / (data_size * cols);

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

#[tracing::instrument(name = "get_offset_commits (gpu)", level = "debug", skip_all)]
fn get_offset_commits(
    column_len: usize,
    offset: usize,
    num_columns: usize,
    num_of_commits: usize,
    scalar: Fr,
    setup: &DoryProverPublicSetup,
) -> Vec<G1Affine> {
    let first_row_offset = offset % num_columns;
    let first_row_len = column_len.min(num_columns - first_row_offset);
    let num_zero_commits = offset / num_columns;
    let data_size = 1;

    let ones = vec![1_u8; column_len];
    let (first_row, remaining_elements) = ones.split_at(first_row_len);

    let mut ones_blitzar_commits =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); num_of_commits];

    if num_zero_commits < num_of_commits {
        // Get the commit of the first non-zero row
        let first_row_offset = offset - (num_zero_commits * num_columns);
        let first_row_transpose =
            transpose_column(first_row, first_row_offset, num_columns, data_size);

        setup.public_parameters().blitzar_handle.msm(
            &mut ones_blitzar_commits[num_zero_commits..num_zero_commits + 1],
            data_size as u32,
            first_row_transpose.as_slice(),
        );

        // If there are more rows, get the commits of the middle row and duplicate them
        let mut chunks = remaining_elements.chunks(num_columns);
        if chunks.len() > 1 {
            if let Some(middle_row) = chunks.next() {
                let middle_row_transpose = transpose_column(middle_row, 0, num_columns, data_size);
                let mut middle_row_blitzar_commit =
                    vec![ElementP2::<ark_bls12_381::g1::Config>::default(); 1];

                setup.public_parameters().blitzar_handle.msm(
                    &mut middle_row_blitzar_commit,
                    data_size as u32,
                    middle_row_transpose.as_slice(),
                );

                ones_blitzar_commits[num_zero_commits + 1..num_of_commits - 1]
                    .par_iter_mut()
                    .for_each(|commit| *commit = middle_row_blitzar_commit[0].clone());
            }
        }

        // Get the commit of the last row to handle an zero padding at the end of the column
        if let Some(last_row) = remaining_elements.chunks(num_columns).last() {
            let last_row_transpose = transpose_column(last_row, 0, num_columns, data_size);

            setup.public_parameters().blitzar_handle.msm(
                &mut ones_blitzar_commits[num_of_commits - 1..num_of_commits],
                data_size as u32,
                last_row_transpose.as_slice(),
            );
        }
    }

    ones_blitzar_commits
        .par_iter()
        .map(Into::into)
        .map(|commit: G1Affine| commit.mul(scalar).into_affine())
        .collect()
}

#[tracing::instrument(name = "compute_dory_commitment_impl (gpu)", level = "debug", skip_all)]
fn compute_dory_commitment_impl<'a, T>(
    column: &'a [T],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment
where
    &'a T: Into<DoryScalar>,
    &'a [T]: Into<Sequence<'a>>,
    T: AsBytes + Copy + OffsetToBytes,
{
    let num_columns = 1 << setup.sigma();
    let data_size = std::mem::size_of::<T>();

    // Format column to match column major data layout required by blitzar's msm
    let column_transpose = transpose_column(column, offset, num_columns, data_size);
    let num_of_commits = column_transpose.len() / (data_size * num_columns);
    let gamma_2_slice = &setup.public_parameters().Gamma_2[0..num_of_commits];

    // Compute the commitment for the entire data set
    let mut blitzar_commits =
        vec![ElementP2::<ark_bls12_381::g1::Config>::default(); num_of_commits];
    setup.public_parameters().blitzar_handle.msm(
        &mut blitzar_commits,
        data_size as u32,
        column_transpose.as_slice(),
    );

    let commits: Vec<G1Affine> = blitzar_commits.par_iter().map(Into::into).collect();

    // Signed data requires offset commitments
    if T::IS_SIGNED {
        let offset_commits = get_offset_commits(
            column.len(),
            offset,
            num_columns,
            num_of_commits,
            T::min_as_fr(),
            setup,
        );

        DoryCommitment(
            pairings::multi_pairing(commits, gamma_2_slice)
                + pairings::multi_pairing(offset_commits, gamma_2_slice),
        )
    } else {
        DoryCommitment(pairings::multi_pairing(commits, gamma_2_slice))
    }
}

fn compute_dory_commitment(
    committable_column: &CommittableColumn,
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment {
    match committable_column {
        CommittableColumn::SmallInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::BigInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int128(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Decimal75(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
        CommittableColumn::Scalar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::VarChar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Boolean(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::TimestampTZ(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
    }
}

pub(super) fn compute_dory_commitments(
    committable_columns: &[CommittableColumn],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> Vec<DoryCommitment> {
    committable_columns
        .iter()
        .map(|column| compute_dory_commitment(column, offset, setup))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn we_can_transpose_empty_column() {
        type T = u64;
        let column: Vec<T> = vec![];
        let offset = 0;
        let num_columns = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_column(&column, offset, num_columns, data_size);

        assert_eq!(transpose.len(), expected_len);
        assert!(transpose.is_empty());
    }

    #[test]
    fn we_can_transpose_u64_column() {
        type T = u64;
        let column: Vec<T> = vec![0, 1, 2, 3];
        let offset = 0;
        let num_columns = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_column(&column, offset, num_columns, data_size);

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
        let num_columns = 3;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset + 1);

        let transpose = transpose_column(&column, offset, num_columns, data_size);

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
        let num_columns = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_column(&column, offset, num_columns, data_size);

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
        let num_columns = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_column(&column, offset, num_columns, data_size);

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
        let num_columns = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_column(&column, offset, num_columns, data_size);

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
        let num_columns = 2;
        let data_size = std::mem::size_of::<T>();

        let expected_len = data_size * (column.len() + offset);

        let transpose = transpose_column(&column, offset, num_columns, data_size);

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
}
