use super::{FinalRoundBuilder, ProofPlan, ProverEvaluate, QueryProof, VerificationBuilder};
use crate::{
    base::{
        bit::BitDistribution,
        byte::ByteDistribution,
        commitment::InnerProductProof,
        database::{
            owned_table_utility::{bigint, owned_table},
            table_utility::*,
            ColumnField, ColumnRef, ColumnType, OwnedTable, OwnedTableTestAccessor, Table,
            TableEvaluation, TableRef,
        },
        map::{indexset, IndexMap, IndexSet},
        proof::ProofError,
        scalar::{test_scalar::TestScalar, Scalar},
    },
    proof_primitive::inner_product::curve_25519_scalar::Curve25519Scalar,
    sql::proof::{FirstRoundBuilder, QueryData, SumcheckSubpolynomialType},
};
use bumpalo::Bump;
use serde::Serialize;
use sqlparser::ast::Ident;

/// Type to allow us to prove and verify an artificial polynomial where we prove
/// that every entry in the result is zero
#[derive(Debug, Serialize)]
struct TrivialTestProofPlan {
    length: usize,
    offset: usize,
    column_fill_value: i64,
    evaluation: i64,
    produce_length: bool,
    bit_distribution: Option<BitDistribution>,
    byte_distribution: Option<ByteDistribution>,
}
impl Default for TrivialTestProofPlan {
    fn default() -> Self {
        Self {
            length: 2,
            offset: 0,
            column_fill_value: 0,
            evaluation: 0,
            produce_length: true,
            bit_distribution: Some(BitDistribution {
                leading_bit_mask: [0; 4],
                vary_mask: [0; 4],
            }),
            byte_distribution: Some(ByteDistribution::new::<TestScalar, TestScalar>(&[])),
        }
    }
}
impl ProverEvaluate for TrivialTestProofPlan {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let col = vec![self.column_fill_value; self.length];
        if self.produce_length {
            builder.produce_chi_evaluation_length(self.length);
        }
        table([borrowed_bigint("a1", col, alloc)])
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let col = alloc.alloc_slice_fill_copy(self.length, self.column_fill_value);
        builder.produce_intermediate_mle(col as &[_]);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![(S::ONE, vec![Box::new(col as &[_])])],
        );
        if let Some(bit_distribution) = &self.bit_distribution {
            builder.produce_bit_distribution(bit_distribution.clone());
        }
        table([borrowed_bigint(
            "a1",
            vec![self.column_fill_value; self.length],
            alloc,
        )])
    }
}
impl ProofPlan for TrivialTestProofPlan {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        _accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        _chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        assert_eq!(builder.try_consume_final_round_mle_evaluation()?, S::ZERO);
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::ZeroSum,
            S::from(self.evaluation),
            1,
        )?;
        let _ = builder.try_consume_bit_distribution()?;
        Ok(TableEvaluation::new(
            vec![S::ZERO],
            builder.try_consume_chi_evaluation()?,
        ))
    }
    ///
    /// # Panics
    ///
    /// This method will panic if the `ColumnField` cannot be created from the provided column name (e.g., if the name parsing fails).
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new("a1".into(), ColumnType::BigInt)]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {}
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset![TableRef::new("sxt", "test")]
    }
}

fn verify_a_trivial_query_proof_with_given_offset(n: usize, offset_generators: usize) {
    let expr = TrivialTestProofPlan {
        length: n,
        offset: offset_generators,
        ..Default::default()
    };
    let column: Vec<i64> = vec![0_i64; n];
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("a1", column.clone())]),
        offset_generators,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash,
        table,
    } = proof
        .clone()
        .verify(&expr, &accessor, result.clone(), &())
        .unwrap();
    assert_ne!(verification_hash, [0; 32]);
    let expected_result = owned_table([bigint("a1", column)]);
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
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("a1", [123_i64; 2])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
}

