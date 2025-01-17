//! This module contains the implementation of the `UniquenessTestPlan` struct. This struct
//! is used to check whether the uniqueness gadget works correctly.
use super::uniqueness::{
    final_round_evaluate_uniqueness, first_round_evaluate_uniqueness, verify_uniqueness,
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
use bumpalo::Bump;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct UniquenessTestPlan {
    pub column: ColumnRef,
}

impl ProverEvaluate for UniquenessTestPlan {
    #[doc = "Evaluate the query, modify `FirstRoundBuilder` and return the result."]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        // Get the tables from the map using the table reference
        let table: &Table<'a, S> = table_map
            .get(&self.column.table_ref())
            .expect("Table not found");
        let num_rows = table.num_rows();
        builder.request_post_result_challenges(2);
        builder.produce_one_evaluation_length(num_rows);
        // Evaluate the first round
        first_round_evaluate_uniqueness(builder, num_rows);
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
        // Get the table from the map using the table reference
        let table: &Table<'a, S> = table_map
            .get(&self.column.table_ref())
            .expect("Table not found");
        let raw_column: Vec<S> = table
            .inner_table()
            .get(&self.column.column_id())
            .expect("Column not found in table")
            .to_scalar_with_scaling(0);
        let alloc_column = alloc.alloc_slice_copy(&raw_column);
        builder.produce_intermediate_mle(alloc_column as &[_]);
        let alpha = builder.consume_post_result_challenge();
        let beta = builder.consume_post_result_challenge();
        final_round_evaluate_uniqueness(builder, alloc, alpha, beta, alloc_column);
        // Return a dummy table
        Table::try_new_with_options(IndexMap::default(), TableOptions { row_count: Some(0) })
            .unwrap()
    }
}

impl ProofPlan for UniquenessTestPlan {
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![]
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {self.column.clone()}
    }

    #[doc = "Return all the tables referenced in the Query"]
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.column.table_ref()}
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
        // Get evaluations
        let column_eval = builder.try_consume_final_round_mle_evaluation()?;
        let one_eval = builder.try_consume_one_evaluation()?;
        // Evaluate the verifier
        verify_uniqueness(builder, alpha, beta, column_eval, one_eval)?;
        Ok(TableEvaluation::new(vec![], S::zero()))
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use super::*;
    use crate::{
        base::database::{table_utility::*, ColumnType, TableTestAccessor},
        sql::proof::VerifiableQueryResult,
    };
    use blitzar::proof::InnerProductProof;

    #[test]
    fn we_can_figure_out_that_unique_increasing_columns_are_unique() {
        let alloc = Bump::new();
        let table = table([
            borrowed_bigint("a", [1, 2], &alloc),
            borrowed_boolean("c", [false, true], &alloc),
        ]);
        let table_ref = "sxt.table".parse().unwrap();
        let accessor =
            TableTestAccessor::<InnerProductProof>::new_from_table(table_ref, table, 0, ());

        // BigInt column
        let plan = UniquenessTestPlan {
            column: ColumnRef::new(table_ref, "a".into(), ColumnType::BigInt),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_ok());

        // Boolean column
        let plan = UniquenessTestPlan {
            column: ColumnRef::new(table_ref, "c".into(), ColumnType::Boolean),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_ok());
    }

    #[test]
    fn we_cannot_pass_uniqueness_check_if_column_is_not_unique() {
        let alloc = Bump::new();
        let table = table([
            borrowed_bigint("a", [1, 1, 2], &alloc),
            borrowed_varchar("b", ["Space"; 3], &alloc),
            borrowed_boolean("c", [true, false, true], &alloc),
        ]);
        let table_ref = "sxt.table".parse().unwrap();
        let accessor =
            TableTestAccessor::<InnerProductProof>::new_from_table(table_ref, table, 0, ());

        // BigInt column
        let plan = UniquenessTestPlan {
            column: ColumnRef::new(table_ref, "a".into(), ColumnType::BigInt),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_err());

        // Varchar column
        let plan = UniquenessTestPlan {
            column: ColumnRef::new(table_ref, "b".into(), ColumnType::VarChar),
        };

        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_err());

        // Boolean column
        let plan = UniquenessTestPlan {
            column: ColumnRef::new(table_ref, "c".into(), ColumnType::Boolean),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res.verify(&plan, &accessor, &());
        assert!(res.is_err());
    }
}
