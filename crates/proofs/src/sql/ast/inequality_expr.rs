use super::{
    count_equals_zero, count_or, count_sign, prover_evaluate_equals_zero, prover_evaluate_or,
    prover_evaluate_sign, verifier_evaluate_equals_zero, verifier_evaluate_or,
    verifier_evaluate_sign, BoolExpr,
};
use crate::base::database::{Column, ColumnRef, CommitmentAccessor, DataAccessor};
use crate::base::proof::ProofError;
use crate::base::scalar::ArkScalar;
use crate::sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder};
use blitzar::compute::get_one_commit;
use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::collections::HashSet;

/// Provable AST expression for
/// ```ignore
///    <col> <= <constant>
/// ```
/// or
/// ```ignore
///    <col> >= <constant>
/// ```
#[derive(Debug, DynPartialEq, PartialEq)]
pub struct InequalityExpr {
    value: ArkScalar,
    column_ref: ColumnRef,
    is_lte: bool,
}

impl InequalityExpr {
    /// Create a new less than or equal expression
    pub fn new(column_ref: ColumnRef, value: ArkScalar, is_lte: bool) -> Self {
        Self {
            value,
            column_ref,
            is_lte,
        }
    }
}

impl BoolExpr for InequalityExpr {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        count_equals_zero(builder);
        count_sign(builder)?;
        count_or(builder);
        Ok(())
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.lte_expr.prover_evaluate",
        level = "info",
        skip_all
    )]
    fn prover_evaluate<'a>(
        &self,
        builder: &mut ProofBuilder<'a>,
        alloc: &'a Bump,
        accessor: &'a dyn DataAccessor,
    ) -> &'a [bool] {
        let table_length = builder.table_length();

        // lhs
        let lhs = if let Column::BigInt(col) = accessor.get_column(self.column_ref) {
            let lhs = alloc.alloc_slice_fill_default(table_length);
            if self.is_lte {
                lhs.par_iter_mut()
                    .zip(col)
                    .for_each(|(a, b)| *a = Into::<ArkScalar>::into(b) - self.value);
            } else {
                lhs.par_iter_mut()
                    .zip(col)
                    .for_each(|(a, b)| *a = self.value - Into::<ArkScalar>::into(b));
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
        builder: &mut VerificationBuilder,
        accessor: &dyn CommitmentAccessor,
    ) -> Result<ArkScalar, ProofError> {
        let table_length = builder.table_length();
        let generator_offset = builder.generator_offset();
        let one_commit = get_one_commit((table_length + generator_offset) as u64)
            - get_one_commit(generator_offset as u64);

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
