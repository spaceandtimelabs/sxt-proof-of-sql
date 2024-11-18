use super::{
    CountBuilder, FinalRoundBuilder, ProofPlan, ProverEvaluate, VerifiableQueryResult,
    VerificationBuilder,
};
use crate::{
    base::{
        commitment::InnerProductProof,
        database::{
            owned_table_utility::{bigint, owned_table},
            table_utility::*,
            ColumnField, ColumnRef, ColumnType, DataAccessor, OwnedTable, OwnedTableTestAccessor,
            Table, TableRef,
        },
        map::{indexset, IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FirstRoundBuilder, ProvableQueryResult, QueryData},
};
use bumpalo::Bump;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub(super) struct EmptyTestQueryExpr {
    pub(super) length: usize,
    pub(super) columns: usize,
}
impl ProverEvaluate for EmptyTestQueryExpr {
    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Table<'a, S> {
        let zeros = vec![0_i64; self.length];
        table_with_row_count(
            (1..=self.columns).map(|i| borrowed_bigint(format!("a{i}"), zeros.clone(), alloc)),
            self.length,
        )
    }
    fn first_round_evaluate(&self, _builder: &mut FirstRoundBuilder) {}
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Table<'a, S> {
        let zeros = vec![0_i64; self.length];
        let res: &[_] = alloc.alloc_slice_copy(&zeros);
        let _ = std::iter::repeat_with(|| builder.produce_intermediate_mle(res))
            .take(self.columns)
            .collect::<Vec<_>>();
        table_with_row_count(
            (1..=self.columns).map(|i| borrowed_bigint(format!("a{i}"), zeros.clone(), alloc)),
            self.length,
        )
    }
}
impl ProofPlan for EmptyTestQueryExpr {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        builder.count_intermediate_mles(self.columns);
        Ok(())
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        _accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
    ) -> Result<Vec<S>, ProofError> {
        let _ = std::iter::repeat_with(|| {
            assert_eq!(builder.consume_intermediate_mle(), S::ZERO);
        })
        .take(self.columns)
        .collect::<Vec<_>>();
        Ok(vec![S::ZERO])
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        (1..=self.columns)
            .map(|i| ColumnField::new(&(format!("a{i}").as_str().into()), ColumnType::BigInt))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {}
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        indexset! {TableRef::new("sxt.test".parse().unwrap())}
    }
}

#[test]
fn we_can_verify_queries_on_an_empty_table() {
    let expr = EmptyTestQueryExpr {
        columns: 1,
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("a1", [0_i64; 0])]),
        0,
        (),
    );
    let res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
    let QueryData {
        verification_hash: _,
        table,
    } = res.verify(&expr, &accessor, &()).unwrap();
    let expected_res = owned_table([bigint("a1", [0; 0])]);
    assert_eq!(table, expected_res);
}

#[test]
fn empty_verification_fails_if_the_result_contains_non_null_members() {
    let expr = EmptyTestQueryExpr {
        columns: 1,
        ..Default::default()
    };
    let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(
        "sxt.test".parse().unwrap(),
        owned_table([bigint("a1", [0_i64; 0])]),
        0,
        (),
    );
    let res = VerifiableQueryResult::<InnerProductProof> {
        provable_result: Some(ProvableQueryResult::default()),
        proof: None,
    };
    assert!(res.verify(&expr, &accessor, &()).is_err());
}
