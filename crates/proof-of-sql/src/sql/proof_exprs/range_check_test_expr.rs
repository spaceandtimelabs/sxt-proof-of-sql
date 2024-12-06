use super::range_check::{
    count_range_check, final_round_evaluate_range_check, verifier_evaluate_range_check,
};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, OwnedTable, Table, TableEvaluation, TableRef},
        map::{indexset, IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        CountBuilder, FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
        VerificationBuilder,
    },
};
use bumpalo::Bump;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RangeCheckTestExpr {
    pub column: ColumnRef,
}

impl ProverEvaluate for RangeCheckTestExpr {
    #[doc = " Evaluate the query, modify `FirstRoundBuilder` and return the result."]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FirstRoundBuilder,
        _alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        builder.request_post_result_challenges(1);
        table_map[&self.column.table_ref()].clone()
    }

    // extract data to test on from here, feed it into range check
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

        let scalars = table
            .inner_table()
            .get(&self.column.column_id())
            .expect("Column not found in table")
            .as_scalar()
            .expect("Failed to convert column to scalar");
        final_round_evaluate_range_check(builder, scalars, 256, alloc);
        table.clone()
    }
}

impl ProofPlan for RangeCheckTestExpr {
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new(
            self.column.column_id(),
            *self.column.column_type(),
        )]
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {self.column}
    }

    #[doc = " Return all the tables referenced in the Query"]
    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.column.table_ref()}
    }

    #[doc = " Count terms used within the Query\'s proof"]
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        count_range_check(builder);
        Ok(())
    }

    #[doc = " Form components needed to verify and proof store into `VerificationBuilder`"]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        one_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        verifier_evaluate_range_check(builder, one_eval_map);

        Ok(TableEvaluation::new(
            vec![accessor[&self.column]],
            one_eval_map[&self.column.table_ref()],
        ))
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {

    use crate::{
        base::database::{
            owned_table_utility::{owned_table, scalar},
            ColumnRef, ColumnType, OwnedTableTestAccessor,
        },
        sql::{
            proof::VerifiableQueryResult, proof_exprs::range_check_test_expr::RangeCheckTestExpr,
        },
    };
    use blitzar::proof::InnerProductProof;

    #[test]
    fn we_can_prove_a_range_check() {
        let data = owned_table([scalar("a", 0..256)]);
        // let data = owned_table([bigint("a", vec![0; 256])]);
        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let ast = RangeCheckTestExpr {
            column: ColumnRef::new(t, "a".parse().unwrap(), ColumnType::Scalar),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &());
        let res = verifiable_res.verify(&ast, &accessor, &());
        println!("failed verification: {:?}", res.err());
    }
}
