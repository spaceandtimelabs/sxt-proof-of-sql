#![allow(non_snake_case)]

use crate::proof_primitive::inner_product::proof::*;

use crate::base::proof::Transcript;
use crate::base::scalar::inner_product;
use curve25519_dalek::ristretto::CompressedRistretto;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::traits::{Identity, VartimeMultiscalarMul};
use pedersen::commitments::get_generators;
use rand_core::SeedableRng;
use std::iter;

fn test_helper_create(n: usize) {
    let mut G = vec![CompressedRistretto::identity(); n + 1];
    get_generators(&mut G, 0);
    let Q = G[n].decompress().unwrap();
    let G: Vec<RistrettoPoint> = G.iter().take(n).map(|&x| x.decompress().unwrap()).collect();

    let mut rng = rand::rngs::StdRng::seed_from_u64(123);

    // a and b are the vectors for which we want to prove c = <a,b>
    let a: Vec<_> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
    let b: Vec<_> = (0..n).map(|_| Scalar::random(&mut rng)).collect();

    let mut transcript = Transcript::new(b"innerproducttest");
    let proof = InnerProductProof::create(&mut transcript, &Q, &G, &a, &b);

    // we can verify a valid proof
    let c = inner_product(&a, &b);
    let mut transcript = Transcript::new(b"innerproducttest");
    let P = RistrettoPoint::vartime_multiscalar_mul(
        a.iter().cloned().chain(iter::once(c)),
        G.iter().chain(iter::once(&Q)),
    );
    assert!(proof.verify(&mut transcript, &P, &Q, &G, &b).is_ok());

    // verification fails if the transcript doesn't match
    if n > 1 {
        let mut transcript = Transcript::new(b"invalid");
        assert!(!proof.verify(&mut transcript, &P, &Q, &G, &b).is_ok());
    }

    // verification fails if the inner product is incorrect
    let mut transcript = Transcript::new(b"innerproducttest");
    let c_plus_1 = c + Scalar::from(1u32);
    let P = RistrettoPoint::vartime_multiscalar_mul(
        a.iter().cloned().chain(iter::once(c_plus_1)),
        G.iter().chain(iter::once(&Q)),
    );
    assert!(!proof.verify(&mut transcript, &P, &Q, &G, &b).is_ok());

    // verification fails if a is different
    let mut transcript = Transcript::new(b"innerproducttest");
    let not_a: Vec<_> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
    let P = RistrettoPoint::vartime_multiscalar_mul(
        not_a.iter().cloned().chain(iter::once(c)),
        G.iter().chain(iter::once(&Q)),
    );
    assert!(!proof.verify(&mut transcript, &P, &Q, &G, &b).is_ok());

    // verification fails if b is different
    let mut transcript = Transcript::new(b"innerproducttest");
    let not_b: Vec<_> = (0..n).map(|_| Scalar::random(&mut rng)).collect();
    let P = RistrettoPoint::vartime_multiscalar_mul(
        a.iter().cloned().chain(iter::once(c)),
        G.iter().chain(iter::once(&Q)),
    );
    assert!(!proof.verify(&mut transcript, &P, &Q, &G, &not_b).is_ok());
}

#[test]
fn make_ipp_1() {
    test_helper_create(1);
}

#[test]
fn make_ipp_2() {
    test_helper_create(2);
}

#[test]
fn make_ipp_4() {
    test_helper_create(4);
}

#[test]
fn make_ipp_32() {
    test_helper_create(32);
}

#[test]
fn make_ipp_64() {
    test_helper_create(64);
}
