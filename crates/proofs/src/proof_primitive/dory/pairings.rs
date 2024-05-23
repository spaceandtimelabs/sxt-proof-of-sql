use super::{G1Affine, G2Affine, GT};
use ark_ec::pairing::Pairing;
#[tracing::instrument(
    name = "proofs.proof_primitive.dory.pairings.multi_pairing",
    level = "info",
    skip_all
)]
// This is a wrapper around multi_pairing_impl simply because tracing doesn't work well with threading.
pub fn multi_pairing(a: &[G1Affine], b: &[G2Affine]) -> GT {
    multi_pairing_impl(a, b)
}
#[tracing::instrument(
    name = "proofs.proof_primitive.dory.pairings.multi_pairing_2",
    level = "info",
    skip_all
)]
// This is a wrapper around multi_pairing_2_impl simply because tracing doesn't work well with threading.
pub fn multi_pairing_2(
    (a0, b0): (&[G1Affine], &[G2Affine]),
    (a1, b1): (&[G1Affine], &[G2Affine]),
) -> (GT, GT) {
    multi_pairing_2_impl((a0, b0), (a1, b1))
}
#[tracing::instrument(
    name = "proofs.proof_primitive.dory.pairings.multi_pairing_4",
    level = "info",
    skip_all
)]
// This is a wrapper around multi_pairing_4_impl simply because tracing doesn't work well with threading.
pub fn multi_pairing_4(
    (a0, b0): (&[G1Affine], &[G2Affine]),
    (a1, b1): (&[G1Affine], &[G2Affine]),
    (a2, b2): (&[G1Affine], &[G2Affine]),
    (a3, b3): (&[G1Affine], &[G2Affine]),
) -> (GT, GT, GT, GT) {
    multi_pairing_4_impl((a0, b0), (a1, b1), (a2, b2), (a3, b3))
}

fn multi_pairing_impl(a: &[G1Affine], b: &[G2Affine]) -> GT {
    Pairing::multi_pairing(a, b)
}
fn multi_pairing_2_impl(
    (a0, b0): (&[G1Affine], &[G2Affine]),
    (a1, b1): (&[G1Affine], &[G2Affine]),
) -> (GT, GT) {
    rayon::join(|| multi_pairing_impl(a0, b0), || multi_pairing_impl(a1, b1))
}
fn multi_pairing_4_impl(
    (a0, b0): (&[G1Affine], &[G2Affine]),
    (a1, b1): (&[G1Affine], &[G2Affine]),
    (a2, b2): (&[G1Affine], &[G2Affine]),
    (a3, b3): (&[G1Affine], &[G2Affine]),
) -> (GT, GT, GT, GT) {
    let ((c0, c1), (c2, c3)) = rayon::join(
        || multi_pairing_2_impl((a0, b0), (a1, b1)),
        || multi_pairing_2_impl((a2, b2), (a3, b3)),
    );
    (c0, c1, c2, c3)
}
