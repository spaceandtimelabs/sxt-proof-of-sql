use super::{BNScalar, HyperKZGPublicSetup};
use crate::base::{
    commitment::{Commitment, CommittableColumn},
    impl_serde_for_ark_serde_checked,
    scalar::Scalar,
    slice_ops,
};
use alloc::vec::Vec;
use ark_bn254::{G1Affine, G1Projective};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use core::ops::{AddAssign, Mul, Neg, Sub, SubAssign};

/// This is the commitment type used in the hyperkzg proof system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CanonicalSerialize, CanonicalDeserialize, Default)]
pub struct HyperKZGCommitment {
    /// The underlying commitment.
    pub commitment: G1Projective,
}
impl_serde_for_ark_serde_checked!(HyperKZGCommitment);

impl AddAssign for HyperKZGCommitment {
    fn add_assign(&mut self, rhs: Self) {
        self.commitment = self.commitment + rhs.commitment;
    }
}
impl From<&G1Affine> for HyperKZGCommitment {
    fn from(value: &G1Affine) -> Self {
        Self {
            commitment: (*value).into(),
        }
    }
}

impl Mul<&HyperKZGCommitment> for BNScalar {
    type Output = HyperKZGCommitment;
    fn mul(self, rhs: &HyperKZGCommitment) -> Self::Output {
        Self::Output {
            commitment: rhs.commitment * self.0,
        }
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
    setup: HyperKzgPublicSetup<'_>,
    offset: usize,
    scalars: &[T],
) -> HyperKZGCommitment {
    assert!(offset + scalars.len() <= setup.len());
    let product: G1Projective = scalars
        .iter()
        .zip(&setup[offset..offset + scalars.len()])
        .map(|(t, s)| *s * Into::<BNScalar>::into(t).0)
        .sum();
    HyperKZGCommitment {
        commitment: G1Projective::from(product),
    }
}
impl Commitment for HyperKZGCommitment {
    type Scalar = BNScalar;
    type PublicSetup<'a> = HyperKZGPublicSetup<'a>;

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
            &setup[offset..offset + max_column_len],
        );

        slice_ops::slice_cast(&blitzar_commitments)
    }

    fn to_transcript_bytes(&self) -> Vec<u8> {
        let mut writer = Vec::with_capacity(self.commitment.compressed_size());
        self.commitment.serialize_compressed(&mut writer).unwrap();
        writer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_ec::AffineRepr;

    #[test]
    fn we_can_convert_default_point_to_a_hyperkzg_commitment_from_ark_bn254_g1_affine() {
        let commitment: HyperKZGCommitment = HyperKZGCommitment::from(&G1Affine::default());
        assert_eq!(commitment.commitment, G1Affine::default());
    }

    #[test]
    fn we_can_convert_generator_to_a_hyperkzg_commitment_from_ark_bn254_g1_affine() {
        let commitment: HyperKZGCommitment = (&G1Affine::generator()).into();
        let expected: HyperKZGCommitment = HyperKZGCommitment::from(&G1Affine::generator());
        assert_eq!(commitment.commitment, expected.commitment);
    }
}
