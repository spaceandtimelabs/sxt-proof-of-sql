use super::{
    count_equals_zero, count_or, count_sign, prover_evaluate_equals_zero, prover_evaluate_or,
    prover_evaluate_sign, result_evaluate_equals_zero, result_evaluate_or, result_evaluate_sign,
    verifier_evaluate_equals_zero, verifier_evaluate_or, verifier_evaluate_sign, BoolExpr,
};
use crate::{
    base::{
        commitment::Commitment,
        database::{Column, ColumnRef, CommitmentAccessor, DataAccessor},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder},
};
use bumpalo::Bump;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Provable AST expression for
/// ```ignore
///    <col> <= <constant>
/// ```
/// or
/// ```ignore
///    <col> >= <constant>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InequalityExpr<S: Scalar> {
    value: S,
    column_ref: ColumnRef,
    is_lte: bool,
}

impl<S: Scalar> InequalityExpr<S> {
    /// Create a new less than or equal expression
    pub fn new(column_ref: ColumnRef, value: S, is_lte: bool) -> Self {
        Self {
            value,
            column_ref,
            is_lte,
        }
    }
}

impl<C: Commitment> BoolExpr<C> for InequalityExpr<C::Scalar> {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        count_equals_zero(builder);
        count_sign(builder)?;
        count_or(builder);
        Ok(())
    }

    fn result_evaluate<'a>(
        &self,
        table_length: usize,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> &'a [bool] {
        // lhs
        let lhs = if let Column::BigInt(col) = accessor.get_column(self.column_ref) {
            let lhs = alloc.alloc_slice_fill_default(table_length);
            if self.is_lte {
                lhs.par_iter_mut()
                    .zip(col)
                    .for_each(|(a, b)| *a = Into::<C::Scalar>::into(b) - self.value);
            } else {
                lhs.par_iter_mut()
                    .zip(col)
                    .for_each(|(a, b)| *a = self.value - Into::<C::Scalar>::into(b));
            }
            lhs
        } else {
            panic!("invalid column type")
        };

        // lhs == 0
        let equals_zero = result_evaluate_equals_zero(table_length, alloc, lhs);

        // sign(lhs) == -1
        let sign = result_evaluate_sign(table_length, alloc, lhs);

        // (lhs == 0) || (sign(lhs) == -1)
        result_evaluate_or(table_length, alloc, equals_zero, sign)
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.lte_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a, C::Scalar>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor<C::Scalar>,
    ) -> &'a [bool] {
        let table_length = builder.table_length();

        // lhs
        let lhs = if let Column::BigInt(col) = accessor.get_column(self.column_ref) {
            let lhs = alloc.alloc_slice_fill_default(table_length);
            if self.is_lte {
                lhs.par_iter_mut()
                    .zip(col)
                    .for_each(|(a, b)| *a = Into::<C::Scalar>::into(b) - self.value);
            } else {
                lhs.par_iter_mut()
                    .zip(col)
                    .for_each(|(a, b)| *a = self.value - Into::<C::Scalar>::into(b));
            }
            lhs
        } else {
            panic!("invalid column type")
        };

        // lhs == 0
        let equals_zero = prover_evaluate_equals_zero(builder, alloc, lhs);

        // sign(lhs) == -1
        let sign = prover_evaluate_sign(builder, alloc, lhs);

        // (lhs == 0) || (sign(lhs) == -1)
        prover_evaluate_or(builder, alloc, equals_zero, sign)
    }

    fn verifier_evaluate(
        &self,
        builder: &mut VerificationBuilder<C>,
        accessor: &dyn CommitmentAccessor<C>,
    ) -> Result<C::Scalar, ProofError> {
        let one_commit = *builder.one_commit();

        // commit
        let commit = if self.is_lte {
            accessor.get_commitment(self.column_ref) - self.value * one_commit
        } else {
            self.value * one_commit - accessor.get_commitment(self.column_ref)
        };

        // lhs == 0
        let equals_zero = verifier_evaluate_equals_zero(builder, &commit);

        // sign(lhs) == -1
        let sign = verifier_evaluate_sign(builder, &commit, &one_commit)?;

        // (lhs == 0) || (sign(lhs) == -1)
        Ok(verifier_evaluate_or(builder, &equals_zero, &sign))
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        columns.insert(self.column_ref);
    }
}
