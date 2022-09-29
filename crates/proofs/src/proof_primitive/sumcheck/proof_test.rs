/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::proof_primitive::sumcheck::proof::*;

use ark_std::rc::Rc;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::base::polynomial::{CompositePolynomial, DenseMultilinearExtension};
use crate::base::proof::{MessageLabel, TranscriptProtocol};

#[test]
fn test_create_verify_proof() {
    let num_vars = 1;
    let mut evaluation_point: [Scalar; 1] = [Scalar::zero(); 1];

    // create a proof
    let mut poly = CompositePolynomial::new(num_vars);
    let a_vec: [Scalar; 2] = [Scalar::from(123u64), Scalar::from(456u64)];
    let fa = Rc::new(DenseMultilinearExtension::from_evaluations_slice(
        num_vars, &a_vec,
    ));
    poly.add_product([fa], Scalar::from(1u64));
    let mut transcript = Transcript::new(b"sumchecktest");
    let mut proof = SumcheckProof::create(&mut transcript, &mut evaluation_point, &poly);

    // verify proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim = proof
        .verify_without_evaluation(&mut transcript, poly.info(), &Scalar::from(579u64))
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
        .verify_without_evaluation(&mut transcript, poly.info(), &Scalar::from(579u64))
        .expect("verify failed");
    assert_ne!(subclaim.evaluation_point, evaluation_point);

    // verify fails if sum is wrong
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim =
        proof.verify_without_evaluation(&mut transcript, poly.info(), &Scalar::from(123u64));
    assert!(subclaim.is_err());

    // verify fails if evaluations are changed
    proof.evaluations[0][1] += Scalar::from(3u64);
    let subclaim =
        proof.verify_without_evaluation(&mut transcript, poly.info(), &Scalar::from(579u64));
    assert!(subclaim.is_err());
}

fn random_product(
    nv: usize,
    num_multiplicands: usize,
    rng: &mut ThreadRng,
) -> (Vec<Rc<DenseMultilinearExtension>>, Scalar) {
    let mut multiplicands = Vec::with_capacity(num_multiplicands);
    for _ in 0..num_multiplicands {
        multiplicands.push(Vec::with_capacity(1 << nv))
    }
    let mut sum = Scalar::zero();

    for _ in 0..(1 << nv) {
        let mut product = Scalar::one();
        for i in 0..num_multiplicands {
            let val = Scalar::random(rng);
            multiplicands[i].push(val);
            product *= val;
        }
        sum += product;
    }

    (
        multiplicands
            .into_iter()
            .map(|x| Rc::new(DenseMultilinearExtension::from_evaluations_slice(nv, &x)))
            .collect(),
        sum,
    )
}

fn random_polynomial(
    nv: usize,
    num_multiplicands_range: (usize, usize),
    num_products: usize,
    rng: &mut ThreadRng,
) -> (CompositePolynomial, Scalar) {
    let mut sum = Scalar::zero();
    let mut poly = CompositePolynomial::new(nv);
    for _ in 0..num_products {
        let num_multiplicands = rng.gen_range(num_multiplicands_range.0, num_multiplicands_range.1);
        let (product, product_sum) = random_product(nv, num_multiplicands, rng);
        let coefficient = Scalar::random(rng);
        poly.add_product(product.into_iter(), coefficient);
        sum += product_sum * coefficient;
    }

    (poly, sum)
}

fn test_polynomial(nv: usize, num_multiplicands_range: (usize, usize), num_products: usize) {
    let mut rng = rand::thread_rng();
    let (poly, asserted_sum) =
        random_polynomial(nv, num_multiplicands_range, num_products, &mut rng);
    let poly_info = poly.info();

    // create a proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let mut evaluation_point = vec![Scalar::zero(); poly_info.num_variables];
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
