use super::range_check::prover_evaluate_range_check;
use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnField, ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor,
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
use indexmap::IndexSet;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RangeCheckTestExpr {
    pub column: ColumnRef,
}

impl<C: Commitment> ProofPlan<C> for RangeCheckTestExpr {
    fn count(
        &self,
        builder: &mut CountBuilder,
        accessor: &dyn MetadataAccessor,
    ) -> Result<(), ProofError> {
        builder.count_intermediate_mles(62); //31 * 2

        // builder.count_subpolynomials(3);
        builder.count_degree(3);
        Ok(())
    }

    fn get_length(&self, accessor: &dyn MetadataAccessor) -> usize {
        todo!()
    }

    fn get_offset(&self, accessor: &dyn MetadataAccessor) -> usize {
        todo!()
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
        result: Option<&OwnedTable<<C as Commitment>::Scalar>>,
    ) -> Result<(), ProofError> {
        todo!()
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        todo!()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        todo!()
    }
}

impl<S: Scalar> ProverEvaluate<S> for RangeCheckTestExpr {
    /// 1st round of prover
    /// This produces the challenge alpha for the prover, however for now
    /// this happens here because ```ProofBuilder``` at current does not have
    /// the ability to produce a result challenge. TODO: add suport for this
    /// in proof builder.
    fn result_evaluate<'a>(
        &self,
        builder: &mut ResultBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        // result builder needs ability to produce intermediate MLE
        builder.request_post_result_challenges(1);
        vec![]
    }

    // second round
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Vec<Column<'a, S>> {
        let a = accessor.get_column(self.column);

        let scalar_values = alloc.alloc_slice_copy(&a.to_scalar_with_scaling(0));
        prover_evaluate_range_check(builder, scalar_values, alloc);
        vec![]
    }
}

#[cfg(all(test, feature = "blitzar"))]
mod tests {
    use crate::{
        base::{
            database::{
                owned_table_utility::{bigint, owned_table},
                ColumnRef, ColumnType, OwnedTableTestAccessor,
            },
            scalar::Curve25519Scalar,
        },
        sql::{
            proof::{
                ProofBuilder, ProverEvaluate, ResultBuilder, SumcheckMleEvaluations,
                SumcheckRandomScalars,
            },
            proof_exprs::range_check_tests::RangeCheckTestExpr,
        },
    };
    use blitzar::proof::InnerProductProof;
    use bumpalo::Bump;

    #[test]
    fn we_can_compute_correct_mles_over_decomposed_scalars() {
        let data = owned_table([bigint(
            "a",
            [1000, 1001, 1002, 1003, 1004, 1005, 1006, 1007],
        )]);
        let t = "sxt.t".parse().unwrap();
        let column = ColumnRef::new(t, "a".parse().unwrap(), ColumnType::BigInt);
        let alloc = Bump::new();
        let accessor = OwnedTableTestAccessor::<InnerProductProof>::new_from_table(t, data, 0, ());

        {
            let expr = RangeCheckTestExpr { column };

            let mut res_builder = ResultBuilder::new(8);
            let result_res = expr.result_evaluate(&mut res_builder, &alloc, &accessor);

            let mut proof_builder = ProofBuilder::new(2, 1, vec![Curve25519Scalar::from(123)]);
            let prover_res = expr.prover_evaluate(&mut proof_builder, &alloc, &accessor);

            let scalars = [Curve25519Scalar::from(123)];
            let sumcheck_random_scalars = SumcheckRandomScalars::new(&scalars, 8, 2);
            let evaluation_point = [Curve25519Scalar::from(123)];
            let sumcheck_evaluations = SumcheckMleEvaluations::new(
                8,
                &evaluation_point,
                &sumcheck_random_scalars,
                &[],
                &[],
                &Default::default(),
            );
            let one_eval = sumcheck_evaluations.one_evaluation;
        }
    }
}
