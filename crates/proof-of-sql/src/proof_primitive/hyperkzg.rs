use crate::base::{
    commitment::{Commitment, CommitmentEvaluationProof, CommittableColumn},
    proof::{Keccak256Transcript, Transcript},
    scalar::{MontScalar, Scalar},
    slice_ops,
};
use alloc::vec::Vec;
#[cfg(feature = "blitzar")]
use ark_bn254::G1Affine;
#[cfg(feature = "blitzar")]
use blitzar;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
use ff::Field;
#[cfg(not(feature = "blitzar"))]
use itertools::Itertools;
use nova_snark::{
    errors::NovaError,
    provider::{
        bn256_grumpkin::bn256::Scalar as NovaScalar,
        hyperkzg::{CommitmentKey, EvaluationArgument, EvaluationEngine, VerifierKey},
    },
    traits::{
        evaluation::EvaluationEngineTrait, Engine, TranscriptEngineTrait, TranscriptReprTrait,
    },
};
#[cfg(not(feature = "blitzar"))]
use nova_snark::{provider::hyperkzg::CommitmentEngine, traits::commitment::CommitmentEngineTrait};
use serde::{Deserialize, Serialize};
use tracing::{span, Level};

/// The scalar used in the `HyperKZG` PCS. This is the BN254 scalar.
pub type BNScalar = MontScalar<ark_bn254::FrConfig>;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
/// The `HyperKZG` engine that implements nova's `Engine` trait.
pub struct HyperKZGEngine;

type NovaCommitment = nova_snark::provider::hyperkzg::Commitment<HyperKZGEngine>;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
/// A newtype wrapper of nova's hyperkzg commitment.
/// This is the commitment type used in the hyperkzg proof system.
pub struct HyperKZGCommitment {
    /// The underlying commitment.
    pub commitment: NovaCommitment,
}

/// The evaluation proof for the `HyperKZG` PCS.
pub type HyperKZGCommitmentEvaluationProof = EvaluationArgument<HyperKZGEngine>;

impl AddAssign for HyperKZGCommitment {
    fn add_assign(&mut self, rhs: Self) {
        self.commitment = self.commitment + rhs.commitment;
    }
}
#[cfg(feature = "blitzar")]
impl From<&G1Affine> for HyperKZGCommitment {
    fn from(value: &G1Affine) -> Self {
        Self {
            commitment: NovaCommitment::new(
                blitzar::compute::convert_to_halo2_bn256_g1_affine(value).into(),
            ),
        }
    }
}
impl From<&BNScalar> for NovaScalar {
    fn from(value: &BNScalar) -> Self {
        ff::PrimeField::from_repr_vartime(bytemuck::cast::<[u64; 4], [u8; 32]>(value.into()).into())
            .unwrap()
    }
}
impl Mul<&HyperKZGCommitment> for BNScalar {
    type Output = HyperKZGCommitment;
    fn mul(self, rhs: &HyperKZGCommitment) -> Self::Output {
        Self::Output {
            commitment: rhs.commitment * NovaScalar::from(self),
        }
    }
}
impl From<BNScalar> for NovaScalar {
    fn from(value: BNScalar) -> Self {
        Self::from(&value)
    }
}
impl Mul<HyperKZGCommitment> for BNScalar {
    type Output = HyperKZGCommitment;
    #[expect(clippy::op_ref)]
    fn mul(self, rhs: HyperKZGCommitment) -> Self::Output {
        self * &rhs
    }
}
impl Neg for HyperKZGCommitment {
    type Output = Self;
    fn neg(self) -> Self::Output {
        (-BNScalar::ONE) * self
    }
}
impl SubAssign for HyperKZGCommitment {
    fn sub_assign(&mut self, rhs: Self) {
        *self += -rhs;
    }
}
impl Sub for HyperKZGCommitment {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

#[cfg(not(feature = "blitzar"))]
#[tracing::instrument(name = "compute_commitments_impl (cpu)", level = "debug", skip_all)]
fn compute_commitments_impl<T: Into<BNScalar> + Clone>(
    setup: &CommitmentKey<HyperKZGEngine>,
    offset: usize,
    scalars: &[T],
) -> HyperKZGCommitment {
    assert!(offset + scalars.len() <= setup.ck().len());
    let product = scalars
        .iter()
        .zip(&setup.ck()[offset..offset + scalars.len()])
        .map(|(t, s)| *s * Into::<NovaScalar>::into(Into::<BNScalar>::into(t)))
        .sum();
    HyperKZGCommitment {
        commitment: NovaCommitment::new(product),
    }
}
impl Commitment for HyperKZGCommitment {
    type Scalar = BNScalar;
    type PublicSetup<'a> = &'a CommitmentKey<HyperKZGEngine>;

