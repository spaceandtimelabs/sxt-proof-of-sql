use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
            OwnedTable, TableRef,
        },
        map::IndexSet,
        proof::ProofError,
    },
    sql::proof::{
        CountBuilder, Indexes, ProofBuilder, ProofPlan, ProverEvaluate, ResultBuilder,
        VerificationBuilder,
    },
};
use bumpalo::Bump;
use proof_of_sql_parser::Identifier;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct ProofExprTestPlan<C: Commitment> {
    pub expr: DynProofExpr<C>,
    pub table: TableRef,
    pub result_name: Identifier,
}
impl<C: Commitment> ProverEvaluate<C::Scalar> for ProofExprTestPlan<C> {
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        let table_length = accessor.get_length(self.table);
        builder.set_result_indexes(Indexes::Dense(0..table_length as u64));
        vec![self.expr.result_evaluate(table_length, alloc, accessor)]
    }
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Vec<Column<'a, C::Scalar>> {
        vec![self.expr.prover_evaluate(builder, alloc, accessor)]
    }
}
impl<C: Commitment> ProofPlan<C> for ProofExprTestPlan<C> {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_result_columns(1);
        self.expr.count(builder)
    }
    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_length(self.table)
    }
    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset(self.table)
    }
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<Vec<C::Scalar>, ProofError> {
        let expected_result_eval = self.expr.verifier_evaluate(builder, accessor)?;
        let actual_result_eval = builder.consume_result_mle();
        if expected_result_eval != actual_result_eval {
            Err(ProofError::VerificationError {
                error: "expected_result_eval not same as actual_result_eval",
            })?
        }
        Ok(vec![actual_result_eval])
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![ColumnField::new(self.result_name, self.expr.data_type())]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut result = IndexSet::default();
        self.expr.get_column_references(&mut result);
        result
    }
}
