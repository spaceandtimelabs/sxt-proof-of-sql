use super::test_cases::sumcheck_test_cases;
use crate::base::{
    polynomial::{CompositePolynomial, CompositePolynomialInfo},
    proof::Transcript as _,
    scalar::{test_scalar::TestScalar, Curve25519Scalar, Scalar},
};
/*
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use crate::proof_primitive::sumcheck::proof::*;
use alloc::rc::Rc;
use ark_std::UniformRand;
use merlin::Transcript;
use num_traits::{One, Zero};

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
    transcript.extend_serialize_as_le(&123u64);
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
    proof.coefficients[0] += Curve25519Scalar::from(3u64);
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
        multiplicands.push(Vec::with_capacity(1 << nv));
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

#[test]
fn we_can_verify_many_random_test_cases() {
    let mut rng = ark_std::test_rng();

    for test_case in sumcheck_test_cases::<TestScalar>(&mut rng) {
        let mut transcript = Transcript::new(b"sumchecktest");
        let mut evaluation_point = vec![Default::default(); test_case.num_vars];
        let proof = SumcheckProof::create(
            &mut transcript,
            &mut evaluation_point,
            &test_case.polynomial,
        );

        let mut transcript = Transcript::new(b"sumchecktest");
        let subclaim = proof
            .verify_without_evaluation(
                &mut transcript,
                CompositePolynomialInfo {
                    max_multiplicands: test_case.max_multiplicands,
                    num_variables: test_case.num_vars,
                },
                &test_case.sum,
            )
            .expect("verification should succeed with the correct setup");
        assert_eq!(
            subclaim.evaluation_point, evaluation_point,
            "the prover's evaluation point should match the verifier's"
        );
        assert_eq!(
            test_case.polynomial.evaluate(&evaluation_point),
            subclaim.expected_evaluation,
            "the claimed evaluation should match the actual evaluation"
        );

        let mut transcript = Transcript::new(b"sumchecktest");
        transcript.extend_serialize_as_le(&123u64);
        let verify_result = proof.verify_without_evaluation(
            &mut transcript,
            CompositePolynomialInfo {
                max_multiplicands: test_case.max_multiplicands,
                num_variables: test_case.num_vars,
            },
            &test_case.sum,
        );
        if let Ok(subclaim) = verify_result {
            assert_ne!(
                subclaim.evaluation_point, evaluation_point,
                "either verification should fail or we should have a different evaluation point with a different transcript"
            );
        }

        let mut transcript = Transcript::new(b"sumchecktest");
        assert!(
            proof
                .verify_without_evaluation(
                    &mut transcript,
                    CompositePolynomialInfo {
                        max_multiplicands: test_case.max_multiplicands,
                        num_variables: test_case.num_vars,
                    },
                    &(test_case.sum + TestScalar::ONE),
                )
                .is_err(),
            "verification should fail when the sum is wrong"
        );

        let mut modified_proof = proof;
        modified_proof.coefficients[0] += TestScalar::ONE;
        let mut transcript = Transcript::new(b"sumchecktest");
        assert!(
            modified_proof
                .verify_without_evaluation(
                    &mut transcript,
                    CompositePolynomialInfo {
                        max_multiplicands: test_case.max_multiplicands,
                        num_variables: test_case.num_vars,
                    },
                    &test_case.sum,
                )
                .is_err(),
            "verification should fail when the proof is modified"
        );
    }
}
