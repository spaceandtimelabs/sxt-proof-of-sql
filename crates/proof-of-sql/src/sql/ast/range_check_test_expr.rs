use super::range_check::{result_evaluate_range_check, verifier_evaluate_range_check};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor, OwnedTable,
        },
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{
        CountBuilder, ProofBuilder, ProofExpr, ProverEvaluate, ResultBuilder, VerificationBuilder,
    },
};
use bumpalo::Bump;
use indexmap::IndexSet;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RangeCheckTestExpr {
    pub column: ColumnRef,
}

impl<S: Scalar> ProverEvaluate<S> for RangeCheckTestExpr {
    // should be named prover round 1
    // 1st round of prover
    // This produces the challenge alpha for the prover, however for now
    // this happens here because ```ProofBuilder``` at current does not have
    // the ability to produce a result challenge. TODO: add suport for this
    // in proof builder.
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        _alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) {
        // result builder needs ability to produce intermediate MLE
        builder.request_post_result_challenges(1);
    }

    // second round
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) {
        let a = accessor.get_column(self.column);

        // If this is changed in place correctly, then we should be able to
        // pass it on without having to invert it again.
        let scalar_values = alloc.alloc_slice_copy(&a.to_scalar_with_scaling(0));
        result_evaluate_range_check(builder, scalar_values, alloc);

        // prover_evaluate_range_check(builder, scalar_values);
    }
}

impl<C: Commitment> ProofExpr<C> for RangeCheckTestExpr {
    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_length(self.column.table_ref())
    }
    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        accessor.get_offset(self.column.table_ref())
    }
    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        vec![]
    }
    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        IndexSet::from([self.column])
    }

    // refactor this to the range check and call that
    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_intermediate_mles(31);
        builder.count_subpolynomials(0);
        builder.count_degree(2);
        Ok(())
    }
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<(), ProofError> {
        let a_eval = builder.consume_anchored_mle(accessor.get_commitment(self.column));

        verifier_evaluate_range_check(builder, a_eval)?;
        Ok(())
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use crate::{
        base::database::{
            owned_table_utility::{bigint, owned_table},
            ColumnRef, ColumnType, OwnedTableTestAccessor,
        },
        sql::{ast::range_check_test_expr::RangeCheckTestExpr, proof::VerifiableQueryResult},
    };
    use blitzar::proof::InnerProductProof;

    #[should_panic]
    #[test]
    fn we_can_verify_that_every_value_in_colum_is_binary() {
        let data = owned_table([bigint(
            "a",
            [1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007],
        )]);
        let t = "sxt.t".parse().unwrap();
        let column = ColumnRef::new(t, "a".parse().unwrap(), ColumnType::BigInt);
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let expr = RangeCheckTestExpr { column };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
        let res = verifiable_res.verify(&expr, &accessor, &()).unwrap().table;
        let expected_res = owned_table([]);
        assert_eq!(res, expected_res);
    }

    #[should_panic]
    #[test]
    fn we_cannot_verify_an_invalid_that_every_value_in_colum_is_binary() {
        let data = owned_table([bigint("a", [1, 0, 1, 3, 1])]);
        let t = "sxt.t".parse().unwrap();
        let column = ColumnRef::new(t, "a".parse().unwrap(), ColumnType::BigInt);
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let expr = RangeCheckTestExpr { column };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
        assert!(verifiable_res.verify(&expr, &accessor, &()).is_err());
    }
}
