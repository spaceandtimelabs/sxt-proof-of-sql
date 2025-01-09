/*
 * Adopted from arkworks
 *
 * See third_party/license/arkworks.LICENSE
 */
use super::test_cases::sumcheck_test_cases;
use crate::{
    base::{
        polynomial::CompositePolynomial,
        proof::Transcript as _,
        scalar::{
            test_scalar::{TestBN254Scalar, TestScalar},
            BN254Scalar, Curve25519Scalar, MontScalar, Scalar,
        },
    },
    proof_primitive::sumcheck::{ProverState, SumcheckProof},
};
use alloc::rc::Rc;
use merlin::Transcript;

fn test_create_verify_proof<S: Scalar + From<u64>>() {
    let num_vars = 1;
    let mut evaluation_point: [S; 1] = [S::zero(); 1];

    // create a proof
    let mut poly = CompositePolynomial::new(num_vars);
    let a_vec: [S; 2] = [S::from(123u64), S::from(456u64)];
    let fa = Rc::new(a_vec.to_vec());
    poly.add_product([fa], S::from(1u64));
    let mut transcript = Transcript::new(b"sumchecktest");
    let mut proof = SumcheckProof::create(
        &mut transcript,
        &mut evaluation_point,
        ProverState::create(&poly),
    );

    // verify proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim = proof
        .verify_without_evaluation(&mut transcript, poly.num_variables, &S::from(579u64))
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
        .verify_without_evaluation(&mut transcript, poly.num_variables, &S::from(579u64))
        .expect("verify failed");
    assert_ne!(subclaim.evaluation_point, evaluation_point);

    // verify fails if sum is wrong
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim =
        proof.verify_without_evaluation(&mut transcript, poly.num_variables, &S::from(123u64));
    assert!(subclaim.is_err());

    // verify fails if evaluations are changed
    proof.coefficients[0] += S::from(3u64);
    let subclaim =
        proof.verify_without_evaluation(&mut transcript, poly.num_variables, &S::from(579u64));
    assert!(subclaim.is_err());
}

#[test]
fn test_create_verify_proof_with_curve25519_scalar() {
    test_create_verify_proof::<Curve25519Scalar>();
}

#[test]
fn test_create_verify_proof_with_bn254_scalar() {
    test_create_verify_proof::<BN254Scalar>();
}

fn random_product<S: Scalar>(
    nv: usize,
    num_multiplicands: usize,
    rng: &mut ark_std::rand::rngs::StdRng,
) -> (Vec<Rc<Vec<S>>>, S) {
    let mut multiplicands = Vec::with_capacity(num_multiplicands);
    for _ in 0..num_multiplicands {
        multiplicands.push(Vec::with_capacity(1 << nv));
    }
    let mut sum = S::zero();

    for _ in 0..(1 << nv) {
        let mut product = S::one();
        for multiplicand in multiplicands.iter_mut().take(num_multiplicands) {
            let val = S::rand(rng);
            multiplicand.push(val);
            product *= val;
        }
        sum += product;
    }

    (multiplicands.into_iter().map(Rc::new).collect(), sum)
}

fn random_polynomial<S: Scalar>(
    nv: usize,
    num_multiplicands_range: (usize, usize),
    num_products: usize,
    rng: &mut ark_std::rand::rngs::StdRng,
) -> (CompositePolynomial<S>, S) {
    use ark_std::rand::Rng;
    let mut sum = S::zero();
    let mut poly = CompositePolynomial::new(nv);
    for _ in 0..num_products {
        let num_multiplicands = rng.gen_range(num_multiplicands_range.0..num_multiplicands_range.1);
        let (product, product_sum) = random_product(nv, num_multiplicands, rng);
        let coefficient = S::rand(rng);
        poly.add_product(product.into_iter(), coefficient);
        sum += product_sum * coefficient;
    }

    (poly, sum)
}

