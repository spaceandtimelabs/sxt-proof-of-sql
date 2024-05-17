use crate::{
    base::commitment::CommittableColumn,
    proof_primitive::dory::{compute_dory_commitments, DoryProverPublicSetup, F, GT},
};
use ark_ec::pairing::Pairing;
use ark_std::test_rng;
use num_traits::Zero;

#[test]
fn we_can_compute_a_dory_commitment_with_only_one_row() {
    let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 1, 2])], 0, &setup);
    let Gamma_1 = &setup.public_parameters().Gamma_1;
    let Gamma_2 = &setup.public_parameters().Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(1)
        + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_exactly_one_full_row() {
    let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 1, 2, 3])], 0, &setup);
    let Gamma_1 = &setup.public_parameters().Gamma_1;
    let Gamma_2 = &setup.public_parameters().Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(1)
        + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2)
        + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(3);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_exactly_one_full_row_and_an_offset() {
    let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[2, 3])], 2, &setup);
    let Gamma_1 = &setup.public_parameters().Gamma_1;
    let Gamma_2 = &setup.public_parameters().Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2)
        + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(3);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_fewer_rows_than_columns() {
    let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
    let res = compute_dory_commitments(
        &[CommittableColumn::BigInt(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])],
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
        + Pairing::pairing(Gamma_1[1], Gamma_2[2]) * F::from(9);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_more_rows_than_columns() {
    let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
    let res = compute_dory_commitments(
        &[CommittableColumn::BigInt(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        ])],
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
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_an_offset_and_only_one_row() {
    let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 1])], 5, &setup);
    let Gamma_1 = &setup.public_parameters().Gamma_1;
    let Gamma_2 = &setup.public_parameters().Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[1], Gamma_2[1]) * F::from(0)
        + Pairing::pairing(Gamma_1[2], Gamma_2[1]) * F::from(1);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_an_offset_and_fewer_rows_than_columns() {
    let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
    let res = compute_dory_commitments(
        &[CommittableColumn::BigInt(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])],
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
        + Pairing::pairing(Gamma_1[2], Gamma_2[3]) * F::from(9);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_an_offset_and_more_rows_than_columns() {
    let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
    let res = compute_dory_commitments(
        &[CommittableColumn::BigInt(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        ])],
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
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_an_empty_dory_commitment() {
    let setup = DoryProverPublicSetup::rand(5, 2, &mut test_rng());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0; 0])], 0, &setup);
    assert_eq!(res[0].0, GT::zero());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0; 0])], 5, &setup);
    assert_eq!(res[0].0, GT::zero());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0; 0])], 20, &setup);
    assert_eq!(res[0].0, GT::zero());
}

#[test]
fn test_compute_dory_commitment_when_sigma_is_zero() {
    let setup = DoryProverPublicSetup::rand(5, 0, &mut test_rng());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 1, 2, 3, 4])], 0, &setup);
    let Gamma_1 = &setup.public_parameters().Gamma_1;
    let Gamma_2 = &setup.public_parameters().Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
        + Pairing::pairing(Gamma_1[0], Gamma_2[1]) * F::from(1)
        + Pairing::pairing(Gamma_1[0], Gamma_2[2]) * F::from(2)
        + Pairing::pairing(Gamma_1[0], Gamma_2[3]) * F::from(3)
        + Pairing::pairing(Gamma_1[0], Gamma_2[4]) * F::from(4);
    assert_eq!(res[0].0, expected);
}

#[test]
fn test_compute_dory_commitment_with_zero_sigma_and_with_an_offset() {
    let setup = DoryProverPublicSetup::rand(5, 0, &mut test_rng());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 1, 2, 3, 4])], 5, &setup);
    let Gamma_1 = &setup.public_parameters().Gamma_1;
    let Gamma_2 = &setup.public_parameters().Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[5]) * F::from(0)
        + Pairing::pairing(Gamma_1[0], Gamma_2[6]) * F::from(1)
        + Pairing::pairing(Gamma_1[0], Gamma_2[7]) * F::from(2)
        + Pairing::pairing(Gamma_1[0], Gamma_2[8]) * F::from(3)
        + Pairing::pairing(Gamma_1[0], Gamma_2[9]) * F::from(4);
    assert_eq!(res[0].0, expected);
}
