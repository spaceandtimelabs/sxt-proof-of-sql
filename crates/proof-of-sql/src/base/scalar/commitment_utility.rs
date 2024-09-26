use crate::base::{scalar::Curve25519Scalar, slice_ops::slice_cast};
use blitzar::compute::compute_curve25519_commitments;
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use blitzar::commitment::{Commitment, CommittableColumn};

/// Compute the commitment of a sequence of values.
///
/// Computing commitments in isolation like this is inefficient so
/// this function shoud only be used for testing.
pub fn compute_commitment_for_testing<T: Into<Curve25519Scalar> + Clone + Sync>(
    vals: &[T],
    offset_generators: usize,
) -> RistrettoPoint {
    let vals = slice_cast::<Curve25519Scalar, [u64; 4]>(&slice_cast(vals));
    let mut commitments = [RistrettoPoint::default()];
    RistrettoPoint::compute_commitments(
        &mut commitments,
        &[CommittableColumn::Scalar(vals.iter().cloned())],
        offset_generators,
        &(),
    );
    commitments[0]
}