#[test]
fn verify_fails_if_the_sumcheck_evaluation_isnt_correct() {
    // set up a proof for an artificial polynomial and specify an evaluation that won't
    // match the evaluation from sumcheck
    let expr = TrivialTestProofPlan {
        evaluation: 123,
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("a1", [123_i64; 2])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
}

#[test]
fn verify_fails_if_counts_dont_match() {
    // prove and verify an artificial polynomial where we try to prove
    // that every entry in the result is zero
    let expr = TrivialTestProofPlan {
        produce_length: false,
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("a1", [0_i64; 2])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
}

#[test]
fn verify_fails_if_the_number_of_bit_distributions_is_not_enough() {
    let expr = TrivialTestProofPlan {
        bit_distribution: None,
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("a1", [0_i64; 2])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
}

#[test]
fn verify_fails_if_a_bit_distribution_is_invalid() {
    let expr = TrivialTestProofPlan {
        bit_distribution: Some(BitDistribution {
            leading_bit_mask: [1; 4],
            vary_mask: [1; 4],
        }),
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("a1", [0_i64; 2])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
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
impl ProverEvaluate for SquareTestProofPlan {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        builder.produce_chi_evaluation_length(2);
        table([borrowed_bigint("a1", self.res, alloc)])
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let x = *table_map
            .get(&TableRef::new("sxt", "test"))
            .unwrap()
            .inner_table()
            .get(&Ident::new("x"))
            .unwrap();
        let res: &[_] = alloc.alloc_slice_copy(&self.res);
        builder.produce_intermediate_mle(res);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::ONE, vec![Box::new(res)]),
                (-S::ONE, vec![Box::new(x), Box::new(x)]),
            ],
        );
        table([borrowed_bigint("a1", self.res, alloc)])
    }
}
impl ProofPlan for SquareTestProofPlan {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        _chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        let x_eval = S::from(self.anchored_commit_multiplier)
            * *accessor
                .get(&ColumnRef::new(
                    TableRef::new("sxt", "test"),
                    "x".into(),
                    ColumnType::BigInt,
                ))
                .unwrap();
        let res_eval = builder.try_consume_final_round_mle_evaluation()?;
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            res_eval - x_eval * x_eval,
            2,
        )?;
        Ok(TableEvaluation::new(
            vec![res_eval],
            builder.try_consume_chi_evaluation()?,
        ))
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new("a1".into(), ColumnType::BigInt)]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {ColumnRef::new(
        TableRef::new("sxt", "test"),
              "x".into(),
              ColumnType::BigInt,
          )}
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset![TableRef::new("sxt", "test")]
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
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        offset_generators,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash,
        table,
    } = proof
        .clone()
        .verify(&expr, &accessor, result.clone(), &())
        .unwrap();
    assert_ne!(verification_hash, [0; 32]);
    let expected_result = owned_table([bigint("a1", [9, 25])]);
    assert_eq!(table, expected_result);

    // invalid offset will fail to verify
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        offset_generators + 1,
        (),
    );
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
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
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
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
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
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
impl ProverEvaluate for DoubleSquareTestProofPlan {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        builder.produce_chi_evaluation_length(2);
        table([borrowed_bigint("a1", self.res, alloc)])
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let x = *table_map
            .get(&TableRef::new("sxt", "test"))
            .unwrap()
            .inner_table()
            .get(&Ident::new("x"))
            .unwrap();
        let res: &[_] = alloc.alloc_slice_copy(&self.res);
        let z: &[_] = alloc.alloc_slice_copy(&self.z);
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
        table([borrowed_bigint("a1", self.res, alloc)])
    }
}
impl ProofPlan for DoubleSquareTestProofPlan {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        _chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        let x_eval = *accessor
            .get(&ColumnRef::new(
                TableRef::new("sxt", "test"),
                "x".into(),
                ColumnType::BigInt,
            ))
            .unwrap();
        let z_eval = builder.try_consume_final_round_mle_evaluation()?;
        let res_eval = builder.try_consume_final_round_mle_evaluation()?;

