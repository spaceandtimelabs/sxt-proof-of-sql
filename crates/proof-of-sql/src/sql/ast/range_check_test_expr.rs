use super::range_check::{prover_evaluate_range_check, verifier_evaluate_range_check};
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
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Serialize)]
pub struct RangeCheckTestExpr {
    pub column: ColumnRef,
}

impl<S: Scalar> ProverEvaluate<S> for RangeCheckTestExpr {
    fn result_evaluate<'a>(
        &self,
        _builder: &mut ResultBuilder<'a>,
        _alloc: &'a Bump,
        _accessor: &'a dyn DataAccessor<S>,
    ) {
    }

    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) {
        let a = accessor.get_column(self.column);

        let scalar_vector = a.clone().to_scalar_with_scaling(1);

        let scalar_values = alloc.alloc_slice_copy(&scalar_vector);

        prover_evaluate_range_check(builder, scalar_values);

        // // a * a - a = 0
        // builder.produce_sumcheck_subpolynomial(
        //     SumcheckSubpolynomialType::Identity,
        //     vec![
        //         (S::one(), vec![Box::new(a.clone()), Box::new(a.clone())]),
        //         (-S::one(), vec![Box::new(a.clone())]),
        //     ],
        // );
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
    fn get_column_references(&self) -> HashSet<ColumnRef> {
        HashSet::from([self.column])
    }

    fn count(
        &self,
        builder: &mut CountBuilder,
        _accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_anchored_mles(0);
        builder.count_intermediate_mles(32);
        builder.count_result_columns(0);
        builder.count_subpolynomials(1);
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
            [
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1,
            ],
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
