use super::{
    DenseProvableResultColumn, MultilinearExtensionImpl, ProofBuilder, ProofCounts,
    SumcheckRandomScalars, SumcheckSubpolynomial,
};

use crate::base::database::{ColumnField, ColumnType};
use crate::base::polynomial::{ArkScalar, CompositePolynomial};
use crate::base::scalar::{compute_commitment_for_testing, ToArkScalar, Zero};
use crate::base::slice_ops;
use crate::sql::proof::{compute_evaluation_vector, MultilinearExtension};

use crate::base::polynomial::Scalar;
use arrow::array::Int64Array;
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use std::sync::Arc;

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_zero_offset() {
    let counts = ProofCounts {
        sumcheck_variables: 1,
        anchored_mles: 1,
        intermediate_mles: 1,
        ..Default::default()
    };
    let mle1 = [1, 2];
    let mle2 = [10u32, 20];
    let mut builder = ProofBuilder::new(&counts);
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let offset_generators = 0_usize;
    let commitments = builder.commit_intermediate_mles(offset_generators);
    assert_eq!(
        commitments,
        [compute_commitment_for_testing(&mle2, offset_generators).compress()]
    );
}

#[test]
fn we_can_compute_commitments_for_intermediate_mles_using_a_non_zero_offset() {
    let counts = ProofCounts {
        sumcheck_variables: 1,
        anchored_mles: 1,
        intermediate_mles: 1,
        ..Default::default()
    };
    let mle1 = [1, 2];
    let mle2 = [10u32, 20];
    let mut builder = ProofBuilder::new(&counts);
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let offset_generators = 123_usize;
    let commitments = builder.commit_intermediate_mles(offset_generators);
    assert_eq!(
        commitments,
        [compute_commitment_for_testing(&mle2, offset_generators).compress()]
    );
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
    let mle2 = [10u32, 20];
    let mut builder = ProofBuilder::new(&counts);
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let evaluation_vec = [Scalar::from(100u64), Scalar::from(10u64)];
    let evals = builder.evaluate_pre_result_mles(&slice_ops::slice_cast_with(
        &evaluation_vec,
        ToArkScalar::to_ark_scalar,
    ));
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
        table_length: 4,
        ..Default::default()
    };
    let mle1 = [1, 2, -1];
    let mle2 = [10u32, 20, 100, 30];
    let mut builder = ProofBuilder::new(&counts);
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);

    builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![(
        -Scalar::one(),
        vec![Box::new(MultilinearExtensionImpl::new(&mle1))],
    )]));
    builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![(
        -Scalar::from(10u64),
        vec![Box::new(MultilinearExtensionImpl::new(&mle2))],
    )]));

    let multipliers = [
        Scalar::from(5u64),
        Scalar::from(2u64),
        Scalar::from(50u64),
        Scalar::from(25u64),
    ];

    let mut evaluation_vector = vec![Zero::zero(); 4];
    compute_evaluation_vector(&mut evaluation_vector, &multipliers[..2]);

    let poly = builder.make_sumcheck_polynomial(&SumcheckRandomScalars::new(&counts, &multipliers));
    let mut expected_poly = CompositePolynomial::new(2);
    let fr = MultilinearExtensionImpl::new(&evaluation_vector).to_sumcheck_term(2);
    expected_poly.add_product(
        [
            fr.clone(),
            MultilinearExtensionImpl::new(&mle1).to_sumcheck_term(2),
        ],
        -ArkScalar::from(1u64) * multipliers[2].to_ark_scalar(),
    );
    expected_poly.add_product(
        [fr, MultilinearExtensionImpl::new(&mle2).to_sumcheck_term(2)],
        -ArkScalar::from(10u64) * multipliers[3].to_ark_scalar(),
    );
    let random_point = [ArkScalar::from(123u64), ArkScalar::from(101112u64)];
    let eval = poly.evaluate(&random_point);
    let expected_eval = expected_poly.evaluate(&random_point);
    assert_eq!(eval, expected_eval);
}

#[test]
fn we_can_form_the_provable_query_result() {
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
    builder.produce_result_column(Box::new(DenseProvableResultColumn::<i64>::new(&col1)));
    builder.produce_result_column(Box::new(DenseProvableResultColumn::<i64>::new(&col2)));

    let res = builder.make_provable_query_result();

    let column_fields =
        vec![ColumnField::new("a".parse().unwrap(), ColumnType::BigInt); counts.result_columns];
    let res = res.into_query_result(&column_fields).unwrap();
    let column_fields = column_fields.iter().map(|v| v.into()).collect();
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
fn we_can_fold_pre_result_mles() {
    let counts = ProofCounts {
        sumcheck_variables: 1,
        anchored_mles: 1,
        intermediate_mles: 1,
        table_length: 2,
        ..Default::default()
    };
    let mle1 = [1, 2];
    let mle2 = [10u32, 20];
    let mut builder = ProofBuilder::new(&counts);
    builder.produce_anchored_mle(&mle1);
    builder.produce_intermediate_mle(&mle2);
    let multipliers = [Scalar::from(100u64), Scalar::from(2u64)];
    let z = builder.fold_pre_result_mles(&multipliers);
    let expected_z = [Scalar::from(120u64), Scalar::from(240u64)];
    assert_eq!(z, expected_z);
}
