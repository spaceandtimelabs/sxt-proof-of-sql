use crate::{
    base::commitment::{Commitment, CommittableColumn},
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
};
use alloc::vec::Vec;
use curve25519_dalek::RistrettoPoint;

impl Commitment for RistrettoPoint {
    type Scalar = Curve25519Scalar;
    type PublicSetup<'a> = ();
    #[cfg(feature = "blitzar")]
    fn compute_commitments(
        committable_columns: &[CommittableColumn],
        offset: usize,
        _setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self> {
        use curve25519_dalek::ristretto::CompressedRistretto;

        let sequences: Vec<_> = committable_columns.iter().map(Into::into).collect();
        let mut compressed_commitments =
            vec![CompressedRistretto::default(); committable_columns.len()];
        blitzar::compute::compute_curve25519_commitments(
            &mut compressed_commitments,
            &sequences,
            offset as u64,
        );
        compressed_commitments
            .into_iter()
            .map(|cc| {
                cc.decompress().expect(
                    "invalid ristretto point decompression in Commitment::compute_commitments",
                )
            })
            .collect()
    }
    #[cfg(not(feature = "blitzar"))]
    fn compute_commitments(
        _committable_columns: &[CommittableColumn],
        _offset: usize,
        _setup: &Self::PublicSetup<'_>,
    ) -> Vec<Self> {
        unimplemented!()
    }

    fn to_transcript_bytes(&self) -> Vec<u8> {
        self.compress().as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::base::commitment::*;
    use curve25519_dalek::{constants::RISTRETTO_BASEPOINT_POINT, ristretto::RistrettoPoint};

    #[test]
    fn we_get_different_transcript_bytes_from_different_ristretto_point_commitments() {
        let commitment1 = RistrettoPoint::default();
        let commitment2 = RISTRETTO_BASEPOINT_POINT;

        assert_ne!(
            commitment1.to_transcript_bytes(),
            commitment2.to_transcript_bytes()
        );
    }
}
