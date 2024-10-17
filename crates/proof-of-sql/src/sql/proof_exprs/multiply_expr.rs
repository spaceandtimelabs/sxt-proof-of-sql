use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            try_multiply_column_types, Column, ColumnRef, ColumnType, CommitmentAccessor,
            DataAccessor,
        },
        map::IndexSet,
        proof::ProofError,
    },
    sql::{
        proof::{CountBuilder, FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_exprs::multiply_columns,
    },
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use num_traits::One;
use serde::{Deserialize, Serialize};
use crate::base::database::ColumnTypeAssociatedData;

/// Provable numerical * expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiplyExpr<C: Commitment> {
    lhs: Box<DynProofExpr<C>>,
    rhs: Box<DynProofExpr<C>>,
}

impl<C: Commitment> MultiplyExpr<C> {
    /// Create numerical `*` expression
    pub fn new(lhs: Box<DynProofExpr<C>>, rhs: Box<DynProofExpr<C>>) -> Self {
        Self { lhs, rhs }
    }
}

impl<C: Commitment> ProofExpr<C> for MultiplyExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        builder.count_subpolynomials(1);
        builder.count_intermediate_mles(1);
        builder.count_degree(3);
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        try_multiply_column_types(self.lhs.data_type(), self.rhs.data_type())
            .expect("Failed to multiply column types")
    }

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
        let scalars = multiply_columns(&lhs_column, &rhs_column, alloc);
        Column::Scalar(ColumnTypeAssociatedData::NOT_NULLABLE, scalars)
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.multiply_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column: Column<'a, C::Scalar> = self.lhs.prover_evaluate(builder, alloc, accessor);
        let rhs_column: Column<'a, C::Scalar> = self.rhs.prover_evaluate(builder, alloc, accessor);

        // lhs_times_rhs
        let lhs_times_rhs: &'a [C::Scalar] = multiply_columns(&lhs_column, &rhs_column, alloc);
        builder.produce_intermediate_mle(lhs_times_rhs);

        // subpolynomial: lhs_times_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (C::Scalar::one(), vec![Box::new(lhs_times_rhs)]),
                (
                    -C::Scalar::one(),
                    vec![Box::new(lhs_column), Box::new(rhs_column)],
                ),
            ],
        );
        Column::Scalar(ColumnTypeAssociatedData::NOT_NULLABLE, lhs_times_rhs)
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let lhs = self.lhs.verifier_evaluate(builder, accessor)?;
        let rhs = self.rhs.verifier_evaluate(builder, accessor)?;

        // lhs_times_rhs
        let lhs_times_rhs = builder.consume_intermediate_mle();

        // subpolynomial: lhs_times_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            lhs_times_rhs - lhs * rhs,
        );

        // selection
        Ok(lhs_times_rhs)
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
