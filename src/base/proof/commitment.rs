use std::{
    ops::{Add, Neg, Sub},
    slice,
};

use crate::base::scalar::SafeInt;

use curve25519_dalek::{
    ristretto::{CompressedRistretto, RistrettoPoint},
    scalar::Scalar,
    traits::Identity,
};
use pedersen::compute::{compute_commitments, update_commitment};

use super::{Commit, ProofError, ProofResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Commitment {
    //The actual commitment to a column/vector. It may make sense for this to be non compressed, and only serialized as compressed.
    commitment: CompressedRistretto,
    //The length of the column/vector.
    pub length: usize,
    /// Keeps track of the log_max value for commitments of SafeInt columns.
    /// See [crate::base::scalar::SafeIntColumn] for more details.
    pub log_max: Option<u8>,
}

/// Similar to SafeInt, Commitment equality ignores the log_max value.
impl PartialEq for Commitment {
    fn eq(&self, other: &Self) -> bool {
        self.commitment == other.commitment && self.length == other.length
    }
}

impl Eq for Commitment {}

impl Add for Commitment {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.length, rhs.length);
        Commitment {
            commitment: (self.commitment.decompress().unwrap()
                + rhs.commitment.decompress().unwrap())
            .compress(),
            length: self.length,
            log_max: match (self.log_max, rhs.log_max) {
                (Some(a), Some(b)) => match a.max(b).checked_add(1) {
                    Some(log_max) if log_max <= SafeInt::LOG_MAX_MAX => Some(log_max),
                    _ => {
                        panic!("possible overflow, add a range check upstream")
                    }
                },
                (None, None) => None,
                _ => panic!("cannot add commitments with and without log_max values together"),
            },
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
            log_max: match (self.log_max, rhs.log_max) {
                (Some(a), Some(b)) => match a.max(b).checked_add(1) {
                    Some(log_max) if log_max <= SafeInt::LOG_MAX_MAX => Some(log_max),
                    _ => {
                        panic!("possible overflow, add a range check upstream")
                    }
                },
                (None, None) => None,
                _ => panic!("cannot sub commitments with and without log_max values together"),
            },
        }
    }
}

impl Neg for Commitment {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Commitment {
            commitment: (-self.commitment.decompress().unwrap()).compress(),
            ..self
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
            log_max: None,
        }
    }
}


impl Commitment {
    /// Returns a decompressed version of the commitment.
    ///
    /// Panics if the compressed point is invalid.
    pub fn try_as_decompressed(&self) -> ProofResult<RistrettoPoint> {
        self.commitment
            .decompress()
            .ok_or(ProofError::DecompressionError)
    }

    /// Returns a compressed version of the commitment.
    pub fn as_compressed(&self) -> CompressedRistretto {
        self.commitment
    }

    /// Creates a Commitment from a compressed point.
    ///
    /// Panics if the compressed point is invalid.
    pub fn from_compressed(compressed: CompressedRistretto, length: usize) -> Self {
        let c = Commitment {
            commitment: compressed,
            length,
            log_max: None,
        };
        assert!(c.commitment.decompress().is_some());
        c
    }

    pub fn update_append_commitment(&self, a: &[Scalar]) -> Self {
        let mut commitment = self.commitment;
        let offset_generators = self.length;
        update_commitment(&mut commitment, offset_generators as u64, a);
        Commitment {
            commitment,
            length: a.len() + offset_generators,
            log_max: None,
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

    /// Returns this [Commitment], but with the provided log_max value
    pub fn with_log_max(self, log_max: u8) -> Commitment {
        Commitment {
            log_max: Some(log_max),
            ..self
        }
    }

    /// Returns this [Commitment], but with no log_max value
    pub fn without_log_max(self) -> Commitment {
        Commitment {
            log_max: None,
            ..self
        }
    }
}
