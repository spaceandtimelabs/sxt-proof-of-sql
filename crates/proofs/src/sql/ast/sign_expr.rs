use super::{is_within_acceptable_range, verify_constant_decomposition, BoolExpr};
use crate::base::bit::BitDistribution;
use crate::base::database::{Column, ColumnRef, CommitmentAccessor, DataAccessor};
use crate::base::proof::ProofError;
use crate::base::scalar::ArkScalar;
use crate::sql::proof::{CountBuilder, ProofBuilder, VerificationBuilder};

use blitzar::compute::get_one_commit;
use bumpalo::Bump;
use dyn_partial_eq::DynPartialEq;

use curve25519_dalek::ristretto::RistrettoPoint;
use num_traits::Zero;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use std::collections::HashSet;

/// Provable AST expression for the sign of a column of values.
///
/// If constructed with `value` and a column x1, ..., xn, the sign expression produces the
/// values s1, ..., sn where
///    si = 1 if xi - value < 0, si = 0 if xi - value > 0, and si = 0 or 1 if xi - value = 0
/// The sign expression can be combined with an equality expression to produce inequality
/// expressions. For example,
///    xi <= value <=> (sign(xi - value) == 1) or (xi - value == 0)
#[derive(Debug, DynPartialEq, PartialEq, Eq)]
pub struct SignExpr {
    value: ArkScalar,
    column_ref: ColumnRef,
}

impl SignExpr {
    /// Create a new sign expression
    pub fn new(column_ref: ColumnRef, value: ArkScalar) -> Self {
        Self { value, column_ref }
    }
}

impl BoolExpr for SignExpr {
    fn count(&self, builder: &mut CountBuilder) -> Result<(), ProofError> {
        let dist = builder.consume_bit_distribution()?;
        if !is_within_acceptable_range(&dist) {
            return Err(ProofError::VerificationError(
                "bit distribution outside of acceptable range",
            ));
        }
        if dist.num_varying_bits() == 0 {
            return Ok(());
        }
        panic!();
    }

    #[tracing::instrument(
        name = "proofs.sql.ast.sign_expr.prover_evaluate",
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

        // expr
        let expr = if let Column::BigInt(col) = accessor.get_column(self.column_ref) {
            let expr = alloc.alloc_slice_fill_default(table_length);
            expr.par_iter_mut()
                .zip(col)
                .for_each(|(a, b)| *a = Into::<ArkScalar>::into(b) - self.value);
            expr
        } else {
            panic!("invalid column type")
        };

        // bit_distribution
        let bit_distribution = BitDistribution::new(expr);
        builder.produce_bit_distribution(bit_distribution.clone());

        // handle the constant case
        if bit_distribution.num_varying_bits() == 0 {
            return alloc.alloc_slice_fill_copy(table_length, bit_distribution.sign_bit());
        }

        todo!();
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
        let commit = accessor.get_commitment(self.column_ref) - self.value * one_commit;

        // bit_distribution
        let bit_distribution = builder.consume_bit_distribution();

        // handle constant case
        if bit_distribution.num_varying_bits() == 0 {
            return verifier_const_evaluate(builder, &bit_distribution, &commit, &one_commit);
        }

        todo!();
    }

    fn get_column_references(&self, columns: &mut HashSet<ColumnRef>) {
        columns.insert(self.column_ref);
    }
}

fn verifier_const_evaluate(
    builder: &VerificationBuilder,
    dist: &BitDistribution,
    commit: &RistrettoPoint,
    one_commit: &RistrettoPoint,
) -> Result<ArkScalar, ProofError> {
    verify_constant_decomposition(dist, commit, one_commit)?;
    if dist.sign_bit() {
        Ok(builder.mle_evaluations.one_evaluation)
    } else {
        Ok(ArkScalar::zero())
    }
}
