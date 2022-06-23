#[derive(Clone, Copy)]
pub struct Commitment {
    //The actual commitment to a column/vector. It may make sense for this to be non compressed, and only serialized as compressed.
    pub commitment: curve25519_dalek::ristretto::CompressedRistretto,
    //The length of the column/vector.
    pub length: usize,
}
