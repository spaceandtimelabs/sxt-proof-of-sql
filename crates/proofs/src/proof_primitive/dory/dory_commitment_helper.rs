use super::{DoryCommitment, DoryProverPublicSetup, DoryScalar, G1};
use crate::base::commitment::CommittableColumn;
use ark_ec::{pairing::Pairing, ScalarMul, VariableBaseMSM};
use core::iter::once;

fn compute_dory_commitment_impl<'a, T>(
    column: &'a [T],
    offset: usize,
    setup: &DoryProverPublicSetup,
) -> DoryCommitment
where
    &'a T: Into<DoryScalar>,
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

    // Compute commitments for the rows.
    let first_row_commit = G1::msm_unchecked(
        &ScalarMul::batch_convert_to_mul_base(
            &setup.public_parameters().Gamma_1[first_row_offset..num_columns],
        ),
        &Vec::from_iter(first_row.iter().map(|s| s.into().0)),
    );
    let remaining_row_commits = remaining_rows.map(|row| {
        G1::msm_unchecked(
            &ScalarMul::batch_convert_to_mul_base(
                &setup.public_parameters().Gamma_1[..num_columns],
            ),
            &Vec::from_iter(row.iter().map(|s| s.into().0)),
        )
    });

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

#[cfg(test)]
mod tests {
    use super::compute_dory_commitment_impl;
    use crate::proof_primitive::dory::{DoryProverPublicSetup, F, GT};
    use ark_ec::pairing::Pairing;
    use ark_std::test_rng;
    use num_traits::Zero;

