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
        CountBuilder, ProofBuilder, ProofExpr, ProverEvaluate, ResultBuilder,
        SumcheckSubpolynomialType, VerificationBuilder,
    },
};
use bumpalo::Bump;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Serialize)]
pub struct DemoTestExpr {
    pub column: ColumnRef,
}

impl<C: Commitment> ProofExpr<C> for DemoTestExpr {
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
        builder.count_anchored_mles(1);
        builder.count_subpolynomials(1);
        builder.count_degree(3);
        Ok(())
    }
    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        _result: Option<&OwnedTable<C::Scalar>>,
    ) -> Result<(), ProofError> {
        let a_eval = builder.consume_anchored_mle(accessor.get_commitment(self.column));

        // a * a - a = 0
        builder.produce_sumcheck_subpolynomial_evaluation(
            &(builder.mle_evaluations.random_evaluation * (a_eval * a_eval - a_eval)),
        );
        Ok(())
    }
}

impl<S: Scalar> ProverEvaluate<S> for DemoTestExpr {
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
        _alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) {
        let a = accessor.get_column(self.column);
        builder.produce_anchored_mle(a.clone());

        // a * a - a = 0
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(a.clone()), Box::new(a.clone())]),
                (-S::one(), vec![Box::new(a.clone())]),
            ],
        );
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use crate::{
        base::database::{
            owned_table_utility::{bigint, owned_table},
            ColumnRef, ColumnType, OwnedTableTestAccessor,
        },
        sql::{ast::demo_test_expr::DemoTestExpr, proof::VerifiableQueryResult},
    };
    use blitzar::proof::InnerProductProof;

    #[test]
    fn we_can_verify_that_every_value_in_colum_is_binary() {
        let data = owned_table([bigint("a", [1, 0, 1, 0, 1])]);
        let t = "sxt.t".parse().unwrap();
        let column = ColumnRef::new(t, "a".parse().unwrap(), ColumnType::BigInt);
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let expr = DemoTestExpr { column };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
        let res = verifiable_res.verify(&expr, &accessor, &()).unwrap().table;
        let expected_res = owned_table([]);
        assert_eq!(res, expected_res);
    }
    #[test]
    fn we_cannot_verify_an_invalid_that_every_value_in_colum_is_binary() {
        let data = owned_table([bigint("a", [1, 0, 1, 3, 1])]);
        let t = "sxt.t".parse().unwrap();
        let column = ColumnRef::new(t, "a".parse().unwrap(), ColumnType::BigInt);
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());
        let expr = DemoTestExpr { column };
        let verifiable_res = VerifiableQueryResult::<InnerProductProof>::new(&expr, &accessor, &());
        assert!(verifiable_res.verify(&expr, &accessor, &()).is_err());
    }
}