fn test_polynomial<S: Scalar>(
    nv: usize,
    num_multiplicands_range: (usize, usize),
    num_products: usize,
) {
    let mut rng = <ark_std::rand::rngs::StdRng as ark_std::rand::SeedableRng>::from_seed([0u8; 32]);
    let (poly, asserted_sum) =
        random_polynomial(nv, num_multiplicands_range, num_products, &mut rng);

    // create a proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let mut evaluation_point = vec![S::zero(); poly.num_variables];
    let proof = SumcheckProof::create(
        &mut transcript,
        &mut evaluation_point,
        ProverState::create(&poly),
    );

    // verify proof
    let mut transcript = Transcript::new(b"sumchecktest");
    let subclaim = proof
        .verify_without_evaluation(&mut transcript, poly.num_variables, &asserted_sum)
        .expect("verify failed");
    assert_eq!(subclaim.evaluation_point, evaluation_point);
    assert_eq!(
        poly.evaluate(&evaluation_point),
        subclaim.expected_evaluation
    );
}

#[test]
fn test_trivial_polynomial_with_curve25519_scalar() {
    let nv = 1;
    let num_multiplicands_range = (4, 13);
    let num_products = 5;

    test_polynomial::<Curve25519Scalar>(nv, num_multiplicands_range, num_products);
}

#[test]
fn test_trivial_polynomial_with_bn254_scalar() {
    let nv = 1;
    let num_multiplicands_range = (4, 13);
    let num_products = 5;

    test_polynomial::<BN254Scalar>(nv, num_multiplicands_range, num_products);
}

#[test]
fn test_normal_polynomial_with_curve25519_scalar() {
    let nv = 7;
    let num_multiplicands_range = (4, 9);
    let num_products = 5;

    test_polynomial::<Curve25519Scalar>(nv, num_multiplicands_range, num_products);
}

#[test]
fn test_normal_polynomial_with_bn254_scalar() {
    let nv = 7;
    let num_multiplicands_range = (4, 9);
    let num_products = 5;

    test_polynomial::<BN254Scalar>(nv, num_multiplicands_range, num_products);
}

#[test]
fn we_can_verify_many_random_cureve25519_test_cases() {
    let mut rng = ark_std::test_rng();

    for test_case in sumcheck_test_cases::<TestScalar>(&mut rng) {
        let mut transcript = Transcript::new(b"sumchecktest");
        let mut evaluation_point = vec![MontScalar::default(); test_case.num_vars];
        let proof = SumcheckProof::create(
            &mut transcript,
            &mut evaluation_point,
            ProverState::create(&test_case.polynomial),
        );

        let mut transcript = Transcript::new(b"sumchecktest");
        let subclaim = proof
            .verify_without_evaluation(&mut transcript, test_case.num_vars, &test_case.sum)
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
        let verify_result =
            proof.verify_without_evaluation(&mut transcript, test_case.num_vars, &test_case.sum);
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
                    test_case.num_vars,
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
                .verify_without_evaluation(&mut transcript, test_case.num_vars, &test_case.sum,)
                .is_err(),
            "verification should fail when the proof is modified"
        );
    }
}

#[test]
fn we_can_verify_many_random_bn254_test_cases() {
    let mut rng = ark_std::test_rng();

    for test_case in sumcheck_test_cases::<TestBN254Scalar>(&mut rng) {
        let mut transcript = Transcript::new(b"sumchecktest");
        let mut evaluation_point = vec![MontScalar::default(); test_case.num_vars];
        let proof = SumcheckProof::create(
            &mut transcript,
            &mut evaluation_point,
            ProverState::create(&test_case.polynomial),
        );

        let mut transcript = Transcript::new(b"sumchecktest");
        let subclaim = proof
            .verify_without_evaluation(&mut transcript, test_case.num_vars, &test_case.sum)
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
        let verify_result =
            proof.verify_without_evaluation(&mut transcript, test_case.num_vars, &test_case.sum);
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
                    test_case.num_vars,
                    &(test_case.sum + TestBN254Scalar::ONE),
                )
                .is_err(),
            "verification should fail when the sum is wrong"
        );

        let mut modified_proof = proof;
        modified_proof.coefficients[0] += TestBN254Scalar::ONE;
        let mut transcript = Transcript::new(b"sumchecktest");
        assert!(
            modified_proof
                .verify_without_evaluation(&mut transcript, test_case.num_vars, &test_case.sum,)
                .is_err(),
            "verification should fail when the proof is modified"
        );
    }
}
