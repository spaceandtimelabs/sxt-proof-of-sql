use super::HyperKZGEngine;

/// Nova hyperkzg commitment corresponding to `HyperKZGCommitment`.
pub type NovaCommitment = nova_snark::provider::hyperkzg::Commitment<HyperKZGEngine>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            commitment::{Commitment, CommittableColumn},
            math::decimal::Precision,
            posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
            scalar::Scalar,
        },
        proof_primitive::hyperkzg::{
            nova_commitment_key_to_hyperkzg_public_setup, BNScalar, HyperKZGCommitment,
        },
    };
    use ark_bn254::G1Affine;
    use ff::Field;
    use itertools::Itertools;
    use nova_snark::{
        provider::{
            bn256_grumpkin::bn256::Scalar as NovaScalar,
            hyperkzg::{CommitmentEngine, CommitmentKey},
        },
        traits::commitment::CommitmentEngineTrait,
    };

    fn ark_to_nova_commitment(commitment: HyperKZGCommitment) -> NovaCommitment {
        NovaCommitment::new(
            blitzar::compute::convert_to_halo2_bn256_g1_affine(&G1Affine::from(
                commitment.commitment,
            ))
            .into(),
        )
    }

    fn compute_commitment_with_hyperkzg_repo<T: Into<BNScalar> + Clone>(
        setup: &CommitmentKey<HyperKZGEngine>,
        offset: usize,
        scalars: &[T],
    ) -> NovaCommitment {
        CommitmentEngine::commit(
            setup,
            &itertools::repeat_n(BNScalar::ZERO, offset)
                .chain(scalars.iter().map(Into::into))
                .map(Into::into)
                .collect_vec(),
            &NovaScalar::ZERO,
        )
    }

    #[test]
    fn we_can_compute_commitment_with_hyperkzg_repo_for_testing() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 6);

        let result = compute_commitment_with_hyperkzg_repo(&ck, 0, &[0]);

        assert_eq!(
            result,
            ark_to_nova_commitment((&G1Affine::default()).into())
        );
    }

    fn compute_expected_commitments(
        committable_columns: &[CommittableColumn],
        offset: usize,
        ck: &CommitmentKey<HyperKZGEngine>,
    ) -> Vec<NovaCommitment> {
        let mut expected: Vec<NovaCommitment> = Vec::with_capacity(committable_columns.len());
        for column in committable_columns {
            match column {
                CommittableColumn::Boolean(vals) => {
                    expected.push(compute_commitment_with_hyperkzg_repo(ck, offset, vals));
                }
                CommittableColumn::Uint8(vals) => {
                    expected.push(compute_commitment_with_hyperkzg_repo(ck, offset, vals));
                }
                CommittableColumn::TinyInt(vals) => {
                    expected.push(compute_commitment_with_hyperkzg_repo(ck, offset, vals));
                }
                CommittableColumn::SmallInt(vals) => {
                    expected.push(compute_commitment_with_hyperkzg_repo(ck, offset, vals));
                }
                CommittableColumn::Int(vals) => {
                    expected.push(compute_commitment_with_hyperkzg_repo(ck, offset, vals));
                }
                CommittableColumn::BigInt(vals) | CommittableColumn::TimestampTZ(_, _, vals) => {
                    expected.push(compute_commitment_with_hyperkzg_repo(ck, offset, vals));
                }
                CommittableColumn::Int128(vals) => {
                    expected.push(compute_commitment_with_hyperkzg_repo(ck, offset, vals));
                }
                CommittableColumn::Decimal75(_, _, vals)
                | CommittableColumn::Scalar(vals)
                | CommittableColumn::VarChar(vals)
                | CommittableColumn::VarBinary(vals) => {
                    expected.push(compute_commitment_with_hyperkzg_repo(ck, offset, vals));
                }
            }
        }
        expected
    }

    #[test]
    fn we_can_compute_expected_commitments_for_testing() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 6);

        let committable_columns = vec![CommittableColumn::BigInt(&[0; 0])];

        let offset = 0;

        let result = compute_expected_commitments(&committable_columns, offset, &ck);

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            ark_to_nova_commitment((&G1Affine::default()).into())
        );
    }

    #[test]
    fn we_can_compute_a_commitment_with_only_one_column() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 6);

        let committable_columns = vec![CommittableColumn::BigInt(&[0, 1, 2, 3, 4, 5, 6, 7])];

        let offset = 0;

        let res = HyperKZGCommitment::compute_commitments(
            &committable_columns,
            offset,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
        )
        .into_iter()
        .map(ark_to_nova_commitment)
        .collect::<Vec<_>>();
        let expected = compute_expected_commitments(&committable_columns, offset, &ck);

        assert_eq!(res, expected);
    }

    #[test]
    fn we_can_compute_commitments_with_a_single_empty_column() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 32);

        let committable_columns = vec![CommittableColumn::BigInt(&[0; 0])];

        for offset in 0..32 {
            let res = HyperKZGCommitment::compute_commitments(
                &committable_columns,
                offset,
                &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            )
            .into_iter()
            .map(ark_to_nova_commitment)
            .collect::<Vec<_>>();
            let expected = compute_expected_commitments(&committable_columns, offset, &ck);

            assert_eq!(res, expected, "Offset: {offset}");
        }
    }

    #[test]
    fn we_can_compute_commitments_with_a_multiple_mixed_empty_columns() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 32);

        let committable_columns = vec![
            CommittableColumn::TinyInt(&[0; 0]),
            CommittableColumn::SmallInt(&[0; 0]),
            CommittableColumn::Uint8(&[0; 0]),
            CommittableColumn::Int(&[0; 0]),
            CommittableColumn::BigInt(&[0; 0]),
            CommittableColumn::Int128(&[0; 0]),
        ];

        for offset in 0..32 {
            let res = HyperKZGCommitment::compute_commitments(
                &committable_columns,
                offset,
                &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            )
            .into_iter()
            .map(ark_to_nova_commitment)
            .collect::<Vec<_>>();
            let expected = compute_expected_commitments(&committable_columns, offset, &ck);

            assert_eq!(res, expected, "Offset: {offset}");
        }
    }

    #[test]
    fn we_can_compute_a_commitment_with_mixed_columns_of_different_sizes_and_offsets() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 128);

        let committable_columns = vec![
            CommittableColumn::BigInt(&[0, 1]),
            CommittableColumn::Uint8(&[2, 3]),
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
                PoSQLTimeZone::utc(),
                &[17, 18, 19, 20],
            ),
            CommittableColumn::VarBinary(vec![[21, 0, 0, 0]]),
        ];

        for offset in 0..64 {
            let res = HyperKZGCommitment::compute_commitments(
                &committable_columns,
                offset,
                &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            )
            .into_iter()
            .map(ark_to_nova_commitment)
            .collect::<Vec<_>>();
            let expected = compute_expected_commitments(&committable_columns, offset, &ck);

            assert_eq!(res, expected, "Offset: {offset}");
        }
    }

    #[test]
    fn we_can_compute_a_commitment_with_mixed_signed_columns_of_different_sizes_and_offsets() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 128);

        let committable_columns = vec![
            CommittableColumn::BigInt(&[-1, -2, -3]),
            CommittableColumn::Int(&[-4, -5, -10]),
            CommittableColumn::SmallInt(&[-6, -7]),
            CommittableColumn::Int128(&[-8, -9]),
        ];

        for offset in 0..60 {
            let res = HyperKZGCommitment::compute_commitments(
                &committable_columns,
                offset,
                &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            )
            .into_iter()
            .map(ark_to_nova_commitment)
            .collect::<Vec<_>>();
            let expected = compute_expected_commitments(&committable_columns, offset, &ck);

            assert_eq!(res, expected, "Offset: {offset}");
        }
    }
}
