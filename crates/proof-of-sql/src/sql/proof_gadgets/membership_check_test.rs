//! This module contains the implementation of the `MembershipCheckTestPlan` struct. This struct
//! is used to check whether the membership check gadgets work correctly.
use super::membership_check::{
    final_round_evaluate_membership_check, first_round_evaluate_membership_check,
    verify_membership_check,
};
use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, OwnedTable, Table, TableEvaluation, TableOptions, TableRef,
        },
        map::{indexset, IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
    },
};
use bumpalo::{
    collections::{vec::Vec as BumpVec, CollectIn},
    Bump,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MembershipCheckTestPlan {
    pub source_table: TableRef,
    pub candidate_table: TableRef,
    pub source_columns: Vec<ColumnRef>,
    pub candidate_columns: Vec<ColumnRef>,
    pub proposed_multiplicities: Vec<usize>,
}

impl ProverEvaluate for MembershipCheckTestPlan {
    #[doc = "Evaluate the query, modify `FirstRoundBuilder` and return the result."]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        // Check that the source and candidate columns have the same amount of columns
        assert_eq!(
            self.source_columns.len(),
            self.candidate_columns.len(),
            "Source and candidate columns must have the same length"
        );
        // Check that the source columns belong to the source table
        for col_ref in &self.source_columns {
            assert_eq!(self.source_table, col_ref.table_ref(), "Table not found");
        }
        // Check that the candidate columns belong to the candidate table
        for col_ref in &self.candidate_columns {
            assert_eq!(self.candidate_table, col_ref.table_ref(), "Table not found");
        }
        // Get the tables from the map using the table reference
        let source_table: &Table<'a, S> =
            table_map.get(&self.source_table).expect("Table not found");
        let candidate_table = table_map
            .get(&self.candidate_table)
            .expect("Table not found");
        // Produce one evaluation lengths
        builder.produce_one_evaluation_length(source_table.num_rows());
        builder.produce_one_evaluation_length(candidate_table.num_rows());
        // Evaluate the first round
        first_round_evaluate_membership_check(builder, &self.proposed_multiplicities, alloc);
        // This is just a dummy table, the actual data is not used
        Table::try_new_with_options(IndexMap::default(), TableOptions { row_count: Some(0) })
            .unwrap()
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        // Check that the source and candidate columns have the same amount of columns
        assert_eq!(
            self.source_columns.len(),
            self.candidate_columns.len(),
            "Source and candidate columns must have the same length"
        );
        // Check that the source columns belong to the source table
        for col_ref in &self.source_columns {
            assert_eq!(self.source_table, col_ref.table_ref(), "Table not found");
        }
        // Check that the candidate columns belong to the candidate table
        for col_ref in &self.candidate_columns {
            assert_eq!(self.candidate_table, col_ref.table_ref(), "Table not found");
        }
        // Get the table from the map using the table reference
        let source_table: &Table<'a, S> =
            table_map.get(&self.source_table).expect("Table not found");
        let source_columns = self
            .source_columns
            .iter()
            .map(|col_ref| {
                let col = *(source_table
                    .inner_table()
                    .get(&col_ref.column_id())
                    .expect("Column not found in table"));
                builder.produce_intermediate_mle(col);
                col
            })
            .collect_in::<BumpVec<_>>(alloc);
        let candidate_table = table_map
            .get(&self.candidate_table)
            .expect("Table not found");
        let candidate_columns = self
            .candidate_columns
            .iter()
            .map(|col_ref| {
                let col = *(candidate_table
                    .inner_table()
                    .get(&col_ref.column_id())
                    .expect("Column not found in table"));
                builder.produce_intermediate_mle(col);
                col
            })
            .collect_in::<BumpVec<_>>(alloc);
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();
        // Perform final membership check
        final_round_evaluate_membership_check(
            builder,
            alloc,
            alpha,
            beta,
            &source_columns,
            &candidate_columns,
            &self.proposed_multiplicities,
            source_table.num_rows(),
            candidate_table.num_rows(),
        );
        // Return a dummy table
        Table::try_new_with_options(IndexMap::default(), TableOptions { row_count: Some(0) })
            .unwrap()
    }
}

