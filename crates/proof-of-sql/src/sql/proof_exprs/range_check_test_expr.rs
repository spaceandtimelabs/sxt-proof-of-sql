use super::range_check::{count, final_round_evaluate_range_check, verifier_evaluate_range_check};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, OwnedTable, Table, TableRef},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        CountBuilder, FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate,
        VerificationBuilder,
    },
};
use bumpalo::Bump;
use proof_of_sql_parser::Identifier;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RangeCheckTestExpr {
    pub column: ColumnRef,
}

impl ProverEvaluate for RangeCheckTestExpr {
    fn first_round_evaluate(&self, builder: &mut FirstRoundBuilder) {
        builder.request_post_result_challenges(1);
    }

    fn result_evaluate<'a, S: Scalar>(
        &self,
        _alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        // Get the table from the map using the table reference
        let table: &Table<'a, S> = table_map
            .get(&self.column.table_ref())
            .expect("Table not found");
        table.clone()
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

        // Get the column identifier from `self.column`
        let column_id: &Identifier = &self.column.column_id();

        // Retrieve the column from the table
        let column = table
            .inner_table()
            .get(column_id)
            .expect("Column not found in table");

        let scalars = column
            .as_scalar()
            .expect("Failed to convert column to scalar");

        final_round_evaluate_range_check(builder, scalars, alloc);
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
        let mut refs = IndexSet::default();
        refs.insert(self.column);
        refs
    }

    #[doc = " Return all the tables referenced in the Query"]
    fn get_table_references(&self) -> IndexSet<TableRef> {
        let mut refs = IndexSet::default();
        refs.insert(self.column.table_ref());
        refs
    }

    #[doc = " Count terms used within the Query\'s proof"]
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        count(builder);
        Ok(())
    }

    #[doc = " Form components needed to verify and proof store into `VerificationBuilder`"]
    // pull out S, this is evaluation of column
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        // this gives the original eval of column
        _accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
    ) -> Result<Vec<S>, ProofError> {
        verifier_evaluate_range_check(builder);
        Ok(vec![])
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
        let data = owned_table([scalar("a", 1000..1256)]);
        // let data = owned_table([bigint("a", vec![0; 256])]);
        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let ast = RangeCheckTestExpr {
            column: ColumnRef::new(t, "a".parse().unwrap(), ColumnType::BigInt),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &());
        let res = verifiable_res.verify(&ast, &accessor, &());
        println!("failed verification: {:?}", res.is_err());
    }
}
