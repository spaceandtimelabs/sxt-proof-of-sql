use crate::base::{
    polynomial::CompositePolynomial,
    proof::{MessageLabel, TranscriptProtocol},
    scalar::Curve25519Scalar,
};
/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::proof_primitive::sumcheck::proof::*;
use ark_std::UniformRand;
use merlin::Transcript;
use num_traits::{One, Zero};
use std::rc::Rc;

#[test]
fn test_create_verify_proof() {
    let num_vars = 1;
    let mut evaluation_point: [Curve25519Scalar; 1] = [Curve25519Scalar::zero(); 1];

    // create a proof
    let mut poly = CompositePolynomial::new(num_vars);
    let a_vec: [Curve25519Scalar; 2] = [
        Curve25519Scalar::from(123u64),
        Curve25519Scalar::from(456u64),
    ];
    let fa = Rc::new(a_vec.to_vec());
    poly.add_product([fa], Curve25519Scalar::from(1u64));
    let mut transcript = Transcript::new(b"sumchecktest");
    let mut proof = SumcheckProof::create(&mut transcript, &mut evaluation_point, &poly);

    // verify proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim = proof
        .verify_without_evaluation(
            &mut transcript,
            poly.info(),
            &Curve25519Scalar::from(579u64),
        )
        .expect("verify failed");
    assert_eq!(subclaim.evaluation_point, evaluation_point);
    assert_eq!(
        poly.evaluate(&evaluation_point),
        subclaim.expected_evaluation
    );

    // we return a different evaluation point if we start with a different transcript
    let mut transcript = Transcript::new(b"sumchecktest");
    transcript.append_auto(MessageLabel::SumcheckChallenge, &123u64);
    let subclaim = proof
        .verify_without_evaluation(
            &mut transcript,
            poly.info(),
            &Curve25519Scalar::from(579u64),
        )
        .expect("verify failed");
    assert_ne!(subclaim.evaluation_point, evaluation_point);

    // verify fails if sum is wrong
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim = proof.verify_without_evaluation(
        &mut transcript,
        poly.info(),
        &Curve25519Scalar::from(123u64),
    );
    assert!(subclaim.is_err());

    // verify fails if evaluations are changed
    proof.evaluations[0][1] += Curve25519Scalar::from(3u64);
    let subclaim = proof.verify_without_evaluation(
        &mut transcript,
        poly.info(),
        &Curve25519Scalar::from(579u64),
    );
    assert!(subclaim.is_err());
}

fn random_product(
    nv: usize,
    num_multiplicands: usize,
    rng: &mut ark_std::rand::rngs::StdRng,
) -> (Vec<Rc<Vec<Curve25519Scalar>>>, Curve25519Scalar) {
    let mut multiplicands = Vec::with_capacity(num_multiplicands);
    for _ in 0..num_multiplicands {
        multiplicands.push(Vec::with_capacity(1 << nv))
    }
    let mut sum = Curve25519Scalar::zero();

    for _ in 0..(1 << nv) {
        let mut product = Curve25519Scalar::one();
        for multiplicand in multiplicands.iter_mut().take(num_multiplicands) {
            let val = Curve25519Scalar::rand(rng);
            multiplicand.push(val);
            product *= val;
        }
        sum += product;
    }

    (multiplicands.into_iter().map(Rc::new).collect(), sum)
}

fn random_polynomial(
    nv: usize,
    num_multiplicands_range: (usize, usize),
    num_products: usize,
    rng: &mut ark_std::rand::rngs::StdRng,
) -> (CompositePolynomial<Curve25519Scalar>, Curve25519Scalar) {
    use ark_std::rand::Rng;
    let mut sum = Curve25519Scalar::zero();
    let mut poly = CompositePolynomial::new(nv);
    for _ in 0..num_products {
        let num_multiplicands = rng.gen_range(num_multiplicands_range.0..num_multiplicands_range.1);
        let (product, product_sum) = random_product(nv, num_multiplicands, rng);
        let coefficient = Curve25519Scalar::rand(rng);
        poly.add_product(product.into_iter(), coefficient);
        sum += product_sum * coefficient;
    }

    (poly, sum)
}

fn test_polynomial(nv: usize, num_multiplicands_range: (usize, usize), num_products: usize) {
    let mut rng = <ark_std::rand::rngs::StdRng as ark_std::rand::SeedableRng>::from_seed([0u8; 32]);
    let (poly, asserted_sum) =
        random_polynomial(nv, num_multiplicands_range, num_products, &mut rng);
    let poly_info = poly.info();

    // create a proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let mut evaluation_point = vec![Curve25519Scalar::zero(); poly_info.num_variables];
    let proof = SumcheckProof::create(&mut transcript, &mut evaluation_point, &poly);

    // verify proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim = proof
        .verify_without_evaluation(&mut transcript, poly_info, &asserted_sum)
        .expect("verify failed");
    assert_eq!(subclaim.evaluation_point, evaluation_point);
    assert_eq!(
        poly.evaluate(&evaluation_point),
        subclaim.expected_evaluation
    );
}

#[test]
fn test_trivial_polynomial() {
    let nv = 1;
    let num_multiplicands_range = (4, 13);
    let num_products = 5;

    test_polynomial(nv, num_multiplicands_range, num_products);
}

#[test]
fn test_normal_polynomial() {
    let nv = 7;
    let num_multiplicands_range = (4, 9);
    let num_products = 5;

    test_polynomial(nv, num_multiplicands_range, num_products);
}
