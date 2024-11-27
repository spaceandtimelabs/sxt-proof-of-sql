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
use ahash::AHasher;
use bumpalo::Bump;
use core::hash::BuildHasherDefault;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RangeCheckTestExpr {
    pub column: ColumnRef,
}

impl ProverEvaluate for RangeCheckTestExpr {
    fn first_round_evaluate(&self, _builder: &mut FirstRoundBuilder) {
        todo!()
    }

    fn result_evaluate<'a, S: Scalar>(
        &self,
        _alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        todo!()
    }

    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        _alloc: &'a Bump,
        _table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        todo!()
    }
}

impl ProofPlan for RangeCheckTestExpr {
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![]
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        todo!()
    }

    #[doc = " Return all the tables referenced in the Query"]
    fn get_table_references(&self) -> indexmap::IndexSet<TableRef, BuildHasherDefault<AHasher>> {
        todo!()
    }

    #[doc = " Count terms used within the Query\'s proof"]
    fn count(&self, _builder: &mut CountBuilder) -> Result<(), ProofError> {
        todo!()
    }

    #[doc = " Form components needed to verify and proof store into `VerificationBuilder`"]
    fn verifier_evaluate<S: Scalar>(
        &self,
        _builder: &mut VerificationBuilder<S>,
        _accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
    ) -> Result<Vec<S>, ProofError> {
        todo!()
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {

    use crate::{
        base::database::{
            owned_table_utility::{bigint, owned_table},
            ColumnRef, ColumnType, OwnedTableTestAccessor,
        },
        sql::{
            proof::VerifiableQueryResult, proof_exprs::range_check_test_expr::RangeCheckTestExpr,
        },
    };
    use blitzar::proof::InnerProductProof;

    #[test]
    #[should_panic(expected = "not yet implemented")]
    fn we_can_prove_a_range_check() {
        // let data = owned_table([bigint("a", 1000..1256)]);
        let data = owned_table([bigint("a", vec![0; 256])]);
        let t = "sxt.t".parse().unwrap();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let ast = RangeCheckTestExpr {
            column: ColumnRef::new(t, "a".parse().unwrap(), ColumnType::BigInt),
        };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&ast, &accessor, &());
        let res = verifiable_res.verify(&ast, &accessor, &()).unwrap().table;
        let expected_res = owned_table([]);
        assert_eq!(res, expected_res);
    }
}
