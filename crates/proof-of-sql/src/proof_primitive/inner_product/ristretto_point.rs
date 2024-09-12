use curve25519_dalek::RistrettoPoint;

use crate::base::commitment::{Commitment, CommittableColumn};

use super::curve_25519_scalar::Curve25519Scalar;

impl Commitment for RistrettoPoint {
    type Scalar = Curve25519Scalar;
    type PublicSetup<'a> = ();
    #[cfg(feature = "blitzar")]
    fn compute_commitments(
        commitments: &mut [Self],
        committable_columns: &[CommittableColumn],
        offset: usize,
        _setup: &Self::PublicSetup<'_>,
    ) {
        let sequences = Vec::from_iter(committable_columns.iter().map(Into::into));
        let mut compressed_commitments = vec![Default::default(); committable_columns.len()];
        blitzar::compute::compute_curve25519_commitments(
            &mut compressed_commitments,
            &sequences,
            offset as u64,
        );
        commitments
            .iter_mut()
            .zip(compressed_commitments.iter())
            .for_each(|(c, cc)| {
                *c = cc.decompress().expect(
                    "invalid ristretto point decompression in Commitment::compute_commitments",
                );
            });
    }
    #[cfg(not(feature = "blitzar"))]
    fn compute_commitments(
        _commitments: &mut [Self],
        _committable_columns: &[CommittableColumn],
        _offset: usize,
        _setup: &Self::PublicSetup<'_>,
    ) {
        unimplemented!()
    }
}
