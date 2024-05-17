use super::{DoryCommitment, DoryProverPublicSetup, DoryScalar, G1};
use crate::base::commitment::CommittableColumn;
use ark_ec::{pairing::Pairing, ScalarMul};
use ark_serialize::CanonicalDeserialize;
use blitzar::{compute::compute_bls12_381_g1_commitments_with_generators, sequence::Sequence};
use core::iter::once;

fn compute_dory_commitment_impl<'a, T>(
    column: &'a [T],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment
where
    &'a T: Into<DoryScalar>,
    &'a [T]: Into<Sequence<'a>>,
{
    // Compute offsets for the matrix.
    let num_columns = 1 << setup.sigma();
    let first_row_offset = offset % num_columns;
    let rows_offset = offset / num_columns;
    let first_row_len = column.len().min(num_columns - first_row_offset);
    let remaining_elements_len = column.len() - first_row_len;
    let remaining_row_count = (remaining_elements_len + num_columns - 1) / num_columns;

    // Break column into rows.
    let (first_row, remaining_elements) = column.split_at(first_row_len);
    let remaining_rows = remaining_elements.chunks(num_columns);

    // Calculate first commit.
    let mut commit = vec![[0_u8; 48]; 1];
    compute_bls12_381_g1_commitments_with_generators(
        &mut commit,
        &[first_row.into()],
        &ScalarMul::batch_convert_to_mul_base(
            &setup.public_parameters().Gamma_1[first_row_offset..num_columns],
        ),
    );

    // The bls12-381 G1 commitment from blitzar is compressed. we need to deserialize it to a projective point.
    let first_row_commit = G1::deserialize_compressed(&commit[0][..]).unwrap();

    // Create a sequence from the remaining chunks.
    let sequences: Vec<Sequence> = remaining_rows.clone().map(|chunk| chunk.into()).collect();

    // Compute the remaining commitments.
    let mut commits = vec![[0_u8; 48]; sequences.len()];
    compute_bls12_381_g1_commitments_with_generators(
        &mut commits,
        &sequences,
        &ScalarMul::batch_convert_to_mul_base(&setup.public_parameters().Gamma_1[..num_columns]),
    );

    // The bls12-381 G1 commitment from blitzar is compressed. we need to deserialize it to a projective point.
    let remaining_row_commits: Vec<_> = {
        commits
            .iter()
            .take(sequences.len())
            .map(|c| G1::deserialize_compressed(&c[..]).unwrap())
            .collect()
    };

    // Compute the commitment for the entire matrix.
    DoryCommitment(Pairing::multi_pairing(
        once(first_row_commit).chain(remaining_row_commits),
        &setup.public_parameters().Gamma_2[rows_offset..(rows_offset + remaining_row_count + 1)],
    ))
}

fn compute_dory_commitment(
    committable_column: &CommittableColumn,
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment {
    match committable_column {
        CommittableColumn::Scalar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::BigInt(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Int128(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Decimal75(_, _, column) => {
            compute_dory_commitment_impl(column, offset, setup)
        }
        CommittableColumn::VarChar(column) => compute_dory_commitment_impl(column, offset, setup),
        CommittableColumn::Boolean(column) => compute_dory_commitment_impl(column, offset, setup),
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