        // poly1
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            z_eval - x_eval * x_eval,
            2,
        )?;

        // poly2
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            res_eval - z_eval * z_eval,
            2,
        )?;
        Ok(TableEvaluation::new(
            vec![res_eval],
            builder.try_consume_chi_evaluation()?,
        ))
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new("a1".into(), ColumnType::BigInt)]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {ColumnRef::new(
        TableRef::new("sxt", "test"),
              "x".into(),
              ColumnType::BigInt,
          )}
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset![TableRef::new("sxt", "test")]
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
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        offset_generators,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash,
        table,
    } = proof
        .clone()
        .verify(&expr, &accessor, result.clone(), &())
        .unwrap();
    assert_ne!(verification_hash, [0; 32]);
    let expected_result = owned_table([bigint("a1", [81, 625])]);
    assert_eq!(table, expected_result);

    // invalid offset will fail to verify
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        offset_generators + 1,
        (),
    );
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
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
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        0,
        (),
    );
    let (mut proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    proof.final_round_message.round_commitments[0] =
        proof.final_round_message.round_commitments[0] * Curve25519Scalar::from(2u64);
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
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
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 4])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
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
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
}

#[derive(Debug, Serialize)]
struct ChallengeTestProofPlan {}
impl ProverEvaluate for ChallengeTestProofPlan {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        builder.request_post_result_challenges(2);
        builder.produce_chi_evaluation_length(2);
        table([borrowed_bigint("a1", [9, 25], alloc)])
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let x = *table_map
            .get(&TableRef::new("sxt", "test"))
            .unwrap()
            .inner_table()
            .get(&Ident::new("x"))
            .unwrap();
        let res: &[_] = alloc.alloc_slice_copy(&[9, 25]);
        let alpha = builder.consume_post_result_challenge();
        let _beta = builder.consume_post_result_challenge();
        builder.produce_intermediate_mle(res);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (alpha, vec![Box::new(res)]),
                (-alpha, vec![Box::new(x), Box::new(x)]),
            ],
        );
        table([borrowed_bigint("a1", [9, 25], alloc)])
    }
}
impl ProofPlan for ChallengeTestProofPlan {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        _chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        let alpha = builder.try_consume_post_result_challenge()?;
        let _beta = builder.try_consume_post_result_challenge()?;
        let x_eval = *accessor
            .get(&ColumnRef::new(
                TableRef::new("sxt", "test"),
                "x".into(),
                ColumnType::BigInt,
            ))
            .unwrap();
        let res_eval = builder.try_consume_final_round_mle_evaluation()?;
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            alpha * res_eval - alpha * x_eval * x_eval,
            2,
        )?;
        Ok(TableEvaluation::new(
            vec![res_eval],
            builder.try_consume_chi_evaluation()?,
        ))
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new("a1".into(), ColumnType::BigInt)]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {ColumnRef::new(
            TableRef::new("sxt", "test"),
            "x".into(),
            ColumnType::BigInt,
        )}
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset![TableRef::new("sxt", "test")]
    }
}

fn verify_a_proof_with_a_post_result_challenge_and_given_offset(offset_generators: usize) {
    // prove and verify an artificial query where
    //     alpha * res_i = alpha * x_i * x_i
    // where the commitment for x is known and alpha depends on res
    // additionally, we will have a second challenge beta, that is unused
    let expr = ChallengeTestProofPlan {};
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        offset_generators,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash,
        table,
    } = proof
        .clone()
        .verify(&expr, &accessor, result.clone(), &())
        .unwrap();
    assert_ne!(verification_hash, [0; 32]);
    let expected_result = owned_table([bigint("a1", [9, 25])]);
    assert_eq!(table, expected_result);

    // invalid offset will fail to verify
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        offset_generators + 1,
        (),
    );
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
}

#[test]
fn we_can_verify_a_proof_with_a_post_result_challenge_and_with_a_zero_offset() {
    verify_a_proof_with_a_post_result_challenge_and_given_offset(0);
}

#[test]
fn we_can_verify_a_proof_with_a_post_result_challenge_and_with_a_non_zero_offset() {
    verify_a_proof_with_a_post_result_challenge_and_given_offset(123);
}