    #[cfg(not(feature = "blitzar"))]
    #[tracing::instrument(name = "compute_commitments (cpu)", level = "debug", skip_all)]
    fn compute_commitments(
        committable_columns: &[crate::base::commitment::CommittableColumn],
        offset: usize,
        setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self> {
        committable_columns
            .iter()
            .map(|column| match column {
                CommittableColumn::Boolean(vals) => compute_commitments_impl(setup, offset, vals),
                CommittableColumn::Uint8(vals) => compute_commitments_impl(setup, offset, vals),
                CommittableColumn::TinyInt(vals) => compute_commitments_impl(setup, offset, vals),
                CommittableColumn::SmallInt(vals) => compute_commitments_impl(setup, offset, vals),
                CommittableColumn::Int(vals) => compute_commitments_impl(setup, offset, vals),
                CommittableColumn::BigInt(vals) | CommittableColumn::TimestampTZ(_, _, vals) => {
                    compute_commitments_impl(setup, offset, vals)
                }
                CommittableColumn::Int128(vals) => compute_commitments_impl(setup, offset, vals),
                CommittableColumn::Decimal75(_, _, vals)
                | CommittableColumn::Scalar(vals)
                | CommittableColumn::VarChar(vals)
                | CommittableColumn::VarBinary(vals) => {
                    compute_commitments_impl(setup, offset, vals)
                }
                CommittableColumn::FixedSizeBinary(_, items) => {
                    compute_commitments_impl(setup, offset, items)
                }
            })
            .collect()
    }

    #[cfg(feature = "blitzar")]
    #[tracing::instrument(name = "compute_commitments (gpu)", level = "debug", skip_all)]
    fn compute_commitments(
        committable_columns: &[crate::base::commitment::CommittableColumn],
        offset: usize,
        setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self> {
        if committable_columns.is_empty() {
            return Vec::new();
        }

        // Find the maximum length of the columns to get number of generators to use
        let max_column_len = committable_columns
            .iter()
            .map(CommittableColumn::len)
            .max()
            .expect("You must have at least one column");

        let mut blitzar_commitments = vec![G1Affine::default(); committable_columns.len()];

        blitzar::compute::compute_bn254_g1_uncompressed_commitments_with_generators(
            &mut blitzar_commitments,
            &slice_ops::slice_cast(committable_columns),
            &slice_ops::slice_cast_with(
                &setup.ck()[offset..offset + max_column_len],
                blitzar::compute::convert_to_ark_bn254_g1_affine,
            ),
        );

        slice_ops::slice_cast(&blitzar_commitments)
    }

    fn to_transcript_bytes(&self) -> Vec<u8> {
        self.commitment.to_transcript_bytes()
    }
}

impl Engine for HyperKZGEngine {
    type Base = nova_snark::provider::bn256_grumpkin::bn256::Base;
    type Scalar = NovaScalar;
    type GE = nova_snark::provider::bn256_grumpkin::bn256::Point;
    type RO = nova_snark::provider::poseidon::PoseidonRO<Self::Base, Self::Scalar>;
    type ROCircuit = nova_snark::provider::poseidon::PoseidonROCircuit<Self::Base>;
    type TE = Keccak256Transcript;
    type CE = nova_snark::provider::hyperkzg::CommitmentEngine<Self>;
}

impl TranscriptEngineTrait<HyperKZGEngine> for Keccak256Transcript {
    fn new(_label: &'static [u8]) -> Self {
        Transcript::new()
    }

    fn squeeze(&mut self, _label: &'static [u8]) -> Result<NovaScalar, NovaError> {
        Ok(Transcript::scalar_challenge_as_be::<BNScalar>(self).into())
    }

    fn absorb<T: TranscriptReprTrait<<HyperKZGEngine as Engine>::GE>>(
        &mut self,
        _label: &'static [u8],
        o: &T,
    ) {
        Transcript::extend_as_le_from_refs(
            self,
            o.to_transcript_bytes()
                .chunks(32)
                // Reverse the bytes in each 32 byte chunk, making them effectivelly big-endian
                .flat_map(|chunk| chunk.iter().rev()),
        );
    }

    fn dom_sep(&mut self, _bytes: &'static [u8]) {}
}

impl CommitmentEvaluationProof for HyperKZGCommitmentEvaluationProof {
    type Scalar = BNScalar;
    type Commitment = HyperKZGCommitment;
    type Error = NovaError;
    type ProverPublicSetup<'a> = &'a CommitmentKey<HyperKZGEngine>;
    type VerifierPublicSetup<'a> = &'a VerifierKey<HyperKZGEngine>;

    fn new(
        transcript: &mut impl crate::base::proof::Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        setup: &Self::ProverPublicSetup<'_>,
    ) -> Self {
        assert_eq!(generators_offset, 0);
        let mut nova_point = slice_ops::slice_cast(b_point);
        nova_point.reverse();
        if nova_point.is_empty() {
            nova_point.push(NovaScalar::ZERO);
        }
        let mut nova_a = slice_ops::slice_cast(a);
        nova_a.extend(itertools::repeat_n(
            NovaScalar::ZERO,
            (1 << nova_point.len()) - nova_a.len(),
        ));
        transcript.wrap_transcript(|keccak_transcript| {
            let span = span!(Level::DEBUG, "EvaluationEngine::prove").entered();
            let eval_eng = EvaluationEngine::prove(
                *setup,
                &EvaluationEngine::setup(*setup).0, // This parameter is unused
                keccak_transcript,
                &NovaCommitment::default(), // This parameter is unused
                &nova_a,
                &nova_point,
                &NovaScalar::default(), // This parameter is unused
            )
            .unwrap();
            span.exit();
            eval_eng
        })
    }

    fn verify_batched_proof(
        &self,
        transcript: &mut impl crate::base::proof::Transcript,
        commit_batch: &[Self::Commitment],
        batching_factors: &[Self::Scalar],
        evaluations: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _table_length: usize,
        setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        if generators_offset != 0 {
            Err(NovaError::InvalidPCS)?;
        }
        let nova_commit = commit_batch
            .iter()
            .zip(batching_factors)
            .map(|(c, m)| c.commitment * NovaScalar::from(m))
            .fold(NovaCommitment::default(), Add::add);
        let nova_eval = evaluations
            .iter()
            .zip(batching_factors)
            .map(|(&e, &f)| e * f)
            .sum::<Self::Scalar>();
        let mut nova_point = slice_ops::slice_cast(b_point);
        nova_point.reverse();
        if nova_point.is_empty() {
            nova_point.push(NovaScalar::ZERO);
        }
        transcript.wrap_transcript(|keccak_transcript| {
            EvaluationEngine::verify(
                setup,
                keccak_transcript,
                &nova_commit,
                &nova_point,
                &nova_eval.into(),
                self,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "blitzar")]
    use crate::base::math::decimal::Precision;
    #[cfg(feature = "blitzar")]
    use crate::base::posql_time::{PoSQLTimeUnit, PoSQLTimeZone};
    use crate::base::{
        commitment::commitment_evaluation_proof_test::{
            test_commitment_evaluation_proof_with_length_1,
            test_random_commitment_evaluation_proof, test_simple_commitment_evaluation_proof,
        },
        scalar::test_scalar_constants,
    };
    #[cfg(feature = "blitzar")]
    use ark_ec::AffineRepr;
    use ark_std::UniformRand;
    #[cfg(feature = "blitzar")]
    use itertools::Itertools;
    use nova_snark::{
        provider::hyperkzg::CommitmentEngine, traits::commitment::CommitmentEngineTrait,
    };

    #[cfg(feature = "blitzar")]
    #[test]
    fn we_can_commit_fixed_size_binary_columns_for_coverage() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 16);
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80];
        let width = 4.try_into().unwrap();
        let binding = data.clone();
        let committable_columns = vec![CommittableColumn::FixedSizeBinary(width, &binding)];
        for offset in 0..4 {
            let result =
                HyperKZGCommitment::compute_commitments(&committable_columns, offset, &&ck);
            let expected = compute_expected_commitments(&committable_columns, offset, &ck);
            assert_eq!(result, expected, "Offset: {offset}");
        }
    }

    #[test]
    fn we_have_correct_constants_for_bn_scalar() {
        test_scalar_constants::<BNScalar>();
    }

    #[test]
    fn we_can_convert_from_posql_scalar_to_nova_scalar() {
        // Test zero
        assert_eq!(NovaScalar::from(0_u64), NovaScalar::from(BNScalar::ZERO));

        // Test one
        assert_eq!(NovaScalar::from(1_u64), NovaScalar::from(BNScalar::ONE));

        // Test negative one
        assert_eq!(-NovaScalar::from(1_u64), NovaScalar::from(-BNScalar::ONE));

        // Test two
        assert_eq!(NovaScalar::from(2_u64), NovaScalar::from(BNScalar::TWO));

        // Test ten
        assert_eq!(NovaScalar::from(10_u64), NovaScalar::from(BNScalar::TEN));

        // Test a large value
        let large_value = BNScalar::from(123_456_789_u64);
        assert_eq!(
            NovaScalar::from(123_456_789_u64),
            NovaScalar::from(large_value)
        );

        let mut rng = ark_std::test_rng();

        for _ in 0..10 {
            let a = BNScalar::rand(&mut rng);
            let b = BNScalar::rand(&mut rng);
            assert_eq!(
                NovaScalar::from(a + b),
                NovaScalar::from(a) + NovaScalar::from(b)
            );
            assert_eq!(
                NovaScalar::from(a * b),
                NovaScalar::from(a) * NovaScalar::from(b)
            );
        }
    }

    #[test]
    fn we_can_create_small_hyperkzg_evaluation_proofs() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 32);
        let (_, vk) = EvaluationEngine::setup(&ck);
        test_simple_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(&&ck, &&vk);
        test_commitment_evaluation_proof_with_length_1::<HyperKZGCommitmentEvaluationProof>(
            &&ck, &&vk,
        );
    }

