use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_multiply_column_types, Column, ColumnRef, ColumnType, DataAccessor},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{CountBuilder, FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_exprs::multiply_columns,
    },
};
use alloc::{boxed::Box, vec};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable numerical * expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiplyExpr {
    lhs: Box<DynProofExpr>,
    rhs: Box<DynProofExpr>,
}

impl MultiplyExpr {
    /// Create numerical `*` expression
    pub fn new(lhs: Box<DynProofExpr>, rhs: Box<DynProofExpr>) -> Self {
        Self { lhs, rhs }
    }
}

impl ProofExpr for MultiplyExpr {
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

    fn result_evaluate<'a, S: Scalar>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Column<'a, S> {
        let lhs_column: Column<'a, S> = self.lhs.result_evaluate(table_length, alloc, accessor);
        let rhs_column: Column<'a, S> = self.rhs.result_evaluate(table_length, alloc, accessor);
        let scalars = multiply_columns(&lhs_column, &rhs_column, alloc);
        Column::Scalar(scalars)
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.multiply_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<S>,
    ) -> Column<'a, S> {
        let lhs_column: Column<'a, S> = self.lhs.prover_evaluate(builder, alloc, accessor);
        let rhs_column: Column<'a, S> = self.rhs.prover_evaluate(builder, alloc, accessor);

        // lhs_times_rhs
        let lhs_times_rhs: &'a [S] = multiply_columns(&lhs_column, &rhs_column, alloc);
        builder.produce_intermediate_mle(lhs_times_rhs);

        // subpolynomial: lhs_times_rhs - lhs * rhs
        builder.produce_sumcheck_subpolynomial(
            SumcheckSubpolynomialType::Identity,
            vec![
                (S::one(), vec![Box::new(lhs_times_rhs)]),
                (-S::one(), vec![Box::new(lhs_column), Box::new(rhs_column)]),
            ],
        );
        Column::Scalar(lhs_times_rhs)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
    ) -> Result<S, ProofError> {
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