/// prove and verify an artificial query where
///     `res_i = x_i * x_i`
/// where the commitment for x is known
#[derive(Debug, Serialize)]
struct FirstRoundSquareTestProofPlan {
    res: [i64; 2],
    anchored_commit_multiplier: i64,
}
impl Default for FirstRoundSquareTestProofPlan {
    fn default() -> Self {
        Self {
            res: [9, 25],
            anchored_commit_multiplier: 1,
        }
    }
}
impl ProverEvaluate for FirstRoundSquareTestProofPlan {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let res: &[_] = alloc.alloc_slice_copy(&self.res);
        builder.produce_intermediate_mle(res);
        builder.produce_chi_evaluation_length(2);
        table([borrowed_bigint("a1", self.res, alloc)])
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        let x = *table_map
            .get(&TableRef::new("sxt", "test"))
            .unwrap()
            .inner_table()
            .get(&Ident::new("x"))
            .unwrap();
        let res: &[_] = alloc.alloc_slice_copy(&self.res);
        builder.produce_intermediate_mle(res);
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::ONE, vec![Box::new(res)]),
                (-S::ONE, vec![Box::new(x), Box::new(x)]),
            ],
        );
        table([borrowed_bigint("a1", self.res, alloc)])
    }
}
impl ProofPlan for FirstRoundSquareTestProofPlan {
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        _chi_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        let x_eval = S::from(self.anchored_commit_multiplier)
            * *accessor
                .get(&ColumnRef::new(
                    TableRef::new("sxt", "test"),
                    "x".into(),
                    ColumnType::BigInt,
                ))
                .unwrap();
        let first_round_res_eval = builder.try_consume_first_round_mle_evaluation()?;
        let final_round_res_eval = builder.try_consume_final_round_mle_evaluation()?;
        assert_eq!(first_round_res_eval, final_round_res_eval);
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            final_round_res_eval - x_eval * x_eval,
            2,
        )?;
        Ok(TableEvaluation::new(
            vec![final_round_res_eval],
            builder.try_consume_chi_evaluation()?,
        ))
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new("a1".into(), ColumnType::BigInt)]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {ColumnRef::new(
            TableRef::new("sxt", "test"),
            "x".into(),
            ColumnType::BigInt,
        )}
    }
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset![TableRef::new("sxt", "test")]
    }
}

fn verify_a_proof_with_a_commitment_and_given_offset(offset_generators: usize) {
    // prove and verify an artificial query where
    //     res_i = x_i * x_i
    // where the commitment for x is known
    let expr = FirstRoundSquareTestProofPlan {
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        offset_generators,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash,
        table,
    } = proof
        .clone()
        .verify(&expr, &accessor, result.clone(), &())
        .unwrap();
    assert_ne!(verification_hash, [0; 32]);
    let expected_result = owned_table([bigint("a1", [9, 25])]);
    assert_eq!(table, expected_result);

    // invalid offset will fail to verify
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        offset_generators + 1,
        (),
    );
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
}

#[test]
fn we_can_verify_a_proof_with_a_commitment_and_with_a_zero_offset() {
    verify_a_proof_with_a_commitment_and_given_offset(0);
}

#[test]
fn we_can_verify_a_proof_with_a_commitment_and_with_a_non_zero_offset() {
    verify_a_proof_with_a_commitment_and_given_offset(123);
}

#[test]
fn verify_fails_if_the_result_doesnt_satisfy_an_equation() {
    // attempt to prove and verify an artificial query where
    //     res_i = x_i * x_i
    // where the commitment for x is known and
    //     res_i != x_i * x_i
    // for some i
    let expr = FirstRoundSquareTestProofPlan {
        res: [9, 26],
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
}

#[test]
fn verify_fails_if_the_commitment_doesnt_match() {
    // prove and verify an artificial query where
    //     res_i = x_i * x_i
    // where the commitment for x is known
    let expr = FirstRoundSquareTestProofPlan {
        anchored_commit_multiplier: 2,
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        TableRef::new("sxt", "test"),
        owned_table([bigint("x", [3, 5])]),
        0,
        (),
    );
    let (proof, result) = QueryProof::<InnerProductProof>::new(&expr, &accessor, &());
    assert!(proof.verify(&expr, &accessor, result, &()).is_err());
}
