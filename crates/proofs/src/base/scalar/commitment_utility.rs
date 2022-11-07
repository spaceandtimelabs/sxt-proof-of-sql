use super::IntoScalar;

use curve25519_dalek::{
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
};
use pedersen::compute::compute_commitments;

/// Compute the commitment of a sequence of values.
///
/// Computing commitments in isolation like this is inefficient so
/// this function shoud only be used for testing.
pub fn compute_commitment_for_testing<T: IntoScalar>(vals: &[T]) -> RistrettoPoint {
    let vals: Vec<Scalar> = vals.iter().map(|x| x.into_scalar()).collect();
    let table = [&vals[..]; 1];
    let mut commitments = [CompressedRistretto::from_slice(&[0_u8; 32])];
    compute_commitments(&mut commitments, &table);
    commitments[0].decompress().unwrap()
}
