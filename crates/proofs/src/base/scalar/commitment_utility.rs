use crate::base::{
    polynomial::ArkScalar,
    slice_ops::{iter_cast, slice_cast_to_iter},
};
use blitzar::compute::compute_commitments;
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};

/// Compute the commitment of a sequence of values.
///
/// Computing commitments in isolation like this is inefficient so
/// this function shoud only be used for testing.
pub fn compute_commitment_for_testing<T: Into<ArkScalar> + Clone + Sync>(
    vals: &[T],
    offset_generators: usize,
) -> RistrettoPoint {
    let vals = iter_cast::<ArkScalar, [u64; 4]>(slice_cast_to_iter(vals));
    let table = [vals.as_slice().into()];
    let mut commitments = [CompressedRistretto::from_slice(&[0_u8; 32])];
    compute_commitments(&mut commitments, &table, offset_generators as u64);
    commitments[0].decompress().unwrap()
}
