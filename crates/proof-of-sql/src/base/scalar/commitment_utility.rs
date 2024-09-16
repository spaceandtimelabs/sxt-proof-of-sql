use super::Scalar;
use crate::{
    base::{
        commitment::naive_commitment::NaiveCommitment,
        scalar::test_scalar::TestScalar,
        slice_ops::{iter_cast, slice_cast_to_iter},
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
};
use blitzar::compute::compute_curve25519_commitments;
use curve25519_dalek::{ristretto::CompressedRistretto, RistrettoPoint};

/// Compute the commitment of a sequence of values.
///
/// Computing commitments in isolation like this is inefficient so
/// this function shoud only be used for testing.
pub fn compute_compressed_ristretto_commitment_for_testing<
    T: Into<Curve25519Scalar> + Clone + Sync,
>(
    vals: &[T],
    offset_generators: usize,
) -> RistrettoPoint {
    let vals = iter_cast::<Curve25519Scalar, [u64; 4]>(slice_cast_to_iter(vals));
    let table = [vals.as_slice().into()];
    let mut commitments = [CompressedRistretto::default()];
    compute_curve25519_commitments(&mut commitments, &table, offset_generators as u64);
    commitments[0].decompress().unwrap()
}

/// Compute the commitment of a sequence of values.
///
/// Computing commitments in isolation like this is inefficient so
/// this function shoud only be used for testing.
pub fn compute_commitment_for_testing<T: Into<TestScalar> + Clone + Sync>(
    vals: &[T],
    offset_generators: usize,
) -> NaiveCommitment {
    let mut offset_scalars: Vec<TestScalar> = vec![TestScalar::ZERO; offset_generators];
    let mut scalars: Vec<TestScalar> = vals.iter().map(|val| val.into()).collect();
    offset_scalars.append(&mut scalars);
    NaiveCommitment(offset_scalars)
}
