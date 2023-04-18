use super::{
    DenseProvableResultColumn, MultilinearExtensionImpl, ProofBuilder, ProofCounts, ProofExpr,
    QueryProof, SumcheckSubpolynomial, TestQueryExpr, VerificationBuilder,
};
use crate::base::database::{CommitmentAccessor, DataAccessor, TestAccessor};
use crate::base::math::log2_up;
use crate::base::scalar::compute_commitment_for_testing;
use arrow::array::Int64Array;
use arrow::datatypes::Schema;
use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};
use std::sync::Arc;

fn verify_a_trivial_query_proof_with_given_offset(n: usize, offset_generators: usize) {
    let num_sumcheck_variables = std::cmp::max(log2_up(n), 1);
    // prove and verify an artificial polynomial where we prove
    // that every entry in the result is zero
    let counts = ProofCounts {
        table_length: n,
        offset_generators,
        sumcheck_variables: num_sumcheck_variables,
        sumcheck_max_multiplicands: 2,
        result_columns: 1,
        sumcheck_subpolynomials: 1,
        ..Default::default()
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        let col = alloc.alloc_slice_fill_copy(counts.table_length, 0i64);
        let indexes = alloc.alloc_slice_fill_copy(1, 0u64);
        builder.set_result_indexes(indexes);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(col)));
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![(
            Scalar::one(),
            vec![Box::new(MultilinearExtensionImpl::new(col))],
        )]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        assert_eq!(builder.consume_result_mle(), Scalar::zero());
        builder.produce_sumcheck_subpolynomial_evaluation(&Scalar::zero());
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    let result = proof
        .verify(&expr, &accessor, &counts, &result)
        .unwrap()
        .unwrap();
    let column_fields = expr
        .get_column_result_fields()
        .iter()
        .map(|v| v.into())
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_result =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![0]))]).unwrap();
    assert_eq!(result, expected_result);
}

#[test]
fn we_can_verify_a_trivial_query_proof_with_a_zero_offset() {
    for n in 1..5 {
        verify_a_trivial_query_proof_with_given_offset(n, 0);
    }
}

#[test]
fn we_can_verify_a_trivial_query_proof_with_a_non_zero_offset() {
    for n in 1..5 {
        verify_a_trivial_query_proof_with_given_offset(n, 123);
    }
}

#[test]
fn verify_fails_if_the_summation_in_sumcheck_isnt_zero() {
    // set up a proof for an artificial polynomial that doesn't sum to zero
    let counts = ProofCounts {
        table_length: 2,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 2,
        result_columns: 1,
        sumcheck_subpolynomials: 1,
        ..Default::default()
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        let col = alloc.alloc_slice_fill_copy(2, 123i64);
        let indexes = alloc.alloc_slice_fill_copy(1, 0u64);
        builder.set_result_indexes(indexes);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(col)));
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![(
            Scalar::one(),
            vec![Box::new(MultilinearExtensionImpl::new(col))],
        )]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        assert_eq!(builder.consume_result_mle(), Scalar::zero());
        builder.produce_sumcheck_subpolynomial_evaluation(&Scalar::zero());
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    assert!(proof.verify(&expr, &accessor, &counts, &result).is_err());
}

#[test]
fn verify_fails_if_the_sumcheck_evaluation_isnt_correct() {
    // set up a proof for an artificial polynomial and specify an evaluation that won't
    // match the evaluation from sumcheck
    let counts = ProofCounts {
        table_length: 2,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 2,
        result_columns: 1,
        sumcheck_subpolynomials: 1,
        ..Default::default()
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        let col = alloc.alloc_slice_fill_copy(2, 0i64);
        let indexes = alloc.alloc_slice_fill_copy(1, 0u64);
        builder.set_result_indexes(indexes);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(col)));
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![(
            Scalar::one(),
            vec![Box::new(MultilinearExtensionImpl::new(col))],
        )]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        assert_eq!(builder.consume_result_mle(), Scalar::zero());
        // specify an arbitrary evaluation so that verify fails
        builder.produce_sumcheck_subpolynomial_evaluation(&Scalar::from(123u64));
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    assert!(proof.verify(&expr, &accessor, &counts, &result).is_err());
}

