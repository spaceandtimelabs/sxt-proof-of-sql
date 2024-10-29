use super::{
    CountBuilder, FinalRoundBuilder, ProofPlan, ProverEvaluate, QueryProof, VerificationBuilder,
};
use crate::{
    base::{
        commitment::{Commitment, InnerProductProof},
        database::{
            owned_table_utility::{bigint, owned_table},
            Column, ColumnField, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor,
            MetadataAccessor, OwnedTable, OwnedTableTestAccessor, TableRef, TestAccessor,
            UnimplementedTestAccessor,
        },
        map::IndexSet,
        proof::ProofError,
        scalar::{Curve25519Scalar, Scalar},
    },
    sql::proof::{FirstRoundBuilder, QueryData, SumcheckSubpolynomialType},
};
use bumpalo::Bump;
use serde::Serialize;

/// Type to allow us to prove and verify an artificial polynomial where we prove
/// that every entry in the result is zero
#[derive(Debug, Serialize)]
struct TrivialTestProofPlan {
    length: usize,
    offset: usize,
    column_fill_value: i64,
    evaluation: i64,
    anchored_mle_count: usize,
}
impl Default for TrivialTestProofPlan {
    fn default() -> Self {
        Self {
            length: 2,
            offset: 0,
            column_fill_value: 0,
            evaluation: 0,
            anchored_mle_count: 0,
        }
    }
}
impl<S: Scalar> ProverEvaluate<S> for TrivialTestProofPlan {
    fn result_evaluate<'a>(
        &self,
        _input_length: usize,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let col = alloc.alloc_slice_fill_copy(self.length, self.column_fill_value);
        vec![Column::BigInt(col)]
    }

    fn first_round_evaluate(&self, _builder: &mut FirstRoundBuilder) {}

    fn final_round_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let col = alloc.alloc_slice_fill_copy(builder.table_length(), self.column_fill_value);
        builder.produce_intermediate_mle(col as &[_]);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![(S::ONE, vec![Box::new(col as &[_])])],
        );
        vec![Column::BigInt(col)]
    }
}
impl<C: Commitment> ProofPlan<C> for TrivialTestProofPlan {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_degree(2);
        builder.count_intermediate_mles(1);
        builder.count_subpolynomials(1);
        builder.count_anchored_mles(self.anchored_mle_count);
        Ok(())
    }
    fn get_length(&self, _accessor: &dyn MetadataAccessor) -> usize {
        self.length
    }
    fn get_offset(&self, _accessor: &dyn MetadataAccessor) -> usize {
        self.offset
    }
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        _accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        assert_eq!(builder.consume_intermediate_mle(), C::Scalar::ZERO);
        builder.produce_sumcheck_subpolynomial_evaluation(
            &SumcheckSubpolynomialType::ZeroSum,
            C::Scalar::from(self.evaluation),
        );
        Ok(vec![C::Scalar::ZERO])
    }
    ///
    /// # Panics
    ///
    /// This method will panic if the `ColumnField` cannot be created from the provided column name (e.g., if the name parsing fails).
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt)]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        unimplemented!("no real usage for this function yet")
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        unimplemented!("no real usage for this function yet")
    }
}

fn verify_a_trivial_query_proof_with_given_offset(n: usize, offset_generators: usize) {
    let expr = TrivialTestProofPlan {
        length: n,
        offset: offset_generators,
        ..Default::default()
    };
    let accessor = UnimplementedTestAccessor::new_empty();
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash,
        table,
    } = proof.verify(&expr, &accessor, &result, &()).unwrap();
    assert_ne!(verification_hash, [0; 32]);
    let expected_col = vec![0_i64; n];
    let expected_result = owned_table([bigint("a1", expected_col)]);
    assert_eq!(table, expected_result);
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
    let expr = TrivialTestProofPlan {
        column_fill_value: 123,
        ..Default::default()
    };
    let accessor = UnimplementedTestAccessor::new_empty();
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
}

