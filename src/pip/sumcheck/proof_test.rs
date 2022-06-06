use crate::pip::sumcheck::proof::*;

use ark_std::rc::Rc;
use curve25519_dalek::scalar::Scalar;
use merlin::Transcript;

use crate::base::polynomial::CompositePolynomial;
use crate::base::polynomial::DenseMultilinearExtension;

#[test]
fn test_create_verify_proof() {
    let num_vars = 1;

    // create a proof
    let mut poly = CompositePolynomial::new(num_vars);
    let a_vec: [Scalar; 2] = [Scalar::from(123u64), Scalar::from(456u64)];
    let fa = Rc::new(DenseMultilinearExtension::from_evaluations_slice(
        num_vars, &a_vec,
    ));
    poly.add_product([fa], Scalar::from(1u64));
    let mut transcript = Transcript::new(b"sumchecktest");
    let proof = SumcheckProof::create(&mut transcript, &poly);

    // verify proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let mut evaluation_point: [Scalar; 2] = [Scalar::from(0u64); 2];
    assert!(proof
        .verify_without_evaluation(&mut evaluation_point, &mut transcript, poly.info())
        .is_ok());
}
