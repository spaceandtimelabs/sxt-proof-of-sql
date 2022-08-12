use std::{
    ops::{Add, Neg, Sub},
    slice,
};

use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar, traits::Identity};
use pedersen::compute::{compute_commitments, update_commitment};

use super::Commit;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
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

impl Neg for Commitment {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Commitment {
            commitment: (-self.commitment.decompress().unwrap()).compress(),
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

impl Commitment {
    pub fn update_append_commitment(&self, a: &[Scalar]) -> Self {
        let mut commitment = self.commitment;
        let offset_generators = self.length;
        update_commitment(&mut commitment, offset_generators as u64, a);
        Commitment {
            commitment,
            length: a.len() + offset_generators,
        }
    }
    pub fn from_ones(length: usize) -> Self {
        super::Column::from(
            std::iter::repeat(Scalar::one())
                .take(length)
                .collect::<Vec<_>>(),
        )
        .commit()
    }
}