#[test]
fn verify_fails_if_the_sumcheck_evaluation_isnt_correct() {
    // set up a proof for an artificial polynomial and specify an evaluation that won't
    // match the evaluation from sumcheck
    let expr = TrivialTestProofPlan {
        evaluation: 123,
        ..Default::default()
    };
    let accessor = UnimplementedTestAccessor::new_empty();
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
}

#[test]
fn verify_fails_if_counts_dont_match() {
    // prove and verify an artificial polynomial where we try to prove
    // that every entry in the result is zero
    let expr = TrivialTestProofPlan {
        anchored_mle_count: 1,
        ..Default::default()
    };
    let accessor = UnimplementedTestAccessor::new_empty();
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
}

/// prove and verify an artificial query where
///     `res_i = x_i * x_i`
/// where the commitment for x is known
#[derive(Debug, Serialize)]
struct SquareTestProofPlan {
    res: [i64; 2],
    anchored_commit_multiplier: i64,
}
impl Default for SquareTestProofPlan {
    fn default() -> Self {
        Self {
            res: [9, 25],
            anchored_commit_multiplier: 1,
        }
    }
}
impl<S: Scalar> ProverEvaluate<S> for SquareTestProofPlan {
    fn result_evaluate<'a>(
        &self,
        _table_length: usize,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let res: &[_] = alloc.alloc_slice_copy(&self.res);
        vec![Column::BigInt(res)]
    }

    fn first_round_evaluate(&self, _builder: &mut FirstRoundBuilder) {}

    fn final_round_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let x = accessor.get_column(ColumnRef::new(
            "sxt.test".parse().unwrap(),
            "x".parse().unwrap(),
            ColumnType::BigInt,
        ));
        let res: &[_] = alloc.alloc_slice_copy(&self.res);
        builder.produce_anchored_mle(x);
        builder.produce_intermediate_mle(res);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::ONE, vec![Box::new(res)]),
                (-S::ONE, vec![Box::new(x), Box::new(x)]),
            ],
        );
        vec![Column::BigInt(res)]
    }
}
impl<C: Commitment> ProofPlan<C> for SquareTestProofPlan {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_degree(3);
        builder.count_intermediate_mles(1);
        builder.count_subpolynomials(1);
        builder.count_anchored_mles(1);
        Ok(())
    }
    fn get_length(&self, _accessor: &dyn MetadataAccessor) -> usize {
        2
    }
    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset("sxt.test".parse().unwrap())
    }
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        let x_commit = C::Scalar::from(self.anchored_commit_multiplier)
            * accessor.get_commitment(ColumnRef::new(
                "sxt.test".parse().unwrap(),
                "x".parse().unwrap(),
                ColumnType::BigInt,
            ));
        let x_eval = builder.consume_anchored_mle(x_commit);
        let res_eval = builder.consume_intermediate_mle();
        builder.produce_sumcheck_subpolynomial_evaluation(
            &SumcheckSubpolynomialType::Identity,
            res_eval - x_eval * x_eval,
        );
        Ok(vec![res_eval])
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt)]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        unimplemented!("no real usage for this function yet")
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        unimplemented!("no real usage for this function yet")
    }
}

fn verify_a_proof_with_an_anchored_commitment_and_given_offset(offset_generators: usize) {
    // prove and verify an artificial query where
    //     res_i = x_i * x_i
    // where the commitment for x is known
    let expr = SquareTestProofPlan {
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 5])]),
        offset_generators,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash,
        table,
    } = proof.verify(&expr, &accessor, &result, &()).unwrap();
    assert_ne!(verification_hash, [0; 32]);
    let expected_result = owned_table([bigint("a1", [9, 25])]);
    assert_eq!(table, expected_result);

    // invalid offset will fail to verify
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 5])]),
        offset_generators + 1,
        (),
    );
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
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
    let expr = SquareTestProofPlan {
        res: [9, 26],
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 5])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
}

#[test]
fn verify_fails_if_the_anchored_commitment_doesnt_match() {
    // prove and verify an artificial query where
    //     res_i = x_i * x_i
    // where the commitment for x is known
    let expr = SquareTestProofPlan {
        anchored_commit_multiplier: 2,
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 5])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
}

