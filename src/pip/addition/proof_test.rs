use crate::{
    base::{
        proof::{Commit, GeneralColumn, PipProve, PipVerify, Transcript},
        scalar::{SafeInt, SafeIntColumn},
    },
    pip::{addition::AdditionProof, range::LogMaxReductionProof},
};

use curve25519_dalek::scalar::Scalar;

#[test]
fn test_addition() {
    let a: SafeIntColumn = vec![1, 2, 3, 5, 0, 1]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let b: SafeIntColumn = vec![4, 3, 4, 0, 2, 3]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let sum = SafeIntColumn::try_new(
        vec![5u32, 5u32, 7u32, 5u32, 2u32, 4u32]
            .into_iter()
            .map(Scalar::from)
            .collect(),
        4,
    )
    .unwrap();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_sum = sum.commit();

    let mut transcript = Transcript::new(b"additiontest");
    let proof = AdditionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));

    let mut transcript = Transcript::new(b"additiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    // correct output commitment
    assert_eq!(proof.get_output_commitments(), c_sum);

    // wrong input commitments
    let mut transcript = Transcript::new(b"additiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_a)).is_err());
}

#[test]
fn test_addition_wrong() {
    let a: SafeIntColumn = vec![1, 2, 3, 5, 0, 1]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let b: SafeIntColumn = vec![1, 1, 1, 1, 1, 1]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let sum = SafeIntColumn::try_new(
        vec![2u32, 3u32, 4u32, 5u32, 1u32, 2u32]
            .into_iter()
            .map(Scalar::from)
            .collect(),
        4,
    )
    .unwrap();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_sum = sum.commit();

    let mut transcript = Transcript::new(b"additiontest");
    let proof = AdditionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));

    assert_ne!(proof.get_output_commitments(), c_sum);
}

#[test]
fn test_addition_log_max_reduction() {
    let a = SafeIntColumn::try_new(
        vec![
            Scalar::from(100u32),
            -Scalar::from(100u32),
            Scalar::from(0u32),
            Scalar::from(10u32),
            -Scalar::from(10u32),
            Scalar::from(0u32),
        ],
        128,
    )
    .unwrap();
    let b: SafeIntColumn = vec![1, 2, 4, 8, 16, 32]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let sum = SafeIntColumn::try_new(
        vec![
            Scalar::from(101u32),
            -Scalar::from(98u32),
            Scalar::from(4u32),
            Scalar::from(18u32),
            Scalar::from(6u32),
            Scalar::from(32u32),
        ],
        128,
    )
    .unwrap();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_sum = sum.commit();

    let mut transcript = Transcript::new(b"additiontest");
    let proof = AdditionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));

    assert!(proof.log_max_reduction_proof.is_some());

    let mut transcript = Transcript::new(b"additiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    assert_eq!(proof.get_output_commitments(), c_sum);
    assert_eq!(proof.get_output_commitments().log_max, c_sum.log_max);
}

#[test]
fn test_addition_log_max_reduction_missing() {
    let a = SafeIntColumn::try_new(
        vec![
            Scalar::from(100u32),
            -Scalar::from(100u32),
            Scalar::from(0u32),
            Scalar::from(10u32),
            -Scalar::from(10u32),
            Scalar::from(0u32),
        ],
        128,
    )
    .unwrap();
    let b: SafeIntColumn = vec![1, 2, 4, 8, 16, 32]
        .into_iter()
        .map(SafeInt::from)
        .collect();
    let sum = SafeIntColumn::try_new(
        vec![
            Scalar::from(101u32),
            -Scalar::from(98u32),
            Scalar::from(4u32),
            Scalar::from(18u32),
            Scalar::from(6u32),
            Scalar::from(32u32),
        ],
        128,
    )
    .unwrap();

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"additiontest");
    let mut proof = AdditionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));

    proof.log_max_reduction_proof = None;

    let mut transcript = Transcript::new(b"additiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_err());
}

#[test]
fn test_addition_log_max_reduction_superfluous() {
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
    let sum = SafeIntColumn::try_new(
        vec![
            Scalar::from(101u32),
            -Scalar::from(98u32),
            Scalar::from(4u32),
            Scalar::from(18u32),
            Scalar::from(6u32),
            Scalar::from(32u32),
        ],
        11,
    )
    .unwrap();

    let sum_overestimated =
        SafeIntColumn::try_new(sum.clone().into_iter().map(|s| s.value()).collect(), 128).unwrap();

    let c_a = a.commit();
    let c_b = b.commit();
    let c_sum = sum.commit();
    let c_sum_overestimated = sum_overestimated.commit();

    let mut transcript = Transcript::new(b"additiontest");
    let mut proof = AdditionProof::prove(
        &mut transcript,
        (a, b),
        sum_overestimated.clone(),
        (c_a, c_b),
    );

    proof.log_max_reduction_proof = Some(LogMaxReductionProof::<128>::prove(
        &mut transcript,
        (sum,),
        sum_overestimated,
        (c_sum,),
    ));
    proof.c_sum.log_max = Some(128);

    let mut transcript = Transcript::new(b"additiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    assert_eq!(proof.get_output_commitments(), c_sum_overestimated);
    assert_eq!(
        proof.get_output_commitments().log_max,
        c_sum_overestimated.log_max
    );
}

#[test]
fn test_addition_general() {
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
    let sum = GeneralColumn::SafeIntColumn(
        SafeIntColumn::try_new(
            vec![
                Scalar::from(5u32),
                Scalar::from(5u32),
                Scalar::from(7u32),
                Scalar::from(5u32),
                Scalar::from(2u32),
                Scalar::from(4u32),
            ],
            4,
        )
        .unwrap(),
    );

    let c_a = a.commit();
    let c_b = b.commit();
    let c_sum = sum.commit();

    let mut transcript = Transcript::new(b"additiontest");
    let proof = AdditionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));

    let mut transcript = Transcript::new(b"additiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_b)).is_ok());

    // correct output commitment
    assert_eq!(proof.get_output_commitments(), c_sum);

    // wrong input commitments
    let mut transcript = Transcript::new(b"additiontest");
    assert!(proof.verify(&mut transcript, (c_a, c_a)).is_err());
}

#[test]
#[should_panic]
fn test_addition_general_mismatched_inputs() {
    let a = GeneralColumn::SafeIntColumn(
        vec![1, 2, 3, 5, 0, 1]
            .into_iter()
            .map(SafeInt::from)
            .collect(),
    );
    let b = GeneralColumn::BooleanColumn(vec![true, true, false, false, true, true].into());
    let sum = GeneralColumn::SafeIntColumn(
        SafeIntColumn::try_new(
            vec![
                Scalar::from(2u32),
                Scalar::from(3u32),
                Scalar::from(3u32),
                Scalar::from(5u32),
                Scalar::from(1u32),
                Scalar::from(2u32),
            ],
            4,
        )
        .unwrap(),
    );

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"additiontest");
    let _should_panic = AdditionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));
}

#[test]
#[should_panic]
fn test_addition_general_non_numeric() {
    let a = GeneralColumn::BooleanColumn(vec![true, true, false, false, true, true].into());
    let b = GeneralColumn::BooleanColumn(vec![true, false, true, false, true, false].into());
    let sum = GeneralColumn::BooleanColumn(vec![false, true, true, false, false, true].into());

    let c_a = a.commit();
    let c_b = b.commit();

    let mut transcript = Transcript::new(b"additiontest");
    let _should_panic = AdditionProof::prove(&mut transcript, (a, b), sum, (c_a, c_b));
}
