use crate::base::scalar::ToArkScalar;
use crate::base::slice_ops;
/**
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::proof_primitive::sumcheck::proof::*;

use crate::base::polynomial::ArkScalar;
use ark_std::rc::Rc;
use merlin::Transcript;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::base::polynomial::{CompositePolynomial, DenseMultilinearExtension};
use crate::base::proof::{MessageLabel, TranscriptProtocol};

#[test]
fn test_create_verify_proof() {
    let num_vars = 1;
    let mut evaluation_point: [ArkScalar; 1] = [ArkScalar::zero(); 1];

    // create a proof
    let mut poly = CompositePolynomial::new(num_vars);
    let a_vec: [ArkScalar; 2] = [ArkScalar::from(123u64), ArkScalar::from(456u64)];
    let fa = Rc::new(a_vec.to_vec());
    poly.add_product([fa], ArkScalar::from(1u64));
    let mut transcript = Transcript::new(b"sumchecktest");
    let mut proof = SumcheckProof::create(&mut transcript, &mut evaluation_point, &poly);

    // verify proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim = proof
        .verify_without_evaluation(&mut transcript, poly.info(), &ArkScalar::from(579u64))
        .expect("verify failed");
    assert_eq!(subclaim.evaluation_point, evaluation_point);
    assert_eq!(
        poly.evaluate(&slice_ops::slice_cast_with(
            &evaluation_point,
            ToArkScalar::to_ark_scalar,
        )),
        subclaim.expected_evaluation.to_ark_scalar()
    );

    // we return a different evaluation point if we start with a different transcript
    let mut transcript = Transcript::new(b"sumchecktest");
    transcript.append_auto(MessageLabel::SumcheckChallenge, &123u64);
    let subclaim = proof
        .verify_without_evaluation(&mut transcript, poly.info(), &ArkScalar::from(579u64))
        .expect("verify failed");
    assert_ne!(subclaim.evaluation_point, evaluation_point);

    // verify fails if sum is wrong
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim =
        proof.verify_without_evaluation(&mut transcript, poly.info(), &ArkScalar::from(123u64));
    assert!(subclaim.is_err());

    // verify fails if evaluations are changed
    proof.evaluations[0][1] += ArkScalar::from(3u64);
    let subclaim =
        proof.verify_without_evaluation(&mut transcript, poly.info(), &ArkScalar::from(579u64));
    assert!(subclaim.is_err());
}

fn random_product(
    nv: usize,
    num_multiplicands: usize,
    rng: &mut ThreadRng,
) -> (Vec<Rc<DenseMultilinearExtension>>, ArkScalar) {
    let mut multiplicands = Vec::with_capacity(num_multiplicands);
    for _ in 0..num_multiplicands {
        multiplicands.push(Vec::with_capacity(1 << nv))
    }
    let mut sum = ArkScalar::zero();

    for _ in 0..(1 << nv) {
        let mut product = ArkScalar::one();
        for multiplicand in multiplicands.iter_mut().take(num_multiplicands) {
            let val = ArkScalar::random(rng).to_ark_scalar();
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
    rng: &mut ThreadRng,
) -> (CompositePolynomial, ArkScalar) {
    let mut sum = ArkScalar::zero();
    let mut poly = CompositePolynomial::new(nv);
    for _ in 0..num_products {
        let num_multiplicands = rng.gen_range(num_multiplicands_range.0, num_multiplicands_range.1);
        let (product, product_sum) = random_product(nv, num_multiplicands, rng);
        let coefficient = ArkScalar::random(rng);
        poly.add_product(product.into_iter(), coefficient.to_ark_scalar());
        sum += product_sum.into_scalar() * coefficient;
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
    let mut evaluation_point = vec![ArkScalar::zero(); poly_info.num_variables];
    let proof = SumcheckProof::create(&mut transcript, &mut evaluation_point, &poly);

    // verify proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim = proof
        .verify_without_evaluation(&mut transcript, poly_info, &asserted_sum)
        .expect("verify failed");
    assert_eq!(subclaim.evaluation_point, evaluation_point);
    assert_eq!(
        poly.evaluate(&slice_ops::slice_cast_with(
            &evaluation_point,
            ToArkScalar::to_ark_scalar,
        )),
        subclaim.expected_evaluation.to_ark_scalar()
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