#[test]
fn veriy_fails_if_result_mle_evaluation_fails() {
    // prove and try to verify an artificial polynomial where we prove
    // that every entry in the result is zero
    let counts = ProofCounts {
        table_length: 2,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 2,
        result_columns: 1,
        sumcheck_subpolynomials: 1,
        ..Default::default()
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        let col = alloc.alloc_slice_fill_copy(2, 0i64);
        let indexes = alloc.alloc_slice_fill_copy(1, 0u64);
        builder.set_result_indexes(indexes);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(col)));
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![(
            Scalar::one(),
            vec![Box::new(MultilinearExtensionImpl::new(col))],
        )]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        assert_eq!(builder.consume_result_mle(), Scalar::zero());
        builder.produce_sumcheck_subpolynomial_evaluation(&Scalar::zero());
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, mut result) = QueryProof::new(&expr, &accessor, &counts);
    result.indexes.pop();
    assert!(proof.verify(&expr, &accessor, &counts, &result).is_err());
}

#[test]
fn verify_fails_if_counts_dont_match() {
    // prove and verify an artificial polynomial where we try to prove
    // that every entry in the result is zero
    let mut counts = ProofCounts {
        table_length: 2,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 2,
        result_columns: 1,
        sumcheck_subpolynomials: 1,
        ..Default::default()
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        let col = alloc.alloc_slice_fill_copy(2, 0i64);
        let indexes = alloc.alloc_slice_fill_copy(1, 0u64);
        builder.set_result_indexes(indexes);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(col)));
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![(
            Scalar::one(),
            vec![Box::new(MultilinearExtensionImpl::new(col))],
        )]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        assert_eq!(builder.consume_result_mle(), Scalar::zero());
        builder.produce_sumcheck_subpolynomial_evaluation(&Scalar::zero());
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    counts.anchored_mles += 1;
    assert!(proof.verify(&expr, &accessor, &counts, &result).is_err());
}

fn verify_a_proof_with_an_anchored_commitment_and_given_offset(offset_generators: usize) {
    // prove and verify an artificial query where
    //     res_i = x_i * x_i
    // where the commitment for x is known
    static RES: [i64; 2] = [9, 25];
    static X: [i64; 2] = [3, 5];
    static INDEXES: [u64; 2] = [0u64, 1u64];
    let counts = ProofCounts {
        table_length: 2,
        offset_generators,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 3,
        result_columns: 1,
        sumcheck_subpolynomials: 1,
        anchored_mles: 1,
        ..Default::default()
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        _alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        builder.set_result_indexes(&INDEXES);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(&RES)));
        builder.produce_anchored_mle(&X);
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&RES))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&X)),
                    Box::new(MultilinearExtensionImpl::new(&X)),
                ],
            ),
        ]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let res_eval = builder.consume_result_mle();
        let x_commit = compute_commitment_for_testing(&X, counts.offset_generators);
        let x_eval = builder.consume_anchored_mle(&x_commit);
        let eval = builder.mle_evaluations.random_evaluation * (res_eval - x_eval * x_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    let result = proof
        .verify(&expr, &accessor, &counts, &result)
        .unwrap()
        .unwrap();
    let column_fields = expr
        .get_column_result_fields()
        .iter()
        .map(|v| v.into())
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_result =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![9, 25]))]).unwrap();
    assert_eq!(result, expected_result);

    // invalid offset will fail to verify
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    let invalid_counts = {
        let mut counts = counts;
        counts.offset_generators += 1;
        counts
    };
    assert!(proof
        .verify(&expr, &accessor, &invalid_counts, &result)
        .is_err());
}

#[test]
fn we_can_verify_a_proof_with_an_anchored_commitment_and_with_a_zero_offset() {
    verify_a_proof_with_an_anchored_commitment_and_given_offset(0);
}

#[test]
fn we_can_verify_a_proof_with_an_anchored_commitment_and_with_a_non_zero_offset() {
    verify_a_proof_with_an_anchored_commitment_and_given_offset(123);
}

