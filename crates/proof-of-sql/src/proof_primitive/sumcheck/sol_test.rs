use super::{
    sol_types::Sumcheck::{Proof as SolProof, Subclaim as SolSubclaim},
    test_cases::sumcheck_test_cases,
    SumcheckProof,
};
use crate::{
    base::{
        polynomial::CompositePolynomialInfo,
        proof::{Keccak256Transcript, Transcript},
        scalar::test_scalar::{TestMontConfig, TestScalar},
    },
    tests::ForgeScript,
};
use alloy_sol_types::private::primitives::{B256, U256};

#[test]
#[ignore = "Because forge needs to be installed, we are ignoring this test by default. They will still be run from within the ci."]
fn we_can_correctly_verify_many_random_test_cases_with_solidity() {
    let mut rng = ark_std::test_rng();

    for test_case in sumcheck_test_cases::<TestScalar>(&mut rng) {
        // Generate proof
        let mut transcript = Keccak256Transcript::new();
        transcript.extend_as_le_from_refs([b"sumchecktest"]);
        let _transcript_start_hash = transcript.challenge_as_le();
        let mut evaluation_point = vec![Default::default(); test_case.num_vars];
        let proof = SumcheckProof::create(
            &mut transcript,
            &mut evaluation_point,
            &test_case.polynomial,
        );
        let _transcript_end_hash = transcript.challenge_as_le();

        // Verify with Rust verifier
        let mut transcript = Keccak256Transcript::new();
        transcript.extend_as_le_from_refs([b"sumchecktest"]);
        let transcript_start_hash = transcript.challenge_as_le();
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
        let transcript_end_hash = transcript.challenge_as_le();

        // Verify with Solidity verifier
        ForgeScript::new(
            "./sol_src/proof_primitive/sumcheck/Sumcheck.t.sol",
            "rustTestVerification",
        )
        .arg(SolProof::from(proof.clone()))
        .arg(B256::from(transcript_start_hash))
        .arg(u16::try_from(test_case.max_multiplicands).unwrap())
        .arg(u16::try_from(test_case.num_vars).unwrap())
        .arg(U256::from_limbs(test_case.sum.into()))
        .arg(U256::from_limbs(
            <TestMontConfig as ark_ff::MontConfig<4>>::MODULUS.0,
        ))
        .arg(SolSubclaim::from(subclaim))
        .arg(B256::from(transcript_end_hash))
        .execute()
        .unwrap();
    }
}
