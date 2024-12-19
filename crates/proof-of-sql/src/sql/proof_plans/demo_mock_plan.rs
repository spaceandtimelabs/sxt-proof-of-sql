use crate::{
    base::{
        database::{ColumnField, ColumnRef, OwnedTable, Table, TableEvaluation, TableRef},
        map::{indexset, IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
    },
};
use alloc::vec::Vec;
use bumpalo::Bump;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct DemoMockPlan {
    pub column: ColumnRef,
}

impl ProofPlan for DemoMockPlan {
    fn verifier_evaluate<S: Scalar>(
        &self,
        _builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        one_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        // place verification logic you want to test here

        Ok(TableEvaluation::new(
            vec![accessor[&self.column]],
            one_eval_map[&self.column.table_ref()],
        ))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new(
            self.column.column_id(),
            *self.column.column_type(),
        )]
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {self.column.clone()}
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {self.column.table_ref()}
    }
}

impl ProverEvaluate for DemoMockPlan {
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FirstRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        // place prover logic you want to test here

        table_map[&self.column.table_ref()].clone()
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        // place prover logic you want to test here

        table_map[&self.column.table_ref()].clone()
    }
}

mod tests {
    use super::DemoMockPlan;
    use crate::{
        base::database::{
            owned_table_utility::{bigint, owned_table},
            ColumnRef, ColumnType, OwnedTableTestAccessor, TableRef,
        },
        sql::proof::VerifiableQueryResult,
    };
    #[cfg(feature = "blitzar")]
    use blitzar::proof::InnerProductProof;

    #[test]
    #[cfg(feature = "blitzar")]
    fn we_can_create_and_prove_a_demo_mock_plan() {
        let table_ref = "namespace.table_name".parse::<TableRef>().unwrap();
        let table = owned_table([bigint("column_name", [0, 1, 2, 3])]);
        let column_ref = ColumnRef::new(table_ref, "column_name".into(), ColumnType::BigInt);
        let plan = DemoMockPlan { column: column_ref };
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
            table_ref,
            table.clone(),
            0_usize,
            (),
        );
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&plan, &accessor, &());
        let res = verifiable_res
            .verify(&plan, &accessor, &())
            .expect("verification should suceeed")
            .table;
        assert_eq!(res, table);
    }
}