#[test]
fn verify_fails_if_the_result_doesnt_satisfy_an_anchored_equation() {
    // attempt to prove and verify an artificial query where
    //     res_i = x_i * x_i
    // where the commitment for x is known and
    //     res_i != x_i * x_i
    // for some i
    static RES: [i64; 2] = [9, 26];
    static X: [i64; 2] = [3, 5];
    static INDEXES: [u64; 2] = [0u64, 1u64];
    let counts = ProofCounts {
        table_length: 2,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 3,
        result_columns: 1,
        sumcheck_subpolynomials: 1,
        anchored_mles: 1,
        ..Default::default()
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        _alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        builder.set_result_indexes(&INDEXES);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(&RES)));
        builder.produce_anchored_mle(&X);
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&RES))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&X)),
                    Box::new(MultilinearExtensionImpl::new(&X)),
                ],
            ),
        ]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let res_eval = builder.consume_result_mle();
        let x_commit = compute_commitment_for_testing(&X, 0_usize);
        let x_eval = builder.consume_anchored_mle(&x_commit);
        let eval = builder.mle_evaluations.random_evaluation * (res_eval - x_eval * x_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    assert!(proof.verify(&expr, &accessor, &counts, &result).is_err());
}

#[test]
fn verify_fails_if_the_anchored_commitment_doesnt_match() {
    // prove and verify an artificial query where
    //     res_i = x_i * x_i
    // where the commitment for x is known
    static RES: [i64; 2] = [9, 25];
    static X: [i64; 2] = [3, 5];
    static INDEXES: [u64; 2] = [0u64, 1u64];
    let counts = ProofCounts {
        table_length: 2,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 3,
        result_columns: 1,
        sumcheck_subpolynomials: 1,
        anchored_mles: 1,
        ..Default::default()
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        _alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        builder.set_result_indexes(&INDEXES);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(&RES)));
        builder.produce_anchored_mle(&X);
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&RES))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&X)),
                    Box::new(MultilinearExtensionImpl::new(&X)),
                ],
            ),
        ]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let res_eval = builder.consume_result_mle();
        let x_commit = Scalar::from(2u64) * compute_commitment_for_testing(&X, 0_usize);
        let x_eval = builder.consume_anchored_mle(&x_commit);
        let eval = builder.mle_evaluations.random_evaluation * (res_eval - x_eval * x_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    assert!(proof.verify(&expr, &accessor, &counts, &result).is_err());
}

fn verify_a_proof_with_an_intermediate_commitment_and_given_offset(offset_generators: usize) {
    // prove and verify an artificial query where
    //     z_i = x_i * x_i
    //     res_i = z_i * z_i
    // where the commitment for x is known
    static RES: [i64; 2] = [81, 625];
    static Z: [i64; 2] = [9, 25];
    static X: [i64; 2] = [3, 5];
    static INDEXES: [u64; 2] = [0u64, 1u64];
    let counts = ProofCounts {
        table_length: 2,
        offset_generators,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 3,
        result_columns: 1,
        sumcheck_subpolynomials: 2,
        anchored_mles: 1,
        intermediate_mles: 1,
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        _alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        builder.set_result_indexes(&INDEXES);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(&RES)));
        builder.produce_anchored_mle(&X);
        builder.produce_intermediate_mle(&Z);

        // poly1
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&Z))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&X)),
                    Box::new(MultilinearExtensionImpl::new(&X)),
                ],
            ),
        ]));

        // poly2
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&RES))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&Z)),
                    Box::new(MultilinearExtensionImpl::new(&Z)),
                ],
            ),
        ]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let x_commit = compute_commitment_for_testing(&X, counts.offset_generators);
        let res_eval = builder.consume_result_mle();
        let x_eval = builder.consume_anchored_mle(&x_commit);
        let z_eval = builder.consume_intermediate_mle();

        // poly1
        let eval = builder.mle_evaluations.random_evaluation * (z_eval - x_eval * x_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);

        // poly2
        let eval = builder.mle_evaluations.random_evaluation * (res_eval - z_eval * z_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    let result = proof
        .verify(&expr, &accessor, &counts, &result)
        .unwrap()
        .unwrap();
    let column_fields = expr
        .get_column_result_fields()
        .iter()
        .map(|v| v.into())
        .collect();
    let schema = Arc::new(Schema::new(column_fields));
    let expected_result =
        RecordBatch::try_new(schema, vec![Arc::new(Int64Array::from(vec![81, 625]))]).unwrap();
    assert_eq!(result, expected_result);

    // invalid offset will fail to verify
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    let invalid_counts = {
        let mut counts = counts;
        counts.offset_generators += 1;
        counts
    };
    assert!(proof
        .verify(&expr, &accessor, &invalid_counts, &result)
        .is_err());
}

