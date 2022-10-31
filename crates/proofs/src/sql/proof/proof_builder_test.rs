use super::{
    make_sumcheck_term, DenseIntermediateResultColumn, ProofBuilder, ProofCounts,
    SumcheckSubpolynomial,
};

use crate::base::polynomial::CompositePolynomial;
use arrow::array::Int64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use byte_slice_cast::AsByteSlice;
use curve25519_dalek::traits::Identity;
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};
use pedersen::compute::compute_commitments;
use pedersen::sequences::{DenseSequence, Sequence};
use std::sync::Arc;

fn compute_commitment(data: &[i64]) -> CompressedRistretto {
    let descriptor = Sequence::Dense(DenseSequence {
        data_slice: data.as_byte_slice(),
        element_size: 8,
    });
    let mut res = CompressedRistretto::identity();
    compute_commitments(
        std::slice::from_mut(&mut res),
        std::slice::from_ref(&descriptor),
    );
    res
}

#[test]
fn we_can_compute_commitments_for_intermediate_mles() {
    let counts = ProofCounts {
        sumcheck_variables: 1,
        anchored_mles: 1,
        intermediate_mles: 1,
        ..Default::default()
    };
    let mle1 = [1, 2];
    let mle2 = [10, 20];
    let mut builder = ProofBuilder::new(&counts);
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let commitments = builder.commit_intermediate_mles();
    assert_eq!(commitments, [compute_commitment(&mle2)]);
}

#[test]
fn we_can_evaluate_pre_result_mles() {
    let counts = ProofCounts {
        sumcheck_variables: 1,
        anchored_mles: 1,
        intermediate_mles: 1,
        ..Default::default()
    };
    let mle1 = [1, 2];
    let mle2 = [10, 20];
    let mut builder = ProofBuilder::new(&counts);
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let evaluation_vec = [Scalar::from(100u64), Scalar::from(10u64)];
    let evals = builder.evaluate_pre_result_mles(&evaluation_vec);
    let expected_evals = [Scalar::from(120u64), Scalar::from(1200u64)];
    assert_eq!(evals, expected_evals);
}

#[test]
fn we_can_form_an_aggregated_sumcheck_polynomial() {
    let counts = ProofCounts {
        sumcheck_variables: 2,
        sumcheck_subpolynomials: 2,
        anchored_mles: 1,
        intermediate_mles: 1,
        ..Default::default()
    };
    let mle1 = [1, 2, -1];
    let mle2 = [10, 20, 100, 30];
    let mut builder = ProofBuilder::new(&counts);
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);

    let poly = SumcheckSubpolynomial::new(vec![(
        -Scalar::from(1u64),
        vec![make_sumcheck_term(2, &mle1)],
    )]);
    builder.produce_sumcheck_subpolynomial(poly);

    let poly = SumcheckSubpolynomial::new(vec![(
        -Scalar::from(10u64),
        vec![make_sumcheck_term(2, &mle2)],
    )]);
    builder.produce_sumcheck_subpolynomial(poly);

    let multipliers = [
        Scalar::from(5u64),
        Scalar::from(2u64),
        Scalar::from(20u64),
        Scalar::from(100u64),
        Scalar::from(50u64),
        Scalar::from(25u64),
    ];
    let poly = builder.make_sumcheck_polynomial(&multipliers);
    let mut expected_poly = CompositePolynomial::new(2);
    let fr = make_sumcheck_term(2, &multipliers[..4]);
    expected_poly.add_product(
        [fr.clone(), make_sumcheck_term(2, &mle1)],
        -Scalar::from(1u64) * multipliers[4],
    );
    expected_poly.add_product(
        [fr, make_sumcheck_term(2, &mle2)],
        -Scalar::from(10u64) * multipliers[5],
    );
    let random_point = [Scalar::from(123u64), Scalar::from(101112u64)];
    let eval = poly.evaluate(&random_point);
    let expected_eval = expected_poly.evaluate(&random_point);
    assert_eq!(eval, expected_eval);
}

#[test]
fn we_can_form_the_intermediate_query_result() {
    let counts = ProofCounts {
        sumcheck_variables: 2,
        result_columns: 2,
        ..Default::default()
    };
    let result_indexes = [1, 2];
    let col1 = [10, 11, 12];
    let col2 = [-2, -3, -4];
    let mut builder = ProofBuilder::new(&counts);
    builder.set_result_indexes(&result_indexes);
    builder.produce_result_column(Box::new(DenseIntermediateResultColumn::<i64>::new(&col1)));
    builder.produce_result_column(Box::new(DenseIntermediateResultColumn::<i64>::new(&col2)));
    let schema = Schema::new(vec![
        Field::new("1", DataType::Int64, false),
        Field::new("2", DataType::Int64, false),
    ]);
    let schema = Arc::new(schema);
    let res = builder.make_intermediate_query_result();
    let res = res.into_query_result(schema.clone()).unwrap();
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
fn we_can_fold_pre_result_mles() {
    let counts = ProofCounts {
        sumcheck_variables: 1,
        anchored_mles: 1,
        intermediate_mles: 1,
        ..Default::default()
    };
    let mle1 = [1, 2];
    let mle2 = [10, 20];
    let mut builder = ProofBuilder::new(&counts);
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let multipliers = [Scalar::from(100u64), Scalar::from(2u64)];
    let z = builder.fold_pre_result_mles(&multipliers);
    let expected_z = [Scalar::from(120u64), Scalar::from(240u64)];
    assert_eq!(z, expected_z);
}