// prove and verify an artificial query where
//     z_i = x_i * x_i
//     res_i = z_i * z_i
// where the commitment for x is known
#[derive(Debug, Serialize)]
struct DoubleSquareTestProofPlan {
    res: [i64; 2],
    z: [i64; 2],
}
impl Default for DoubleSquareTestProofPlan {
    fn default() -> Self {
        Self {
            res: [81, 625],
            z: [9, 25],
        }
    }
}
impl<S: Scalar> ProverEvaluate<S> for DoubleSquareTestProofPlan {
    fn result_evaluate<'a>(
        &self,
        _input_length: usize,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let res: &[_] = alloc.alloc_slice_copy(&self.res);
        vec![Column::BigInt(res)]
    }

    fn first_round_evaluate(&self, _builder: &mut FirstRoundBuilder) {}

    fn final_round_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let x = accessor.get_column(ColumnRef::new(
            "sxt.test".parse().unwrap(),
            "x".parse().unwrap(),
            ColumnType::BigInt,
        ));
        let res: &[_] = alloc.alloc_slice_copy(&self.res);
        let z: &[_] = alloc.alloc_slice_copy(&self.z);
        builder.produce_anchored_mle(x);
        builder.produce_intermediate_mle(z);

        // poly1
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::ONE, vec![Box::new(z)]),
                (-S::ONE, vec![Box::new(x), Box::new(x)]),
            ],
        );

        // poly2
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::ONE, vec![Box::new(res)]),
                (-S::ONE, vec![Box::new(z), Box::new(z)]),
            ],
        );
        builder.produce_intermediate_mle(res);
        vec![Column::BigInt(res)]
    }
}
impl<C: Commitment> ProofPlan<C> for DoubleSquareTestProofPlan {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_degree(3);
        builder.count_intermediate_mles(2);
        builder.count_subpolynomials(2);
        builder.count_anchored_mles(1);
        Ok(())
    }
    fn get_length(&self, _accessor: &dyn MetadataAccessor) -> usize {
        2
    }
    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset("sxt.test".parse().unwrap())
    }
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        let x_commit = accessor.get_commitment(ColumnRef::new(
            "sxt.test".parse().unwrap(),
            "x".parse().unwrap(),
            ColumnType::BigInt,
        ));
        let x_eval = builder.consume_anchored_mle(x_commit);
        let z_eval = builder.consume_intermediate_mle();
        let res_eval = builder.consume_intermediate_mle();

        // poly1
        builder.produce_sumcheck_subpolynomial_evaluation(
            &SumcheckSubpolynomialType::Identity,
            z_eval - x_eval * x_eval,
        );

        // poly2
        builder.produce_sumcheck_subpolynomial_evaluation(
            &SumcheckSubpolynomialType::Identity,
            res_eval - z_eval * z_eval,
        );
        Ok(vec![res_eval])
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt)]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        unimplemented!("no real usage for this function yet")
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        unimplemented!("no real usage for this function yet")
    }
}

fn verify_a_proof_with_an_intermediate_commitment_and_given_offset(offset_generators: usize) {
    // prove and verify an artificial query where
    //     z_i = x_i * x_i
    //     res_i = z_i * z_i
    // where the commitment for x is known
    let expr = DoubleSquareTestProofPlan {
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 5])]),
        offset_generators,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash,
        table,
    } = proof.verify(&expr, &accessor, &result, &()).unwrap();
    assert_ne!(verification_hash, [0; 32]);
    let expected_result = owned_table([bigint("a1", [81, 625])]);
    assert_eq!(table, expected_result);

    // invalid offset will fail to verify
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 5])]),
        offset_generators + 1,
        (),
    );
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
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
    let expr = DoubleSquareTestProofPlan {
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 5])]),
        0,
        (),
    );
    let (mut proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    proof.commitments[0] = proof.commitments[0] * Curve25519Scalar::from(2u64);
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
}

#[test]
fn verify_fails_if_an_intermediate_equation_isnt_satified() {
    // attempt to prove and verify an artificial query where
    //     z_i = x_i * x_i
    //     res_i = z_i * z_i
    // where the commitment for x is known and
    //     z_i != x_i * x_i
    // for some i
    let expr = DoubleSquareTestProofPlan {
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 4])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
}

