use crate::base::{
    commitment::{Commitment, CommitmentEvaluationProof, CommittableColumn},
    proof::{Keccak256Transcript, Transcript},
    scalar::{MontScalar, Scalar},
    slice_ops,
};
use alloc::vec::Vec;
use ark_bn254::g1::G1Affine;
use ark_ec::AffineRepr;
use ark_ff::PrimeField;
#[cfg(feature = "blitzar")]
use blitzar;
use byte_slice_cast::AsByteSlice;
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
use ff::Field;
use itertools::Itertools;
#[cfg(not(feature = "blitzar"))]
use nova_snark::provider::hyperkzg::CommitmentEngine;
#[cfg(not(feature = "blitzar"))]
use nova_snark::traits::commitment::{CommitmentEngineTrait, CommitmentTrait};
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
    #[allow(clippy::op_ref)]
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
fn compute_commitments_impl_cpu<T: Into<BNScalar> + Clone>(
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
fn convert_to_ark_bn254_g1_affine(
    point: nova_snark::provider::bn256_grumpkin::bn256::Affine,
) -> ark_bn254::G1Affine {
    ark_bn254::G1Affine::new(
        ark_bn254::Fq::from_le_bytes_mod_order(&point.x.to_bytes()),
        ark_bn254::Fq::from_le_bytes_mod_order(&point.y.to_bytes()),
    )
}

#[cfg(feature = "blitzar")]
/// Converts a `NovaScalar` to an array of four `u64` values.
///
/// This function takes a `NovaScalar`, converts it to a 32-byte array, and then splits
/// that array into four `u64` values.
///
/// # Panics
///
/// This function will panic if:
/// - The scalar is not 32 bytes long.
/// - The conversion from bytes to `u64` fails.
///
/// # Arguments
///
/// * `scalar` - A `NovaScalar` to be converted.
///
/// # Returns
///
/// An array of four `u64` values representing the `NovaScalar`.
fn convert_to_u64_array(scalar: NovaScalar) -> [u64; 4] {
    let bytes: [u8; 32] = scalar.to_bytes();
    let mut array = [0u64; 4];
    for i in 0..4 {
        array[i] = u64::from_le_bytes(bytes[i * 8..(i + 1) * 8].try_into().unwrap());
    }
    array
}

#[cfg(feature = "blitzar")]
/// Converts an `ark_bn254::G1Affine` point to a `nova_snark::provider::bn256_grumpkin::bn256::Affine` point.
///
/// # Panics
///
/// This function will panic if:
/// - The x or y coordinates of the point are not 32 bytes long.
/// - The conversion from bytes to `nova_snark::provider::bn256_grumpkin::bn256::Base` fails.
fn convert_to_nova_g1_affine(
    point: ark_bn254::G1Affine,
) -> nova_snark::provider::bn256_grumpkin::bn256::Affine {
    let Some(x) = point.x() else {
        return nova_snark::provider::bn256_grumpkin::bn256::Affine::default();
    };
    let Some(y) = point.y() else {
        return nova_snark::provider::bn256_grumpkin::bn256::Affine::default();
    };
    let x = x.into_bigint();
    let y = y.into_bigint();
    let x_bytes: &[u8; 32] = x
        .as_byte_slice()
        .try_into()
        .expect("Failed to convert x coordinate to bytes");
    let y_bytes: &[u8; 32] = y
        .as_byte_slice()
        .try_into()
        .expect("Failed to convert y coordinate to bytes");

    nova_snark::provider::bn256_grumpkin::bn256::Affine {
        x: nova_snark::provider::bn256_grumpkin::bn256::Base::from_bytes(x_bytes)
            .expect("Failed to convert x coordinate to bytes"),
        y: nova_snark::provider::bn256_grumpkin::bn256::Base::from_bytes(y_bytes)
            .expect("Failed to convert y coordinate to bytes"),
    }
}

#[cfg(feature = "blitzar")]
fn write_vec<T: Into<BNScalar> + Clone>(offset: usize, scalars: &[T]) -> Vec<BNScalar> {
    itertools::repeat_n(BNScalar::ZERO, offset)
        .chain(scalars.iter().map(Into::into))
        .map(Into::into)
        .chain(std::iter::once(BNScalar::ZERO))
        .collect_vec()
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

        let max_column_len = committable_columns
            .iter()
            .map(CommittableColumn::len)
            .max()
            .unwrap_or(0)
            + offset;

        let mut v: Vec<Vec<BNScalar>> = Vec::with_capacity(committable_columns.len());
        for column in committable_columns {
            let mut column_vec: Vec<BNScalar> = Vec::with_capacity(column.len());

            match column {
                CommittableColumn::Boolean(vals) => {
                    column_vec.extend(write_vec(offset, vals));
                }
                CommittableColumn::Uint8(vals) => {
                    column_vec.extend(write_vec(offset, vals));
                }
                CommittableColumn::TinyInt(vals) => {
                    column_vec.extend(write_vec(offset, vals));
                }
                CommittableColumn::SmallInt(vals) => {
                    column_vec.extend(write_vec(offset, vals));
                }
                CommittableColumn::Int(vals) => {
                    column_vec.extend(write_vec(offset, vals));
                }
                CommittableColumn::BigInt(vals) | CommittableColumn::TimestampTZ(_, _, vals) => {
                    column_vec.extend(write_vec(offset, vals));
                }
                CommittableColumn::Int128(vals) => {
                    column_vec.extend(write_vec(offset, vals));
                }
                CommittableColumn::Decimal75(_, _, vals)
                | CommittableColumn::Scalar(vals)
                | CommittableColumn::VarChar(vals) => {
                    column_vec.extend(write_vec(offset, vals));
                }
            }
            v.push(column_vec);
        }

        // Get the scalars
        let blitzar_scalar_vec: Vec<Vec<[u64; 4]>> = v
            .iter()
            .map(|x| {
                x.iter()
                    .map(|x| convert_to_u64_array((*x).into()))
                    .collect::<Vec<[u64; 4]>>()
            })
            .collect();

        let blitzar_sequence: Vec<blitzar::sequence::Sequence> =
            blitzar_scalar_vec.iter().map(Into::into).collect();

        // Get bases from the setup
        let blitzar_bases_slice: Vec<G1Affine> = setup.ck()[..max_column_len]
            .iter()
            .chain(std::iter::once(setup.h()))
            .map(|x| convert_to_ark_bn254_g1_affine(*x))
            .collect::<Vec<_>>();

        let mut blitzar_commitments = vec![G1Affine::default(); committable_columns.len()];

        blitzar::compute::compute_bn254_g1_uncompressed_commitments_with_generators(
            &mut blitzar_commitments,
            &blitzar_sequence,
            &blitzar_bases_slice,
        );

        blitzar_commitments
            .into_iter()
            .map(|commitment| HyperKZGCommitment {
                commitment: NovaCommitment::from_g1_affine(convert_to_nova_g1_affine(commitment)),
            })
            .collect()
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
    use crate::base::{
        commitment::commitment_evaluation_proof_test::{
            test_commitment_evaluation_proof_with_length_1,
            test_random_commitment_evaluation_proof, test_simple_commitment_evaluation_proof,
        },
        scalar::test_scalar_constants,
    };
    use ark_std::UniformRand;
    use nova_snark::{
        provider::hyperkzg::CommitmentEngine, traits::commitment::CommitmentEngineTrait,
    };

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
}
