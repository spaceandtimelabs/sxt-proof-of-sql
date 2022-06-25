#![allow(non_snake_case)]

use std::slice;

use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::Identity;
use pedersen::commitments::compute_commitments;
use crate::base::proof::{Commitment, PIPProof, Transcript};
use crate::pip::equality::EqualityProof;

#[test]
    fn test_equality() {
        let a = vec![
            Scalar::from(1 as u32),
            Scalar::from(1 as u32),
            Scalar::from(1 as u32),
            Scalar::from(1 as u32),
            Scalar::from(2 as u32),
            Scalar::from(2 as u32),
            Scalar::from(2 as u32)
        ];
        let b = vec![
            Scalar::from(1 as u32),
            Scalar::from(1 as u32),
            Scalar::from(2 as u32),
            Scalar::from(3 as u32),
            Scalar::from(2 as u32),
            Scalar::from(2 as u32),
            Scalar::from(2 as u32)
        ];

        let output = vec![
            Scalar::from(1 as u32),
            Scalar::from(1 as u32),
            Scalar::from(0 as u32),
            Scalar::from(0 as u32),
            Scalar::from(1 as u32),
            Scalar::from(1 as u32),
            Scalar::from(1 as u32)
        ];

        let mut C_a = CompressedRistretto::identity();
        compute_commitments(slice::from_mut(&mut C_a), &[&a[..]]);
        let commitment_a = Commitment {
            commitment: C_a,
            length: a.len(),
        };

        let mut C_b = CompressedRistretto::identity();
        compute_commitments(slice::from_mut(&mut C_b), &[&b[..]]);
        let commitment_b = Commitment {
            commitment: C_b,
            length: b.len(),
        };

        let mut transcript = Transcript::new(b"equalitytest");
        let equalityproof = EqualityProof::create(&mut transcript, &[&a, &b], &[&output], &[]);
        assert!(equalityproof.verify(&mut transcript, &[commitment_a, commitment_b]).is_ok());
    }
