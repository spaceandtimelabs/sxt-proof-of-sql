use super::range_check::{count, prover_evaluate_range_check, verifier_evaluate_range_check};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            self, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
            OwnedTable,
        },
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        CountBuilder, ProofBuilder, ProofPlan, ProverEvaluate, ResultBuilder, VerificationBuilder,
    },
};
use bumpalo::Bump;
use indexmap::{indexset, IndexSet};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RangeCheckTestExpr {
    pub column: ColumnRef,
}

impl<S: Scalar> ProverEvaluate<S> for RangeCheckTestExpr {
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder,
        _alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<database::Column<'a, S>> {
        builder.request_post_result_challenges(1);
        vec![]
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<database::Column<'a, S>> {
        let a = accessor.get_column(self.column);

        let scalar_values = alloc.alloc_slice_copy(&a.to_scalar_with_scaling(0));

        prover_evaluate_range_check(builder, scalar_values, alloc);
        vec![]
    }
}

impl<C: Commitment> ProofPlan<C> for RangeCheckTestExpr {
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        count(builder);
        Ok(())
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_length(self.column.table_ref())
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset(self.column.table_ref())
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        _accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<<C as Commitment>::Scalar>>,
    ) -> Result<Vec<<C as Commitment>::Scalar>, ProofError> {
        verifier_evaluate_range_check(builder);
        Ok(vec![])
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![]
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        indexset! {self.column}
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
    fn we_can_prove_a_range_check() {
        let data = owned_table([bigint("a", 1000..1256)]);
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
