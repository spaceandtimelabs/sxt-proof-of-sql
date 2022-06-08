use crate::pip::sumcheck::proof::*;

use ark_std::rc::Rc;
use curve25519_dalek::scalar::Scalar;

use crate::base::polynomial::CompositePolynomial;
use crate::base::polynomial::DenseMultilinearExtension;
use crate::base::proof::Transcript;

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
    let mut proof = SumcheckProof::create(&mut evaluation_point, &mut transcript, &poly);

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
    transcript.multiplication_domain_sep(123u64);
    let subclaim = proof
        .verify_without_evaluation(&mut transcript, poly.info(), &Scalar::from(579u64))
        .expect("verify failed");
    assert_ne!(subclaim.evaluation_point, evaluation_point);

    // verify fails if sum is wrong
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim =
        proof.verify_without_evaluation(&mut transcript, poly.info(), &Scalar::from(123u64));
    assert!(!subclaim.is_ok());

    // verify fails if evaluations are changed
    proof.evaluations[0][1] += Scalar::from(3u64);
    let subclaim =
        proof.verify_without_evaluation(&mut transcript, poly.info(), &Scalar::from(579u64));
    assert!(!subclaim.is_ok());
}