    #[test]
    fn we_can_compute_a_dory_commitment_with_only_one_row() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
        let res = compute_dory_commitment_impl(&[0, 1, 2], 0, &setup);
        let Gamma_1 = &setup.public_parameters().Gamma_1;
        let Gamma_2 = &setup.public_parameters().Gamma_2;
        let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
            + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(1)
            + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2);
        assert_eq!(res.0, expected);
    }

    #[test]
    fn we_can_compute_a_dory_commitment_with_exactly_one_full_row() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
        let res = compute_dory_commitment_impl(&[0, 1, 2, 3], 0, &setup);
        let Gamma_1 = &setup.public_parameters().Gamma_1;
        let Gamma_2 = &setup.public_parameters().Gamma_2;
        let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
            + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(1)
            + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2)
            + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(3);
        assert_eq!(res.0, expected);
    }

    #[test]
    fn we_can_compute_a_dory_commitment_with_exactly_one_full_row_and_an_offset() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
        let res = compute_dory_commitment_impl(&[2, 3], 2, &setup);
        let Gamma_1 = &setup.public_parameters().Gamma_1;
        let Gamma_2 = &setup.public_parameters().Gamma_2;
        let expected: GT = Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2)
            + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(3);
        assert_eq!(res.0, expected);
    }

    #[test]
    fn we_can_compute_a_dory_commitment_with_fewer_rows_than_columns() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
        let res = compute_dory_commitment_impl(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 0, &setup);
        let Gamma_1 = &setup.public_parameters().Gamma_1;
        let Gamma_2 = &setup.public_parameters().Gamma_2;
        let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
            + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(1)
            + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2)
            + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(3)
            + Pairing::pairing(Gamma_1[0], Gamma_2[1]) * F::from(4)
            + Pairing::pairing(Gamma_1[1], Gamma_2[1]) * F::from(5)
            + Pairing::pairing(Gamma_1[2], Gamma_2[1]) * F::from(6)
            + Pairing::pairing(Gamma_1[3], Gamma_2[1]) * F::from(7)
            + Pairing::pairing(Gamma_1[0], Gamma_2[2]) * F::from(8)
            + Pairing::pairing(Gamma_1[1], Gamma_2[2]) * F::from(9);
        assert_eq!(res.0, expected);
    }
    #[test]
    fn we_can_compute_a_dory_commitment_with_more_rows_than_columns() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
        let res = compute_dory_commitment_impl(
            &[
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            ],
            0,
            &setup,
        );
        let Gamma_1 = &setup.public_parameters().Gamma_1;
        let Gamma_2 = &setup.public_parameters().Gamma_2;
        let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
            + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(1)
            + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2)
            + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(3)
            + Pairing::pairing(Gamma_1[0], Gamma_2[1]) * F::from(4)
            + Pairing::pairing(Gamma_1[1], Gamma_2[1]) * F::from(5)
            + Pairing::pairing(Gamma_1[2], Gamma_2[1]) * F::from(6)
            + Pairing::pairing(Gamma_1[3], Gamma_2[1]) * F::from(7)
            + Pairing::pairing(Gamma_1[0], Gamma_2[2]) * F::from(8)
            + Pairing::pairing(Gamma_1[1], Gamma_2[2]) * F::from(9)
            + Pairing::pairing(Gamma_1[2], Gamma_2[2]) * F::from(10)
            + Pairing::pairing(Gamma_1[3], Gamma_2[2]) * F::from(11)
            + Pairing::pairing(Gamma_1[0], Gamma_2[3]) * F::from(12)
            + Pairing::pairing(Gamma_1[1], Gamma_2[3]) * F::from(13)
            + Pairing::pairing(Gamma_1[2], Gamma_2[3]) * F::from(14)
            + Pairing::pairing(Gamma_1[3], Gamma_2[3]) * F::from(15)
            + Pairing::pairing(Gamma_1[0], Gamma_2[4]) * F::from(16)
            + Pairing::pairing(Gamma_1[1], Gamma_2[4]) * F::from(17)
            + Pairing::pairing(Gamma_1[2], Gamma_2[4]) * F::from(18);
        assert_eq!(res.0, expected);
    }

    #[test]
    fn we_can_compute_a_dory_commitment_with_an_offset_and_only_one_row() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
        let res = compute_dory_commitment_impl(&[0, 1], 5, &setup);
        let Gamma_1 = &setup.public_parameters().Gamma_1;
        let Gamma_2 = &setup.public_parameters().Gamma_2;
        let expected: GT = Pairing::pairing(Gamma_1[1], Gamma_2[1]) * F::from(0)
            + Pairing::pairing(Gamma_1[2], Gamma_2[1]) * F::from(1);
        assert_eq!(res.0, expected);
    }

    #[test]
    fn we_can_compute_a_dory_commitment_with_an_offset_and_fewer_rows_than_columns() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
        let res = compute_dory_commitment_impl(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 5, &setup);
        let Gamma_1 = &setup.public_parameters().Gamma_1;
        let Gamma_2 = &setup.public_parameters().Gamma_2;
        let expected: GT = Pairing::pairing(Gamma_1[1], Gamma_2[1]) * F::from(0)
            + Pairing::pairing(Gamma_1[2], Gamma_2[1]) * F::from(1)
            + Pairing::pairing(Gamma_1[3], Gamma_2[1]) * F::from(2)
            + Pairing::pairing(Gamma_1[0], Gamma_2[2]) * F::from(3)
            + Pairing::pairing(Gamma_1[1], Gamma_2[2]) * F::from(4)
            + Pairing::pairing(Gamma_1[2], Gamma_2[2]) * F::from(5)
            + Pairing::pairing(Gamma_1[3], Gamma_2[2]) * F::from(6)
            + Pairing::pairing(Gamma_1[0], Gamma_2[3]) * F::from(7)
            + Pairing::pairing(Gamma_1[1], Gamma_2[3]) * F::from(8)
            + Pairing::pairing(Gamma_1[2], Gamma_2[3]) * F::from(9);
        assert_eq!(res.0, expected);
    }

    #[test]
    fn we_can_compute_a_dory_commitment_with_an_offset_and_more_rows_than_columns() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
        let res = compute_dory_commitment_impl(
            &[
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
            ],
            5,
            &setup,
        );
        let Gamma_1 = &setup.public_parameters().Gamma_1;
        let Gamma_2 = &setup.public_parameters().Gamma_2;
        let expected: GT = Pairing::pairing(Gamma_1[1], Gamma_2[1]) * F::from(0)
            + Pairing::pairing(Gamma_1[2], Gamma_2[1]) * F::from(1)
            + Pairing::pairing(Gamma_1[3], Gamma_2[1]) * F::from(2)
            + Pairing::pairing(Gamma_1[0], Gamma_2[2]) * F::from(3)
            + Pairing::pairing(Gamma_1[1], Gamma_2[2]) * F::from(4)
            + Pairing::pairing(Gamma_1[2], Gamma_2[2]) * F::from(5)
            + Pairing::pairing(Gamma_1[3], Gamma_2[2]) * F::from(6)
            + Pairing::pairing(Gamma_1[0], Gamma_2[3]) * F::from(7)
            + Pairing::pairing(Gamma_1[1], Gamma_2[3]) * F::from(8)
            + Pairing::pairing(Gamma_1[2], Gamma_2[3]) * F::from(9)
            + Pairing::pairing(Gamma_1[3], Gamma_2[3]) * F::from(10)
            + Pairing::pairing(Gamma_1[0], Gamma_2[4]) * F::from(11)
            + Pairing::pairing(Gamma_1[1], Gamma_2[4]) * F::from(12)
            + Pairing::pairing(Gamma_1[2], Gamma_2[4]) * F::from(13)
            + Pairing::pairing(Gamma_1[3], Gamma_2[4]) * F::from(14)
            + Pairing::pairing(Gamma_1[0], Gamma_2[5]) * F::from(15)
            + Pairing::pairing(Gamma_1[1], Gamma_2[5]) * F::from(16)
            + Pairing::pairing(Gamma_1[2], Gamma_2[5]) * F::from(17)
            + Pairing::pairing(Gamma_1[3], Gamma_2[5]) * F::from(18);
        assert_eq!(res.0, expected);
    }

    #[test]
    fn we_can_compute_an_empty_dory_commitment() {
        let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
        let res = compute_dory_commitment_impl(&[0; 0], 0, &setup);
        assert_eq!(res.0, GT::zero());
        let res = compute_dory_commitment_impl(&[0; 0], 5, &setup);
        assert_eq!(res.0, GT::zero());
        let res = compute_dory_commitment_impl(&[0; 0], 20, &setup);
        assert_eq!(res.0, GT::zero());
    }

    #[test]
    fn test_compute_dory_commitment_when_sigma_is_zero() {
        let setup = DoryProverPublicSetup::rand(5, 0, &mut test_rng());
        let res = compute_dory_commitment_impl(&[0, 1, 2, 3, 4], 0, &setup);
        let Gamma_1 = &setup.public_parameters().Gamma_1;
        let Gamma_2 = &setup.public_parameters().Gamma_2;
        let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
            + Pairing::pairing(Gamma_1[0], Gamma_2[1]) * F::from(1)
            + Pairing::pairing(Gamma_1[0], Gamma_2[2]) * F::from(2)
            + Pairing::pairing(Gamma_1[0], Gamma_2[3]) * F::from(3)
            + Pairing::pairing(Gamma_1[0], Gamma_2[4]) * F::from(4);
        assert_eq!(res.0, expected);
    }

    #[test]
    fn test_compute_dory_commitment_with_zero_sigma_and_with_an_offset() {
        let setup = DoryProverPublicSetup::rand(5, 0, &mut test_rng());
        let res = compute_dory_commitment_impl(&[0, 1, 2, 3, 4], 5, &setup);
        let Gamma_1 = &setup.public_parameters().Gamma_1;
        let Gamma_2 = &setup.public_parameters().Gamma_2;
        let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[5]) * F::from(0)
            + Pairing::pairing(Gamma_1[0], Gamma_2[6]) * F::from(1)
            + Pairing::pairing(Gamma_1[0], Gamma_2[7]) * F::from(2)
            + Pairing::pairing(Gamma_1[0], Gamma_2[8]) * F::from(3)
            + Pairing::pairing(Gamma_1[0], Gamma_2[9]) * F::from(4);
        assert_eq!(res.0, expected);
    }
}
