use super::{
    make_sumcheck_term, DenseProvableResultColumn, ProofBuilder, ProofCounts, QueryProof,
    SumcheckSubpolynomial, TestQueryExpr, VerificationBuilder,
};
use crate::base::database::{CommitmentAccessor, DataAccessor, TestAccessor};
use crate::base::scalar::compute_commitment_for_testing;
use crate::sql::proof::QueryExpr;
use arrow::array::Int64Array;
use arrow::record_batch::RecordBatch;
use bumpalo::Bump;
use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar};
use std::sync::Arc;

#[test]
fn we_can_verify_a_trivial_query_proof() {
    // prove and verify an artificial polynomial where we prove
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
        let terms = vec![(Scalar::one(), vec![make_sumcheck_term(1, col)])];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
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
    let expected_result =
        RecordBatch::try_new(expr.get_result_schema(), vec![Arc::new(Int64Array::from(vec![0]))]).unwrap();
    assert_eq!(result, expected_result);
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
        let terms = vec![(Scalar::one(), vec![make_sumcheck_term(1, col)])];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
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
        let terms = vec![(Scalar::one(), vec![make_sumcheck_term(1, col)])];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
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
        let terms = vec![(Scalar::one(), vec![make_sumcheck_term(1, col)])];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
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
        let terms = vec![(Scalar::one(), vec![make_sumcheck_term(1, col)])];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
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

#[test]
fn we_can_verify_a_proof_with_an_anchored_commitment() {
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
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &RES)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &X), make_sumcheck_term(1, &X)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let res_eval = builder.consume_result_mle();
        let x_commit = compute_commitment_for_testing(&X);
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
    let expected_result = RecordBatch::try_new(
        expr.get_result_schema(),
        vec![Arc::new(Int64Array::from(vec![9, 25]))],
    )
    .unwrap();
    assert_eq!(result, expected_result);
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
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &RES)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &X), make_sumcheck_term(1, &X)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let res_eval = builder.consume_result_mle();
        let x_commit = compute_commitment_for_testing(&X);
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
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &RES)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &X), make_sumcheck_term(1, &X)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let res_eval = builder.consume_result_mle();
        let x_commit = Scalar::from(2u64) * compute_commitment_for_testing(&X);
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
fn we_can_verify_a_proof_with_an_intermediate_commitment() {
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
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &Z)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &X), make_sumcheck_term(1, &X)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));

        // poly2
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &RES)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &Z), make_sumcheck_term(1, &Z)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let x_commit = compute_commitment_for_testing(&X);
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
    let expected_result = RecordBatch::try_new(
        expr.get_result_schema(),
        vec![Arc::new(Int64Array::from(vec![81, 625]))],
    )
    .unwrap();
    assert_eq!(result, expected_result);
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
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &Z)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &X), make_sumcheck_term(1, &X)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));

        // poly2
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &RES)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &Z), make_sumcheck_term(1, &Z)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let x_commit = compute_commitment_for_testing(&X);
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
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &Z)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &X), make_sumcheck_term(1, &X)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));

        // poly2
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &RES)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &Z), make_sumcheck_term(1, &Z)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let x_commit = compute_commitment_for_testing(&X);
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
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &Z)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &X), make_sumcheck_term(1, &X)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));

        // poly2
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &RES)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &Z), make_sumcheck_term(1, &Z)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let x_commit = compute_commitment_for_testing(&X);
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
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &Z)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &X), make_sumcheck_term(1, &X)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));

        // poly2
        let terms = vec![
            (Scalar::one(), vec![make_sumcheck_term(1, &RES)]),
            (
                -Scalar::one(),
                vec![make_sumcheck_term(1, &Z), make_sumcheck_term(1, &Z)],
            ),
        ];
        builder.produce_sumcheck_subpolynomial(SumcheckSubpolynomial::new(terms));
    }
    fn verifier_eval(
        builder: &mut VerificationBuilder,
        _counts: &ProofCounts,
        _accessor: &dyn CommitmentAccessor,
    ) {
        let x_commit = compute_commitment_for_testing(&X);
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
