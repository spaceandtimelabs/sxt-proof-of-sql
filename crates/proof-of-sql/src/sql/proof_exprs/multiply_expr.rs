use super::{DynProofExpr, ProofExpr};
use crate::{
    base::{
        database::{try_multiply_column_types, Column, ColumnRef, ColumnType, NullableColumn, Table},
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{FinalRoundBuilder, SumcheckSubpolynomialType, VerificationBuilder},
        proof_exprs::multiply_columns,
    },
    utils::log,
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
    fn data_type(&self) -> ColumnType {
        try_multiply_column_types(self.lhs.data_type(), self.rhs.data_type())
            .expect("Failed to multiply column types")
    }

    fn result_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
    ) -> NullableColumn<'a, S> {
        let lhs_column = self.lhs.result_evaluate(alloc, table).values;
        let rhs_column = self.rhs.result_evaluate(alloc, table).values;
        let scalars = multiply_columns(&lhs_column, &rhs_column, alloc);
        let res = Column::Scalar(scalars);
        NullableColumn::new(res)
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
        table: &Table<'a, S>,
    ) -> NullableColumn<'a, S> {
        log::log_memory_usage("Start");

        let lhs_column = self.lhs.prover_evaluate(builder, alloc, table).values;
        let rhs_column = self.rhs.prover_evaluate(builder, alloc, table).values;

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
        let res = Column::Scalar(lhs_times_rhs);

        log::log_memory_usage("End");

        NullableColumn::new(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut impl VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
    ) -> Result<(S, Option<S>), ProofError> {
        let (lhs, _) = self.lhs.verifier_evaluate(builder, accessor, chi_eval)?;
        let (rhs, _) = self.rhs.verifier_evaluate(builder, accessor, chi_eval)?;

        // lhs_times_rhs
        let lhs_times_rhs = builder.try_consume_final_round_mle_evaluation()?;

        // subpolynomial: lhs_times_rhs - lhs * rhs
        builder.try_produce_sumcheck_subpolynomial_evaluation(
            SumcheckSubpolynomialType::Identity,
            lhs_times_rhs - lhs * rhs,
            2,
        )?;

        // selection
        Ok((lhs_times_rhs, None))
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
