use crate::base::commitment::commitment_evaluation_proof_test::{
    test_simple_commitment_evaluation_proof,
    test_commitment_evaluation_proof_with_length_1,
    test_random_commitment_evaluation_proof
};
use blitzar::proof::InnerProductProof;

#[test]
#[cfg(feature = "blitzar")]
fn test_simple_ipa() {
    test_simple_commitment_evaluation_proof::<InnerProductProof>(&(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_1() {
    test_commitment_evaluation_proof_with_length_1::<InnerProductProof>(&(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_128() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(128, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(128, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(128, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(128, 64, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(128, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_100() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(100, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(100, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(100, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(100, 64, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(100, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_64() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(64, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(64, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(64, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(64, 32, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(64, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_50() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(50, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(50, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(50, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(50, 32, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(50, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_32() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(32, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(32, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(32, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(32, 16, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(32, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_20() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(20, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(20, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(20, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(20, 16, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(20, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_16() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(16, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(16, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(16, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(16, 8, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(16, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_10() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(10, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(10, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(10, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(10, 8, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(10, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_8() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(8, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(8, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(8, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(8, 4, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(8, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_5() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(5, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(5, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(5, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(5, 4, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(5, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_4() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(4, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(4, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(4, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(4, 2, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(4, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_3() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(3, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(3, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(3, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(3, 2, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(3, 200, &(), &());
}

#[test]
#[cfg(feature = "blitzar")]
fn test_random_ipa_with_length_2() {
    test_random_commitment_evaluation_proof::<InnerProductProof>(2, 0, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(2, 1, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(2, 10, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(2, 2, &(), &());
    test_random_commitment_evaluation_proof::<InnerProductProof>(2, 200, &(), &());
}