    #[test]
    fn we_can_create_hyperkzg_evaluation_proofs_with_various_lengths() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 128);
        let (_, vk) = EvaluationEngine::setup(&ck);
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            2, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            3, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            4, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            5, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            8, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            10, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            16, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            20, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            32, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            50, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            64, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            100, 0, &&ck, &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            128, 0, &&ck, &&vk,
        );
    }

    #[cfg(feature = "blitzar")]
    fn compute_commitment_with_hyperkzg_repo<T: Into<BNScalar> + Clone>(
        setup: &CommitmentKey<HyperKZGEngine>,
        offset: usize,
        scalars: &[T],
    ) -> HyperKZGCommitment {
        let commitment = CommitmentEngine::commit(
            setup,
            &itertools::repeat_n(BNScalar::ZERO, offset)
                .chain(scalars.iter().map(Into::into))
                .map(Into::into)
                .collect_vec(),
            &NovaScalar::ZERO,
        );
        HyperKZGCommitment { commitment }
    }

    #[cfg(feature = "blitzar")]
    #[test]
    fn we_can_compute_commitment_with_hyperkzg_repo_for_testing() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 6);

        let result = compute_commitment_with_hyperkzg_repo(&ck, 0, &[0]);

        assert_eq!(result, (&G1Affine::default()).into());
    }

    #[cfg(feature = "blitzar")]
    fn compute_expected_commitments(
        committable_columns: &[CommittableColumn],
        offset: usize,
        ck: &CommitmentKey<HyperKZGEngine>,
    ) -> Vec<HyperKZGCommitment> {
        let mut expected: Vec<HyperKZGCommitment> = Vec::with_capacity(committable_columns.len());
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
                CommittableColumn::FixedSizeBinary(_, items) => {
                    expected.push(compute_commitment_with_hyperkzg_repo(ck, offset, items));
                }
            }
        }
        expected
    }

    #[cfg(feature = "blitzar")]
    #[test]
    fn we_can_compute_expected_commitments_for_testing() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 6);

        let committable_columns = vec![CommittableColumn::BigInt(&[0; 0])];

        let offset = 0;

        let result = compute_expected_commitments(&committable_columns, offset, &ck);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].commitment, NovaCommitment::default());
    }

    #[cfg(feature = "blitzar")]
    #[test]
    fn we_can_compute_a_commitment_with_only_one_column() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 6);

        let committable_columns = vec![CommittableColumn::BigInt(&[0, 1, 2, 3, 4, 5, 6, 7])];

        let offset = 0;

        let res = HyperKZGCommitment::compute_commitments(&committable_columns, offset, &&ck);
        let expected = compute_expected_commitments(&committable_columns, offset, &ck);

        assert_eq!(res, expected);
    }

    #[cfg(feature = "blitzar")]
    #[test]
    fn we_can_compute_commitments_with_a_single_empty_column() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 32);

        let committable_columns = vec![CommittableColumn::BigInt(&[0; 0])];

        for offset in 0..32 {
            let res = HyperKZGCommitment::compute_commitments(&committable_columns, offset, &&ck);
            let expected = compute_expected_commitments(&committable_columns, offset, &ck);

            assert_eq!(res, expected, "Offset: {offset}");
        }
    }

    #[cfg(feature = "blitzar")]
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
            let res = HyperKZGCommitment::compute_commitments(&committable_columns, offset, &&ck);
            let expected = compute_expected_commitments(&committable_columns, offset, &ck);

            assert_eq!(res, expected, "Offset: {offset}");
        }
    }

    #[cfg(feature = "blitzar")]
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
            let res = HyperKZGCommitment::compute_commitments(&committable_columns, offset, &&ck);
            let expected = compute_expected_commitments(&committable_columns, offset, &ck);

            assert_eq!(res, expected, "Offset: {offset}");
        }
    }

    #[cfg(feature = "blitzar")]
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
            let res = HyperKZGCommitment::compute_commitments(&committable_columns, offset, &&ck);
            let expected = compute_expected_commitments(&committable_columns, offset, &ck);

            assert_eq!(res, expected, "Offset: {offset}");
        }
    }

    #[cfg(feature = "blitzar")]
    #[test]
    fn we_can_convert_default_point_to_a_hyperkzg_commitment_from_ark_bn254_g1_affine() {
        let commitment: HyperKZGCommitment = HyperKZGCommitment::from(&G1Affine::default());
        assert_eq!(commitment.commitment, NovaCommitment::default());
    }

    #[cfg(feature = "blitzar")]
    #[test]
    fn we_can_convert_generator_to_a_hyperkzg_commitment_from_ark_bn254_g1_affine() {
        let commitment: HyperKZGCommitment = (&G1Affine::generator()).into();
        let expected: HyperKZGCommitment = HyperKZGCommitment::from(&G1Affine::generator());
        assert_eq!(commitment.commitment, expected.commitment);
    }
}
