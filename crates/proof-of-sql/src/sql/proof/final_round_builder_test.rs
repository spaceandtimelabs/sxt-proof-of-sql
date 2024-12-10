use super::{FinalRoundBuilder, ProvableQueryResult};
use crate::base::{
    commitment::{Commitment, CommittableColumn},
    database::{Column, ColumnField, ColumnType},
    scalar::Curve25519Scalar,
};
use alloc::sync::Arc;
#[cfg(feature = "arrow")]
use arrow::{
    array::Int64Array,
    datatypes::{Field, Schema},
    record_batch::RecordBatch,
};
use curve25519_dalek::RistrettoPoint;

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::<Curve25519Scalar>::new(1, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let offset_generators = 0_usize;
    let commitments: Vec<RistrettoPoint> = builder.commit_intermediate_mles(offset_generators, &());
    assert_eq!(
        commitments,
        [RistrettoPoint::compute_commitments(
            &[CommittableColumn::from(&mle2[..])],
            offset_generators,
            &()
        )[0]]
    );
}

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_non_zero_offset() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::<Curve25519Scalar>::new(1, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let offset_generators = 123_usize;
    let commitments: Vec<RistrettoPoint> = builder.commit_intermediate_mles(offset_generators, &());
    assert_eq!(
        commitments,
        [RistrettoPoint::compute_commitments(
            &[CommittableColumn::from(&mle2[..])],
            offset_generators,
            &()
        )[0]]
    );
}

#[test]
fn we_can_evaluate_pcs_proof_mles() {
    let mle1 = [1, 2];
    let mle2 = [10i64, 20];
    let mut builder = FinalRoundBuilder::new(1, Vec::new());
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2[..]);
    let evaluation_vec = [
        Curve25519Scalar::from(100u64),
        Curve25519Scalar::from(10u64),
    ];
    let evals = builder.evaluate_pcs_proof_mles(&evaluation_vec);
    let expected_evals = [
        Curve25519Scalar::from(120u64),
        Curve25519Scalar::from(1200u64),
    ];
    assert_eq!(evals, expected_evals);
}

#[cfg(feature = "arrow")]
#[test]
fn we_can_form_the_provable_query_result() {
    let col1: Column<Curve25519Scalar> = Column::BigInt(&[11_i64, 12]);
    let col2: Column<Curve25519Scalar> = Column::BigInt(&[-3_i64, -4]);
    let res = ProvableQueryResult::new(2, &[col1, col2]);

    let column_fields = vec![
        ColumnField::new("a".parse().unwrap(), ColumnType::BigInt),
        ColumnField::new("b".parse().unwrap(), ColumnType::BigInt),
    ];
    let res = RecordBatch::try_from(
        res.to_owned_table::<Curve25519Scalar>(&column_fields)
            .unwrap(),
    )
    .unwrap();
    let column_fields: Vec<Field> = column_fields
        .iter()
        .map(core::convert::Into::into)
        .collect();
    let schema = Arc::new(Schema::new(column_fields));

    let expected_res = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![11, 12])),
            Arc::new(Int64Array::from(vec![-3, -4])),
        ],
    )
    .unwrap();
    assert_eq!(res, expected_res);
}

#[test]
fn we_can_consume_post_result_challenges_in_proof_builder() {
    let mut builder = FinalRoundBuilder::new(
        0,
        vec![
            Curve25519Scalar::from(123),
            Curve25519Scalar::from(456),
            Curve25519Scalar::from(789),
        ],
    );
    assert_eq!(
        Curve25519Scalar::from(789),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        Curve25519Scalar::from(456),
        builder.consume_post_result_challenge()
    );
    assert_eq!(
        Curve25519Scalar::from(123),
        builder.consume_post_result_challenge()
    );
}