impl ProofPlan for MembershipCheckTestPlan {
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![]
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        self.source_columns
            .iter()
            .chain(self.candidate_columns.iter())
            .cloned()
            .collect()
    }

    #[doc = "Return all the tables referenced in the Query"]
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.source_table, self.candidate_table}
    }

    #[doc = "Form components needed to verify and proof store into `VerificationBuilder`"]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        _accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        _one_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        // Get the challenges from the builder
        let alpha = builder.try_consume_post_result_challenge()?;
        let beta = builder.try_consume_post_result_challenge()?;
        let num_columns = self.source_columns.len();
        // Check that the source and candidate columns have the same amount of columns
        if num_columns != self.candidate_columns.len() {
            return Err(ProofError::VerificationError {
                error: "Source and candidate columns must have the same length",
            });
        }
        // Get the columns
        let column_evals = builder.try_consume_final_round_mle_evaluations(num_columns)?;
        // Get the target columns
        let candidate_subset_evals =
            builder.try_consume_final_round_mle_evaluations(num_columns)?;
        // Get the one evaluations
        let one_eval = builder.try_consume_one_evaluation()?;
        let candidate_subset_one_eval = builder.try_consume_one_evaluation()?;
        // Evaluate the verifier
        verify_membership_check(
            builder,
            alpha,
            beta,
            one_eval,
            candidate_subset_one_eval,
            &column_evals,
            &candidate_subset_evals,
        )?;
        Ok(TableEvaluation::new(vec![], S::zero()))
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{table_utility::*, ColumnType, TableTestAccessor, TestAccessor},
            scalar::Curve25519Scalar,
        },
        sql::proof::VerifiableQueryResult,
    };
    use blitzar::proof::InnerProductProof;

    #[test]
    fn we_can_do_minimal_membership_check() {
        let alloc = Bump::new();
        let source_table = table([borrowed_bigint("a", [1, 2, 3], &alloc)]);
        let candidate_table = table([borrowed_bigint("c", [1, 2, 2, 1, 2], &alloc)]);
        let source_table_ref = "sxt.source_table".parse().unwrap();
        let candidate_table_ref = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref,
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref, candidate_table, 0);
        let plan = MembershipCheckTestPlan {
            source_table: source_table_ref,
            candidate_table: candidate_table_ref,
            source_columns: vec![ColumnRef::new(
                source_table_ref,
                "a".into(),
                ColumnType::BigInt,
            )],
            candidate_columns: vec![ColumnRef::new(
                candidate_table_ref,
                "c".into(),
                ColumnType::BigInt,
            )],
            proposed_multiplicities: vec![2, 3, 0],
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_ok());
    }

    #[test]
    fn we_can_do_membership_check() {
        let alloc = Bump::new();
        let source_table = table([
            borrowed_bigint("a", [1, 2, 3], &alloc),
            borrowed_varchar("b", ["Space", "and", "Time"], &alloc),
            borrowed_boolean("c", [true, false, true], &alloc),
            borrowed_bigint("d", [5, 6, 7], &alloc),
        ]);
        let candidate_table = table([
            borrowed_bigint("c", [1, 2, 1, 1, 2], &alloc),
            borrowed_varchar("d", ["Space", "and", "Space", "Space", "and"], &alloc),
            borrowed_boolean("e", [true, false, true, true, false], &alloc),
            borrowed_bigint("f", [5, 6, 7, 8, 9], &alloc),
        ]);
        let source_table_ref = "sxt.source_table".parse().unwrap();
        let candidate_table_ref = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref,
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref, candidate_table, 0);
        let plan = MembershipCheckTestPlan {
            source_table: source_table_ref,
            candidate_table: candidate_table_ref,
            source_columns: vec![
                ColumnRef::new(source_table_ref, "a".into(), ColumnType::BigInt),
                ColumnRef::new(source_table_ref, "b".into(), ColumnType::VarChar),
                ColumnRef::new(source_table_ref, "c".into(), ColumnType::Boolean),
            ],
            candidate_columns: vec![
                ColumnRef::new(candidate_table_ref, "c".into(), ColumnType::BigInt),
                ColumnRef::new(candidate_table_ref, "d".into(), ColumnType::VarChar),
                ColumnRef::new(candidate_table_ref, "e".into(), ColumnType::Boolean),
            ],
            proposed_multiplicities: vec![3, 2, 0],
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_ok());
    }

    #[test]
    fn we_can_do_membership_check_when_candidate_table_has_no_rows() {
        let alloc = Bump::new();
        let source_table = table([
            borrowed_bigint("a", [1, 2, 3], &alloc),
            borrowed_varchar("b", ["Space", "and", "Time"], &alloc),
            borrowed_boolean("c", [true, false, true], &alloc),
            borrowed_bigint("d", [5, 6, 7], &alloc),
        ]);
        let candidate_table = table([
            borrowed_bigint("c", [0_i64; 0], &alloc),
            borrowed_varchar("d", [""; 0], &alloc),
            borrowed_boolean("e", [true; 0], &alloc),
            borrowed_bigint("f", [0_i64; 0], &alloc),
        ]);
        let source_table_ref = "sxt.source_table".parse().unwrap();
        let candidate_table_ref = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref,
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref, candidate_table, 0);
        let plan = MembershipCheckTestPlan {
            source_table: source_table_ref,
            candidate_table: candidate_table_ref,
            source_columns: vec![
                ColumnRef::new(source_table_ref, "a".into(), ColumnType::BigInt),
                ColumnRef::new(source_table_ref, "b".into(), ColumnType::VarChar),
                ColumnRef::new(source_table_ref, "c".into(), ColumnType::Boolean),
            ],
            candidate_columns: vec![
                ColumnRef::new(candidate_table_ref, "c".into(), ColumnType::BigInt),
                ColumnRef::new(candidate_table_ref, "d".into(), ColumnType::VarChar),
                ColumnRef::new(candidate_table_ref, "e".into(), ColumnType::Boolean),
            ],
            proposed_multiplicities: vec![0; 3],
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_ok());
    }

    #[test]
    fn we_cannot_do_membership_check_if_multiplicities_are_wrong() {
        let alloc = Bump::new();
        let source_table = table([
            borrowed_bigint("a", [1, 2], &alloc),
            borrowed_bigint("b", [3, 4], &alloc),
        ]);
        let candidate_table = table([
            borrowed_bigint("a", [1, 2, 1, 1, 2], &alloc),
            borrowed_bigint("b", [3, 4, 3, 3, 4], &alloc),
        ]);
        let source_table_ref = "sxt.source_table".parse().unwrap();
        let candidate_table_ref = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref,
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref, candidate_table, 0);
        let plan = MembershipCheckTestPlan {
            source_table: source_table_ref,
            candidate_table: candidate_table_ref,
            source_columns: vec![
                ColumnRef::new(source_table_ref, "a".into(), ColumnType::BigInt),
                ColumnRef::new(source_table_ref, "b".into(), ColumnType::BigInt),
            ],
            candidate_columns: vec![
                ColumnRef::new(candidate_table_ref, "a".into(), ColumnType::BigInt),
                ColumnRef::new(candidate_table_ref, "b".into(), ColumnType::BigInt),
            ],
            proposed_multiplicities: vec![4, 1],
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        assert!(verifiable_res.verify(&plan, &accessor, &()).is_err());
    }

    #[test]
    #[should_panic(expected = "Source and candidate columns must have the same length")]
    fn we_cannot_do_membership_check_if_source_and_candidate_have_different_number_of_columns() {
        let alloc = Bump::new();
        let source_table = table([
            borrowed_bigint("a", [1, 2], &alloc),
            borrowed_bigint("b", [3, 4], &alloc),
        ]);
        let candidate_table = table([borrowed_bigint("a", [1, 2, 1, 1, 2], &alloc)]);
        let source_table_ref = "sxt.source_table".parse().unwrap();
        let candidate_table_ref = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref,
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref, candidate_table, 0);
        let plan = MembershipCheckTestPlan {
            source_table: source_table_ref,
            candidate_table: candidate_table_ref,
            source_columns: vec![
                ColumnRef::new(source_table_ref, "a".into(), ColumnType::BigInt),
                ColumnRef::new(source_table_ref, "b".into(), ColumnType::BigInt),
            ],
            candidate_columns: vec![ColumnRef::new(
                candidate_table_ref,
                "a".into(),
                ColumnType::BigInt,
            )],
            proposed_multiplicities: vec![3, 2],
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        assert!(verifiable_res.verify(&plan, &accessor, &()).is_err());
    }

    #[test]
    fn we_can_do_membership_check_if_there_are_no_columns_in_the_tables() {
        let source_table = Table::<'_, Curve25519Scalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions { row_count: Some(5) },
        )
        .unwrap();
        let candidate_table = Table::<'_, Curve25519Scalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions { row_count: Some(4) },
        )
        .unwrap();
        let source_table_ref = "sxt.source_table".parse().unwrap();
        let candidate_table_ref = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref,
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref, candidate_table, 0);
        let plan = MembershipCheckTestPlan {
            source_table: source_table_ref,
            candidate_table: candidate_table_ref,
            source_columns: vec![],
            candidate_columns: vec![],
            proposed_multiplicities: vec![2, 0, 1, 1, 0],
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_ok());
    }

    #[test]
    fn we_can_do_membership_check_if_there_are_no_columns_in_the_tables_and_candidate_has_no_rows_either(
    ) {
        let source_table = Table::<'_, Curve25519Scalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions { row_count: Some(5) },
        )
        .unwrap();
        let candidate_table = Table::<'_, Curve25519Scalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions { row_count: Some(0) },
        )
        .unwrap();
        let source_table_ref = "sxt.source_table".parse().unwrap();
        let candidate_table_ref = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref,
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref, candidate_table, 0);
        let plan = MembershipCheckTestPlan {
            source_table: source_table_ref,
            candidate_table: candidate_table_ref,
            source_columns: vec![],
            candidate_columns: vec![],
            proposed_multiplicities: vec![0; 5],
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_ok());
    }

    #[test]
    fn we_can_do_membership_check_if_there_are_neither_rows_nor_columns_in_the_tables() {
        let source_table = Table::<'_, Curve25519Scalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions { row_count: Some(0) },
        )
        .unwrap();
        let candidate_table = Table::<'_, Curve25519Scalar>::try_new_with_options(
            IndexMap::default(),
            TableOptions { row_count: Some(0) },
        )
        .unwrap();
        let source_table_ref = "sxt.source_table".parse().unwrap();
        let candidate_table_ref = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref,
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref, candidate_table, 0);
        let plan = MembershipCheckTestPlan {
            source_table: source_table_ref,
            candidate_table: candidate_table_ref,
            source_columns: vec![],
            candidate_columns: vec![],
            proposed_multiplicities: vec![],
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_ok());
    }

    #[test]
    fn we_can_do_membership_check_if_no_column_is_selected() {
        let alloc = Bump::new();
        let source_table = table([
            borrowed_bigint("a", [1, 2], &alloc),
            borrowed_bigint("b", [3, 4], &alloc),
        ]);
        let candidate_table = table([
            borrowed_bigint("a", [1, 2, 1, 1, 2], &alloc),
            borrowed_bigint("b", [3, 4, 3, 3, 4], &alloc),
        ]);
        let source_table_ref = "sxt.source_table".parse().unwrap();
        let candidate_table_ref = "sxt.candidate_table".parse().unwrap();
        let mut accessor = TableTestAccessor::<InnerProductProof>::new_from_table(
            source_table_ref,
            source_table,
            0,
            (),
        );
        accessor.add_table(candidate_table_ref, candidate_table, 0);
        let plan = MembershipCheckTestPlan {
            source_table: source_table_ref,
            candidate_table: candidate_table_ref,
            source_columns: vec![],
            candidate_columns: vec![],
            proposed_multiplicities: vec![2, 3],
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_ok());
    }
}