#[test]
fn we_can_verify_a_proof_with_an_intermediate_commitment_and_with_a_zero_offset() {
    verify_a_proof_with_an_intermediate_commitment_and_given_offset(0);
}

#[test]
fn we_can_verify_a_proof_with_an_intermediate_commitment_and_with_a_non_zero_offset() {
    verify_a_proof_with_an_intermediate_commitment_and_given_offset(89);
}

#[test]
fn verify_fails_if_an_intermediate_commitment_doesnt_match() {
    // prove and verify an artificial query where
    //     z_i = x_i * x_i
    //     res_i = z_i * z_i
    // where the commitment for x is known
    static RES: [i64; 2] = [81, 625];
    static Z: [i64; 2] = [9, 25];
    static X: [i64; 2] = [3, 5];
    static INDEXES: [u64; 2] = [0u64, 1u64];
    let counts = ProofCounts {
        table_length: 2,
        offset_generators: 0,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 3,
        result_columns: 1,
        sumcheck_subpolynomials: 2,
        anchored_mles: 1,
        intermediate_mles: 1,
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        _alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        builder.set_result_indexes(&INDEXES);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(&RES)));
        builder.produce_anchored_mle(&X);
        builder.produce_intermediate_mle(&Z);

        // poly1
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&Z))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&X)),
                    Box::new(MultilinearExtensionImpl::new(&X)),
                ],
            ),
        ]));

        // poly2
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&RES))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&Z)),
                    Box::new(MultilinearExtensionImpl::new(&Z)),
                ],
            ),
        ]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let x_commit = compute_commitment_for_testing(&X, 0_usize);
        let res_eval = builder.consume_result_mle();
        let x_eval = builder.consume_anchored_mle(&x_commit);
        let z_eval = builder.consume_intermediate_mle();

        // poly1
        let eval = builder.mle_evaluations.random_evaluation * (z_eval - x_eval * x_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);

        // poly2
        let eval = builder.mle_evaluations.random_evaluation * (res_eval - z_eval * z_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (mut proof, result) = QueryProof::new(&expr, &accessor, &counts);
    proof.commitments[0] =
        (proof.commitments[0].decompress().unwrap() * Scalar::from(2u64)).compress();
    assert!(proof.verify(&expr, &accessor, &counts, &result).is_err());
}

#[test]
fn verify_fails_if_an_intermediate_commitment_cant_be_decompressed() {
    // prove and verify an artificial query where
    //     z_i = x_i * x_i
    //     res_i = z_i * z_i
    // where the commitment for x is known
    static RES: [i64; 2] = [81, 625];
    static Z: [i64; 2] = [9, 25];
    static X: [i64; 2] = [3, 5];
    static INDEXES: [u64; 2] = [0u64, 1u64];
    let counts = ProofCounts {
        table_length: 2,
        offset_generators: 0,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 3,
        result_columns: 1,
        sumcheck_subpolynomials: 2,
        anchored_mles: 1,
        intermediate_mles: 1,
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        _alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        builder.set_result_indexes(&INDEXES);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(&RES)));
        builder.produce_anchored_mle(&X);
        builder.produce_intermediate_mle(&Z);

        // poly1
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&Z))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&X)),
                    Box::new(MultilinearExtensionImpl::new(&X)),
                ],
            ),
        ]));

        // poly2
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&RES))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&Z)),
                    Box::new(MultilinearExtensionImpl::new(&Z)),
                ],
            ),
        ]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let x_commit = compute_commitment_for_testing(&X, 0_usize);
        let res_eval = builder.consume_result_mle();
        let x_eval = builder.consume_anchored_mle(&x_commit);
        let z_eval = builder.consume_intermediate_mle();

        // poly1
        let eval = builder.mle_evaluations.random_evaluation * (z_eval - x_eval * x_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);

        // poly2
        let eval = builder.mle_evaluations.random_evaluation * (res_eval - z_eval * z_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (mut proof, result) = QueryProof::new(&expr, &accessor, &counts);
    let mut bytes = [0u8; 32];
    bytes[31] = 1u8;
    let commit = CompressedRistretto::from_slice(&bytes);
    assert!(commit.decompress().is_none());
    proof.commitments[0] = commit;
    assert!(proof.verify(&expr, &accessor, &counts, &result).is_err());
}

