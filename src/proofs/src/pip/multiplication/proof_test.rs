use crate::{
    base::{
        proof::{Commit, GeneralColumn, PipProve, PipVerify, Transcript},
        scalar::{SafeInt, SafeIntColumn},
    },
    pip::{multiplication::MultiplicationProof, range::LogMaxReductionProof},
};

use curve25519_dalek::scalar::Scalar;

#[test]
fn test_multiplication() {
    let a: SafeIntColumn = vec![1, 2, 3, 5, 0, 1]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let b: SafeIntColumn = vec![4, 3, 4, 0, 2, 3]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let product = SafeIntColumn::try_new(
        vec![4u32, 6u32, 12u32, 0u32, 0u32, 3u32]
            .into_iter()
            .map(Scalar::from)
            .collect(),
        5,
    )
    .unwrap();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_product = product.commit();

    let mut transcript = Transcript::new(b"multiplicationtest");
    let proof = MultiplicationProof::prove(&mut transcript, (a, b), product, (c_a, c_b));

    let mut transcript = Transcript::new(b"multiplicationtest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    // correct output commitment
    assert_eq!(proof.get_output_commitments(), c_product);

    // wrong input commitments
    let mut transcript = Transcript::new(b"multiplicationtest");
    assert!(proof.verify(&mut transcript, (c_a, c_a)).is_err());
}

#[test]
fn test_multiplication_wrong() {
    let a: SafeIntColumn = vec![1, 2, 3, 5, 0, 1]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let b: SafeIntColumn = vec![1, 1, 1, 1, 1, 1]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let product = SafeIntColumn::try_new(
        vec![1u32, 2u32, 3u32, 4u32, 0u32, 1u32]
            .into_iter()
            .map(Scalar::from)
            .collect(),
        3,
    )
    .unwrap();

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"multiplicationtest");
    let proof = MultiplicationProof::prove(&mut transcript, (a, b), product, (c_a, c_b));

    // The wrong output for multiplication will result in a VerificationError instead of having an
    // output commitment that doesn't match the actual output.
    let mut transcript = Transcript::new(b"multiplicationtest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_err());
}

#[test]
fn test_multiplication_log_max_reduction() {
    let a = SafeIntColumn::try_new(
        vec![
            Scalar::from(100u32),
            -Scalar::from(100u32),
            Scalar::from(0u32),
            Scalar::from(10u32),
            -Scalar::from(10u32),
            Scalar::from(0u32),
        ],
        125,
    )
    .unwrap();
    let b: SafeIntColumn = vec![1, 2, 4, 8, 16, 32]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let product = SafeIntColumn::try_new(
        vec![
            Scalar::from(100u32),
            -Scalar::from(200u32),
            Scalar::from(0u32),
            Scalar::from(80u32),
            -Scalar::from(160u32),
            Scalar::from(0u32),
        ],
        128,
    )
    .unwrap();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_product = product.commit();

    let mut transcript = Transcript::new(b"multiplicationtest");
    let proof = MultiplicationProof::prove(&mut transcript, (a, b), product, (c_a, c_b));

    assert!(proof.log_max_reduction_proof.is_some());

    let mut transcript = Transcript::new(b"multiplicationtest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    assert_eq!(proof.get_output_commitments(), c_product);
    assert_eq!(proof.get_output_commitments().log_max, c_product.log_max);
}

#[test]
fn test_multiplication_log_max_reduction_missing() {
    let a = SafeIntColumn::try_new(
        vec![
            Scalar::from(100u32),
            -Scalar::from(100u32),
            Scalar::from(0u32),
            Scalar::from(10u32),
            -Scalar::from(10u32),
            Scalar::from(0u32),
        ],
        125,
    )
    .unwrap();
    let b: SafeIntColumn = vec![1, 2, 4, 8, 16, 32]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let product = SafeIntColumn::try_new(
        vec![
            Scalar::from(100u32),
            -Scalar::from(200u32),
            Scalar::from(0u32),
            Scalar::from(80u32),
            -Scalar::from(160u32),
            Scalar::from(0u32),
        ],
        128,
    )
    .unwrap();

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"multiplicationtest");
    let mut proof = MultiplicationProof::prove(&mut transcript, (a, b), product, (c_a, c_b));

    proof.log_max_reduction_proof = None;

    let mut transcript = Transcript::new(b"multiplicationtest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_err());
}

#[test]
fn test_multiplication_log_max_reduction_superfluous() {
    let a = SafeIntColumn::try_new(
        vec![
            Scalar::from(100u32),
            -Scalar::from(100u32),
            Scalar::from(0u32),
            Scalar::from(10u32),
            -Scalar::from(10u32),
            Scalar::from(0u32),
        ],
        10,
    )
    .unwrap();
    let b: SafeIntColumn = vec![1, 2, 4, 8, 16, 32]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let product = SafeIntColumn::try_new(
        vec![
            Scalar::from(100u32),
            -Scalar::from(200u32),
            Scalar::from(0u32),
            Scalar::from(80u32),
            -Scalar::from(160u32),
            Scalar::from(0u32),
        ],
        15,
    )
    .unwrap();

    let product_overestimated = SafeIntColumn::try_new(
        product.clone().into_iter().map(|s| s.value()).collect(),
        128,
    )
    .unwrap();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_product = product.commit();
    let c_product_overestimated = product_overestimated.commit();

    let mut transcript = Transcript::new(b"multiplicationtest");
    let mut proof = MultiplicationProof::prove(
        &mut transcript,
        (a, b),
        product_overestimated.clone(),
        (c_a, c_b),
    );

    proof.log_max_reduction_proof = Some(LogMaxReductionProof::<128>::prove(
        &mut transcript,
        (product,),
        product_overestimated,
        (c_product,),
    ));
    proof.c_product.log_max = Some(128);

    let mut transcript = Transcript::new(b"multiplicationtest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    assert_eq!(proof.get_output_commitments(), c_product_overestimated);
    assert_eq!(
        proof.get_output_commitments().log_max,
        c_product_overestimated.log_max
    );
}

#[test]
fn test_multiplication_general() {
    let a = GeneralColumn::SafeIntColumn(
        vec![1, 2, 3, 5, 0, 1]
            .into_iter()
            .map(SafeInt::from)
            .collect(),
    );
    let b = GeneralColumn::SafeIntColumn(
        vec![4, 3, 4, 0, 2, 3]
            .into_iter()
            .map(SafeInt::from)
            .collect(),
    );
    let product = GeneralColumn::SafeIntColumn(
        SafeIntColumn::try_new(
            vec![
                Scalar::from(4u32),
                Scalar::from(6u32),
                Scalar::from(12u32),
                Scalar::from(0u32),
                Scalar::from(0u32),
                Scalar::from(3u32),
            ],
            5,
        )
        .unwrap(),
    );

    let c_a = a.commit();
    let c_b = b.commit();
    let c_product = product.commit();

    let mut transcript = Transcript::new(b"multiplicationtest");
    let proof = MultiplicationProof::prove(&mut transcript, (a, b), product, (c_a, c_b));

    let mut transcript = Transcript::new(b"multiplicationtest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    // correct output commitment
    assert_eq!(proof.get_output_commitments(), c_product);

    // wrong input commitments
    let mut transcript = Transcript::new(b"multiplicationtest");
    assert!(proof.verify(&mut transcript, (c_a, c_a)).is_err());
}

#[test]
#[should_panic]
fn test_multiplication_general_mismatched_inputs() {
    let a = GeneralColumn::SafeIntColumn(
        vec![1, 2, 3, 5, 0, 1]
            .into_iter()
            .map(SafeInt::from)
            .collect(),
    );
    let b = GeneralColumn::BooleanColumn(vec![true, true, false, false, true, true].into());
    let product = GeneralColumn::SafeIntColumn(
        SafeIntColumn::try_new(
            vec![
                Scalar::from(1u32),
                Scalar::from(2u32),
                Scalar::from(0u32),
                Scalar::from(0u32),
                Scalar::from(0u32),
                Scalar::from(1u32),
            ],
            3,
        )
        .unwrap(),
    );

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"multiplicationtest");
    let _should_panic = MultiplicationProof::prove(&mut transcript, (a, b), product, (c_a, c_b));
}

#[test]
#[should_panic]
fn test_multiplication_general_non_numeric() {
    let a = GeneralColumn::BooleanColumn(vec![true, true, false, false, true, true].into());
    let b = GeneralColumn::BooleanColumn(vec![true, false, true, false, true, false].into());
    let product = GeneralColumn::BooleanColumn(vec![true, false, false, false, true, false].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"multiplicationtest");
    let _should_panic = MultiplicationProof::prove(&mut transcript, (a, b), product, (c_a, c_b));
}
