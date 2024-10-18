use super::{DynProofExpr, ProofExpr};
use crate::base::database::ColumnTypeAssociatedData;
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, ColumnType, CommitmentAccessor, DataAccessor},
        map::IndexSet,
        proof::ProofError,
    },
    sql::proof::{CountBuilder, FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use num_traits::One;
use serde::{Deserialize, Serialize};

/// Provable logical AND expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AndExpr<C: Commitment> {
    lhs: Box<DynProofExpr<C>>,
    rhs: Box<DynProofExpr<C>>,
}

impl<C: Commitment> AndExpr<C> {
    /// Create logical AND expression
    pub fn new(lhs: Box<DynProofExpr<C>>, rhs: Box<DynProofExpr<C>>) -> Self {
        Self { lhs, rhs }
    }
}

impl<C: Commitment> ProofExpr<C> for AndExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        builder.count_subpolynomials(1);
        builder.count_intermediate_mles(1);
        builder.count_degree(3);
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean(ColumnTypeAssociatedData::NOT_NULLABLE)
    }

    #[tracing::instrument(name = "AndExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column: Column<'a, C::Scalar> =
            self.lhs.result_evaluate(table_length, alloc, accessor);
        let rhs_column: Column<'a, C::Scalar> =
            self.rhs.result_evaluate(table_length, alloc, accessor);
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        Column::Boolean(
            ColumnTypeAssociatedData::NOT_NULLABLE,
            alloc.alloc_slice_fill_with(table_length, |i| lhs[i] && rhs[i]),
        )
    }

    #[tracing::instrument(name = "AndExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column: Column<'a, C::Scalar> = self.lhs.prover_evaluate(builder, alloc, accessor);
        let rhs_column: Column<'a, C::Scalar> = self.rhs.prover_evaluate(builder, alloc, accessor);
        let lhs = lhs_column.as_boolean().expect("lhs is not boolean");
        let rhs = rhs_column.as_boolean().expect("rhs is not boolean");
        let n = lhs.len();
        assert_eq!(n, rhs.len());

        // lhs_and_rhs
        let lhs_and_rhs: &[bool] = alloc.alloc_slice_fill_with(n, |i| lhs[i] && rhs[i]);
        builder.produce_intermediate_mle(lhs_and_rhs);

        // subpolynomial: lhs_and_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (C::Scalar::one(), vec![Box::new(lhs_and_rhs)]),
                (-C::Scalar::one(), vec![Box::new(lhs), Box::new(rhs)]),
            ],
        );
        Column::Boolean(ColumnTypeAssociatedData::NOT_NULLABLE, lhs_and_rhs)
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let lhs = self.lhs.verifier_evaluate(builder, accessor)?;
        let rhs = self.rhs.verifier_evaluate(builder, accessor)?;

        // lhs_and_rhs
        let lhs_and_rhs = builder.consume_intermediate_mle();

        // subpolynomial: lhs_and_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            lhs_and_rhs - lhs * rhs,
        );

        // selection
        Ok(lhs_and_rhs)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
