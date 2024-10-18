use super::{
    CountBuilder, FinalRoundBuilder, ProofPlan, ProverEvaluate, VerifiableQueryResult,
    VerificationBuilder,
};
use crate::{
    base::{
        commitment::{Commitment, InnerProductProof},
        database::{
            owned_table_utility::{bigint, owned_table},
            Column, ColumnField, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor,
            MetadataAccessor, OwnedTable, TestAccessor, UnimplementedTestAccessor,
        },
        map::IndexSet,
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{FirstRoundBuilder, ProvableQueryResult, QueryData},
};
use bumpalo::Bump;
use serde::Serialize;
use crate::base::database::ColumnTypeAssociatedData;

#[derive(Debug, Serialize, Default)]
pub(super) struct EmptyTestQueryExpr {
    pub(super) length: usize,
    pub(super) columns: usize,
}
impl<S: Scalar> ProverEvaluate<S> for EmptyTestQueryExpr {
    fn result_evaluate<'a>(
        &self,
        _input_length: usize,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let zeros = vec![0; self.length];
        let res: &[_] = alloc.alloc_slice_copy(&zeros);
        vec![Column::BigInt(ColumnTypeAssociatedData::NOT_NULLABLE, res); self.columns]
    }
    fn first_round_evaluate(&self, _builder: &mut FirstRoundBuilder) {}
    fn final_round_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let zeros = vec![0; self.length];
        let res: &[_] = alloc.alloc_slice_copy(&zeros);
        let _ = std::iter::repeat_with(|| builder.produce_intermediate_mle(res))
            .take(self.columns)
            .collect::<Vec<_>>();
        vec![Column::BigInt(ColumnTypeAssociatedData::NOT_NULLABLE, res); self.columns]
    }
}
impl<C: Commitment> ProofPlan<C> for EmptyTestQueryExpr {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_intermediate_mles(self.columns);
        Ok(())
    }
    fn get_length(&self, _accessor: &dyn MetadataAccessor) -> usize {
        self.length
    }
    fn get_offset(&self, _accessor: &dyn MetadataAccessor) -> usize {
        0
    }
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        _accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<<C as Commitment>::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        let _ = std::iter::repeat_with(|| {
            assert_eq!(builder.consume_intermediate_mle(), C::Scalar::ZERO);
        })
        .take(self.columns)
        .collect::<Vec<_>>();
        Ok(vec![C::Scalar::ZERO])
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        (1..=self.columns)
            .map(|i| ColumnField::new(format!("a{i}").parse().unwrap(), ColumnType::BigInt(ColumnTypeAssociatedData::NOT_NULLABLE)))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        unimplemented!("no real usage for this function yet")
    }
}

#[test]
fn we_can_verify_queries_on_an_empty_table() {
    let expr = EmptyTestQueryExpr {
        columns: 1,
        ..Default::default()
    };
    let accessor = UnimplementedTestAccessor::new_empty();
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
    let accessor = UnimplementedTestAccessor::new_empty();
    let res = VerifiableQueryResult::<InnerProductProof> {
        provable_result: Some(ProvableQueryResult::default()),
        proof: None,
    };
    assert!(res.verify(&expr, &accessor, &()).is_err());
}