#[test]
fn verify_fails_if_an_intermediate_equation_isnt_satified() {
    // attempt to prove and verify an artificial query where
    //     z_i = x_i * x_i
    //     res_i = z_i * z_i
    // where the commitment for x is known and
    //     z_i != x_i * x_i
    // for some i
    static RES: [i64; 2] = [81, 625];
    static Z: [i64; 2] = [9, 25];
    static X: [i64; 2] = [3, 4];
    static INDEXES: [u64; 2] = [0u64, 1u64];
    let counts = ProofCounts {
        table_length: 2,
        offset_generators: 0,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 3,
        result_columns: 1,
        sumcheck_subpolynomials: 2,
        anchored_mles: 1,
        intermediate_mles: 1,
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        _alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        builder.set_result_indexes(&INDEXES);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(&RES)));
        builder.produce_anchored_mle(&X);
        builder.produce_intermediate_mle(&Z);

        // poly1
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&Z))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&X)),
                    Box::new(MultilinearExtensionImpl::new(&X)),
                ],
            ),
        ]));

        // poly2
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&RES))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&Z)),
                    Box::new(MultilinearExtensionImpl::new(&Z)),
                ],
            ),
        ]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let x_commit = compute_commitment_for_testing(&X, 0_usize);
        let res_eval = builder.consume_result_mle();
        let x_eval = builder.consume_anchored_mle(&x_commit);
        let z_eval = builder.consume_intermediate_mle();

        // poly1
        let eval = builder.mle_evaluations.random_evaluation * (z_eval - x_eval * x_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);

        // poly2
        let eval = builder.mle_evaluations.random_evaluation * (res_eval - z_eval * z_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    assert!(proof.verify(&expr, &accessor, &counts, &result).is_err());
}

#[test]
fn verify_fails_the_result_doesnt_satisfy_an_intermediate_equation() {
    // attempt to prove and verify an artificial query where
    //     z_i = x_i * x_i
    //     res_i = z_i * z_i
    // where the commitment for x is known and
    //     res_i != z_i * z_i
    // for some i
    static RES: [i64; 2] = [81, 624];
    static Z: [i64; 2] = [9, 25];
    static X: [i64; 2] = [3, 5];
    static INDEXES: [u64; 2] = [0u64, 1u64];
    let counts = ProofCounts {
        table_length: 2,
        offset_generators: 0,
        sumcheck_variables: 1,
        sumcheck_max_multiplicands: 3,
        result_columns: 1,
        sumcheck_subpolynomials: 2,
        anchored_mles: 1,
        intermediate_mles: 1,
    };
    fn prover_eval<'a>(
        builder: &mut ProofBuilder<'a>,
        _alloc: &'a Bump,
        _counts: &ProofCounts,
        _accessor: &'a dyn DataAccessor,
    ) {
        builder.set_result_indexes(&INDEXES);
        builder.produce_result_column(Box::new(DenseProvableResultColumn::new(&RES)));
        builder.produce_anchored_mle(&X);
        builder.produce_intermediate_mle(&Z);

        // poly1
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&Z))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&X)),
                    Box::new(MultilinearExtensionImpl::new(&X)),
                ],
            ),
        ]));

        // poly2
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(vec![
            (
                Scalar::one(),
                vec![Box::new(MultilinearExtensionImpl::new(&RES))],
            ),
            (
                -Scalar::one(),
                vec![
                    Box::new(MultilinearExtensionImpl::new(&Z)),
                    Box::new(MultilinearExtensionImpl::new(&Z)),
                ],
            ),
        ]));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let x_commit = compute_commitment_for_testing(&X, 0_usize);
        let res_eval = builder.consume_result_mle();
        let x_eval = builder.consume_anchored_mle(&x_commit);
        let z_eval = builder.consume_intermediate_mle();

        // poly1
        let eval = builder.mle_evaluations.random_evaluation * (z_eval - x_eval * x_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);

        // poly2
        let eval = builder.mle_evaluations.random_evaluation * (res_eval - z_eval * z_eval);
        builder.produce_sumcheck_subpolynomial_evaluation(&eval);
    }
    let expr = TestQueryExpr {
        counts,
        prover_fn: Some(Box::new(prover_eval)),
        verifier_fn: Some(Box::new(verifier_eval)),
    };
    let accessor = TestAccessor::new();
    let (proof, result) = QueryProof::new(&expr, &accessor, &counts);
    assert!(proof.verify(&expr, &accessor, &counts, &result).is_err());
}
