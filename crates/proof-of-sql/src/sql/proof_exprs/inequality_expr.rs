use super::{
    count_equals_zero, count_or, count_sign, prover_evaluate_equals_zero, prover_evaluate_or,
    prover_evaluate_sign, result_evaluate_equals_zero, result_evaluate_or, result_evaluate_sign,
    scale_and_add_subtract_eval, scale_and_subtract, verifier_evaluate_equals_zero,
    verifier_evaluate_or, verifier_evaluate_sign, DynProofExpr, ProofExpr,
};
use crate::{
    base::{
        commitment::Commitment,
        database::{
            Column, ColumnRef, ColumnType, ColumnTypeAssociatedData, CommitmentAccessor,
            DataAccessor,
        },
        map::IndexSet,
        proof::ProofError,
    },
    sql::proof::{CountBuilder, FinalRoundBuilder, VerificationBuilder},
};
use alloc::boxed::Box;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable AST expression for an inequality expression
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InequalityExpr<C: Commitment> {
    lhs: Box<DynProofExpr<C>>,
    rhs: Box<DynProofExpr<C>>,
    is_lte: bool,
    #[cfg(test)]
    pub(crate) treat_column_of_zeros_as_negative: bool,
}

impl<C: Commitment> InequalityExpr<C> {
    /// Create a new less than or equal expression
    pub fn new(lhs: Box<DynProofExpr<C>>, rhs: Box<DynProofExpr<C>>, is_lte: bool) -> Self {
        Self {
            lhs,
            rhs,
            is_lte,
            #[cfg(test)]
            treat_column_of_zeros_as_negative: false,
        }
    }
}

impl<C: Commitment> ProofExpr<C> for InequalityExpr<C> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        self.lhs.count(builder)?;
        self.rhs.count(builder)?;
        count_equals_zero(builder);
        count_sign(builder)?;
        count_or(builder);
        Ok(())
    }

    fn data_type(&self) -> ColumnType {
        ColumnType::Boolean(ColumnTypeAssociatedData::NOT_NULLABLE)
    }

    #[tracing::instrument(name = "InequalityExpr::result_evaluate", level = "debug", skip_all)]
    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column = self.lhs.result_evaluate(table_length, alloc, accessor);
        let rhs_column = self.rhs.result_evaluate(table_length, alloc, accessor);
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let diff = if self.is_lte {
            scale_and_subtract(alloc, lhs_column, rhs_column, lhs_scale, rhs_scale, false)
                .expect("Failed to scale and subtract")
        } else {
            scale_and_subtract(alloc, rhs_column, lhs_column, rhs_scale, lhs_scale, false)
                .expect("Failed to scale and subtract")
        };

        // diff == 0
        let equals_zero = result_evaluate_equals_zero(table_length, alloc, diff);

        // sign(diff) == -1
        let sign = result_evaluate_sign(table_length, alloc, diff);

        // (diff == 0) || (sign(diff) == -1)
        Column::Boolean(
            ColumnTypeAssociatedData::NOT_NULLABLE,
            result_evaluate_or(table_length, alloc, equals_zero, sign),
        )
    }

    #[tracing::instrument(name = "InequalityExpr::prover_evaluate", level = "debug", skip_all)]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut FinalRoundBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> Column<'a, C::Scalar> {
        let lhs_column = self.lhs.prover_evaluate(builder, alloc, accessor);
        let rhs_column = self.rhs.prover_evaluate(builder, alloc, accessor);
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let diff = if self.is_lte {
            scale_and_subtract(alloc, lhs_column, rhs_column, lhs_scale, rhs_scale, false)
                .expect("Failed to scale and subtract")
        } else {
            scale_and_subtract(alloc, rhs_column, lhs_column, rhs_scale, lhs_scale, false)
                .expect("Failed to scale and subtract")
        };

        // diff == 0
        let equals_zero = prover_evaluate_equals_zero(builder, alloc, diff);

        // sign(diff) == -1
        let sign = prover_evaluate_sign(
            builder,
            alloc,
            diff,
            #[cfg(test)]
            self.treat_column_of_zeros_as_negative,
        );

        // (diff == 0) || (sign(diff) == -1)
        Column::Boolean(
            ColumnTypeAssociatedData::NOT_NULLABLE,
            prover_evaluate_or(builder, alloc, equals_zero, sign),
        )
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let one_eval = builder.mle_evaluations.input_one_evaluation;
        let lhs_eval = self.lhs.verifier_evaluate(builder, accessor)?;
        let rhs_eval = self.rhs.verifier_evaluate(builder, accessor)?;
        let lhs_scale = self.lhs.data_type().scale().unwrap_or(0);
        let rhs_scale = self.rhs.data_type().scale().unwrap_or(0);
        let diff_eval = if self.is_lte {
            scale_and_add_subtract_eval(lhs_eval, rhs_eval, lhs_scale, rhs_scale, true)
        } else {
            scale_and_add_subtract_eval(rhs_eval, lhs_eval, rhs_scale, lhs_scale, true)
        };

        // diff == 0
        let equals_zero = verifier_evaluate_equals_zero(builder, diff_eval);

        // sign(diff) == -1
        let sign = verifier_evaluate_sign(builder, diff_eval, one_eval)?;

        // (diff == 0) || (sign(diff) == -1)
        Ok(verifier_evaluate_or(builder, &equals_zero, &sign))
    }

    fn get_column_references(&self, columns: &mut IndexSet<ColumnRef>) {
        self.lhs.get_column_references(columns);
        self.rhs.get_column_references(columns);
    }
}
