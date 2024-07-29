use crate::{
    base::{commitment::CommittableColumn, math::decimal::Precision},
    proof_primitive::dory::{
        compute_dory_commitments, DoryProverPublicSetup, ProverSetup, PublicParameters, F, GT,
    },
};
use ark_ec::pairing::Pairing;
use ark_std::test_rng;
use num_traits::Zero;
use proof_of_sql_parser::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};

#[test]
fn we_can_compute_a_dory_commitment_with_int128_values() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(&[CommittableColumn::Int128(&[0, -1, 2])], 0, &setup);
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0_i128)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(-1_i128)
        + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2_i128);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_boolean_values() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(
        &[CommittableColumn::Boolean(&[true, false, true])],
        0,
        &setup,
    );
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(true)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(false)
        + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(true);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_only_one_row() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 1, 2])], 0, &setup);
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(1)
        + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_exactly_one_full_row() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 1, 2, 3])], 0, &setup);
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(1)
        + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2)
        + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(3);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_exactly_one_full_row_and_an_offset() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[2, 3])], 2, &setup);
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(2)
        + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(3);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_exactly_one_full_row_and_an_offset_with_signed_data() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[-2, -3])], 2, &setup);
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(-2)
        + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(-3);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_fewer_rows_than_columns() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(
        &[CommittableColumn::BigInt(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])],
        0,
        &setup,
    );
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
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
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(
        &[CommittableColumn::BigInt(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        ])],
        0,
        &setup,
    );
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
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
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 1])], 5, &setup);
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[1], Gamma_2[1]) * F::from(0)
        + Pairing::pairing(Gamma_1[2], Gamma_2[1]) * F::from(1);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_an_offset_and_fewer_rows_than_columns() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(
        &[CommittableColumn::BigInt(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])],
        5,
        &setup,
    );
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
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
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(
        &[CommittableColumn::BigInt(&[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
        ])],
        5,
        &setup,
    );
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
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
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 2);
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 0])], 0, &setup);
    assert_eq!(res[0].0, GT::zero());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 0])], 5, &setup);
    assert_eq!(res[0].0, GT::zero());
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 0])], 20, &setup);
    assert_eq!(res[0].0, GT::zero());
}

#[test]
fn test_compute_dory_commitment_when_sigma_is_zero() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 0);
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 1, 2, 3, 4])], 0, &setup);
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
        + Pairing::pairing(Gamma_1[0], Gamma_2[1]) * F::from(1)
        + Pairing::pairing(Gamma_1[0], Gamma_2[2]) * F::from(2)
        + Pairing::pairing(Gamma_1[0], Gamma_2[3]) * F::from(3)
        + Pairing::pairing(Gamma_1[0], Gamma_2[4]) * F::from(4);
    assert_eq!(res[0].0, expected);
}

#[test]
fn test_compute_dory_commitment_with_zero_sigma_and_with_an_offset() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 0);
    let res = compute_dory_commitments(&[CommittableColumn::BigInt(&[0, 1, 2, 3, 4])], 5, &setup);
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[5]) * F::from(0)
        + Pairing::pairing(Gamma_1[0], Gamma_2[6]) * F::from(1)
        + Pairing::pairing(Gamma_1[0], Gamma_2[7]) * F::from(2)
        + Pairing::pairing(Gamma_1[0], Gamma_2[8]) * F::from(3)
        + Pairing::pairing(Gamma_1[0], Gamma_2[9]) * F::from(4);
    assert_eq!(res[0].0, expected);
}

#[test]
fn we_can_compute_a_dory_commitment_with_mixed_committable_columns() {
    let public_parameters = PublicParameters::rand(5, &mut test_rng());
    let prover_setup = ProverSetup::from(&public_parameters);
    let setup = DoryProverPublicSetup::new(&prover_setup, 5);
    let res = compute_dory_commitments(
        &[
            CommittableColumn::BigInt(&[0, 1]),
            CommittableColumn::BigInt(&[2, 3]),
            CommittableColumn::Int(&[4, 5, 10]),
            CommittableColumn::SmallInt(&[6, 7]),
            CommittableColumn::Int128(&[8, 9]),
            CommittableColumn::Boolean(&[true, true]),
            CommittableColumn::Decimal75(
                Precision::new(1).unwrap(),
                0,
                vec![[10, 0, 0, 0], [11, 0, 0, 0], [12, 0, 0, 0], [13, 0, 0, 0]],
            ),
            CommittableColumn::Scalar(vec![[14, 0, 0, 0], [15, 0, 0, 0]]),
            CommittableColumn::VarChar(vec![[16, 0, 0, 0]]),
            CommittableColumn::TimestampTZ(
                PoSQLTimeUnit::Second,
                PoSQLTimeZone::Utc,
                &[17, 18, 19, 20, 21],
            ),
        ],
        0,
        &setup,
    );
    let Gamma_1 = public_parameters.Gamma_1;
    let Gamma_2 = public_parameters.Gamma_2;
    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(0)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(1);
    assert_eq!(res[0].0, expected);

    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(2)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(3);
    assert_eq!(res[1].0, expected);

    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(4)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(5)
        + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(10);
    assert_eq!(res[2].0, expected);

    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(6)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(7);
    assert_eq!(res[3].0, expected);

    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(8)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(9);
    assert_eq!(res[4].0, expected);

    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(true)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(true);
    assert_eq!(res[5].0, expected);

    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(10)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(11)
        + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(12)
        + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(13);
    assert_eq!(res[6].0, expected);

    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(14)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(15);
    assert_eq!(res[7].0, expected);

    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(16);
    assert_eq!(res[8].0, expected);

    let expected: GT = Pairing::pairing(Gamma_1[0], Gamma_2[0]) * F::from(17)
        + Pairing::pairing(Gamma_1[1], Gamma_2[0]) * F::from(18)
        + Pairing::pairing(Gamma_1[2], Gamma_2[0]) * F::from(19)
        + Pairing::pairing(Gamma_1[3], Gamma_2[0]) * F::from(20)
        + Pairing::pairing(Gamma_1[4], Gamma_2[0]) * F::from(21);
    assert_eq!(res[9].0, expected);
}