#[test]
fn verify_fails_the_result_doesnt_satisfy_an_intermediate_equation() {
    // attempt to prove and verify an artificial query where
    //     z_i = x_i * x_i
    //     res_i = z_i * z_i
    // where the commitment for x is known and
    //     res_i != z_i * z_i
    // for some i
    let expr = DoubleSquareTestProofPlan {
        res: [81, 624],
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 5])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
}

#[derive(Debug, Serialize)]
struct ChallengeTestProofPlan {}
impl<S: Scalar> ProverEvaluate<S> for ChallengeTestProofPlan {
    fn result_evaluate<'a>(
        &self,
        _input_length: usize,
        _alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        vec![Column::BigInt(&[9, 25])]
    }

    fn first_round_evaluate(&self, builder: &mut FirstRoundBuilder) {
        builder.request_post_result_challenges(2);
    }

    fn final_round_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let x = accessor.get_column(ColumnRef::new(
            "sxt.test".parse().unwrap(),
            "x".parse().unwrap(),
            ColumnType::BigInt,
        ));
        let res: &[_] = alloc.alloc_slice_copy(&[9, 25]);
        let alpha = builder.consume_post_result_challenge();
        let _beta = builder.consume_post_result_challenge();
        builder.produce_anchored_mle(x);
        builder.produce_intermediate_mle(res);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (alpha, vec![Box::new(res)]),
                (-alpha, vec![Box::new(x), Box::new(x)]),
            ],
        );
        vec![Column::BigInt(&[9, 25])]
    }
}
impl<C: Commitment> ProofPlan<C> for ChallengeTestProofPlan {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_degree(3);
        builder.count_intermediate_mles(1);
        builder.count_subpolynomials(1);
        builder.count_anchored_mles(1);
        builder.count_post_result_challenges(2);
        Ok(())
    }
    fn get_length(&self, _accessor: &dyn MetadataAccessor) -> usize {
        2
    }
    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset("sxt.test".parse().unwrap())
    }
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        let alpha = builder.consume_post_result_challenge();
        let _beta = builder.consume_post_result_challenge();
        let x_commit = accessor.get_commitment(ColumnRef::new(
            "sxt.test".parse().unwrap(),
            "x".parse().unwrap(),
            ColumnType::BigInt,
        ));
        let x_eval = builder.consume_anchored_mle(x_commit);
        let res_eval = builder.consume_intermediate_mle();
        builder.produce_sumcheck_subpolynomial_evaluation(
            &SumcheckSubpolynomialType::Identity,
            alpha * res_eval - alpha * x_eval * x_eval,
        );
        Ok(vec![res_eval])
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new("a1".parse().unwrap(), ColumnType::BigInt)]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        unimplemented!("no real usage for this function yet")
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        unimplemented!("no real usage for this function yet")
    }
}

fn verify_a_proof_with_a_post_result_challenge_and_given_offset(offset_generators: usize) {
    // prove and verify an artificial query where
    //     alpha * res_i = alpha * x_i * x_i
    // where the commitment for x is known and alpha depends on res
    // additionally, we will have a second challenge beta, that is unused
    let expr = ChallengeTestProofPlan {};
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 5])]),
        offset_generators,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash,
        table,
    } = proof.verify(&expr, &accessor, &result, &()).unwrap();
    assert_ne!(verification_hash, [0; 32]);
    let expected_result = owned_table([bigint("a1", [9, 25])]);
    assert_eq!(table, expected_result);

    // invalid offset will fail to verify
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("x", [3, 5])]),
        offset_generators + 1,
        (),
    );
    assert!(proof.verify(&expr, &accessor, &result, &()).is_err());
}

#[test]
fn we_can_verify_a_proof_with_a_post_result_challenge_and_with_a_zero_offset() {
    verify_a_proof_with_a_post_result_challenge_and_given_offset(0);
}

#[test]
fn we_can_verify_a_proof_with_a_post_result_challenge_and_with_a_non_zero_offset() {
    verify_a_proof_with_a_post_result_challenge_and_given_offset(123);
}
