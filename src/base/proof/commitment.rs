use std::{
    ops::{Add, Sub},
    slice,
};

use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar, traits::Identity};
use pedersen::commitments::compute_commitments;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Commitment {
    //The actual commitment to a column/vector. It may make sense for this to be non compressed, and only serialized as compressed.
    pub commitment: CompressedRistretto,
    //The length of the column/vector.
    pub length: usize,
}

impl Add for Commitment {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.length, rhs.length);
        Commitment {
            commitment: (self.commitment.decompress().unwrap()
                + rhs.commitment.decompress().unwrap())
            .compress(),
            length: self.length,
        }
    }
}

impl Sub for Commitment {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        assert_eq!(self.length, rhs.length);
        Commitment {
            commitment: (self.commitment.decompress().unwrap()
                - rhs.commitment.decompress().unwrap())
            .compress(),
            length: self.length,
        }
    }
}

impl From<&[Scalar]> for Commitment {
    fn from(data: &[Scalar]) -> Self {
        let mut commitment = CompressedRistretto::identity();
        compute_commitments(slice::from_mut(&mut commitment), &[data]);
        Commitment {
            commitment,
            length: data.len(),
        }
    }
}
