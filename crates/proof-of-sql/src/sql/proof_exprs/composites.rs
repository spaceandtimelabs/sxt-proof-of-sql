//! Higher-level SQL constructs built by composing existing provable expressions.
//!
//! The constructions in this module produce plain [`DynProofExpr`] trees - no new
//! expression variants, no proof-system changes, and no new serialization formats.
//! This is also the only place allowed to use [`CastExpr::try_new_relabel`]: each
//! construction must argue structurally that the raw scalar encodings it relabels
//! are valid for the claimed types.
use super::{CastExpr, DynProofExpr, ProofExpr};
use crate::{
    base::database::ColumnType,
    sql::{AnalyzeError, AnalyzeResult},
};
use alloc::{boxed::Box, string::ToString, vec::Vec};

/// Relabel an expression's raw scalar encoding as `to_type` without a checked cast.
/// Only for use by constructions in this module that guarantee the encoding is valid.
fn relabel_cast(from_expr: DynProofExpr, to_type: ColumnType) -> AnalyzeResult<DynProofExpr> {
    Ok(DynProofExpr::Cast(CastExpr::try_new_relabel(
        Box::new(from_expr),
        to_type,
    )?))
}

/// Create an expression equivalent to
/// `CASE WHEN c_1 THEN v_1 ... WHEN c_n THEN v_n ELSE v_else END`
/// by composing existing provable expressions.
///
/// Each arm gets a first-match-wins guard `g_i = NOT c_1 AND ... AND NOT c_{i-1} AND c_i`
/// (the ELSE guard negates every condition). The guards are relabeled as raw scalars
/// (0 or 1), each value is relabeled as its raw scalar encoding, and the result is
/// `g_1 * v_1 + ... + g_n * v_n + g_else * v_else`, relabeled back to the branch type.
///
/// The final relabeling is sound because the guards are boolean and mutually
/// exclusive by construction, and exactly one of them is 1 for every row - so the
/// sum always equals the raw encoding of exactly one branch value, which has the
/// result type already.
///
/// Requirements:
/// - every condition must be boolean
/// - every branch value (including ELSE) must have the same type
/// - the branch type must not be `VarChar`/`VarBinary` (their scalars are hashes,
///   which cannot be materialized back into strings/bytes)
/// - there must be at least one WHEN arm
pub fn try_new_case(
    when_thens: Vec<(DynProofExpr, DynProofExpr)>,
    else_expr: DynProofExpr,
) -> AnalyzeResult<DynProofExpr> {
    let result_type = else_expr.data_type();
    if matches!(
        result_type,
        ColumnType::VarChar | ColumnType::VarBinary | ColumnType::Scalar
    ) {
        return Err(AnalyzeError::InvalidDataType {
            expr_type: result_type,
        });
    }
    for (condition, value) in &when_thens {
        if condition.data_type() != ColumnType::Boolean {
            return Err(AnalyzeError::DataTypeMismatch {
                left_type: condition.data_type().to_string(),
                right_type: ColumnType::Boolean.to_string(),
            });
        }
        if value.data_type() != result_type {
            return Err(AnalyzeError::DataTypeMismatch {
                left_type: value.data_type().to_string(),
                right_type: result_type.to_string(),
            });
        }
    }
    // Seed the accumulators with the first arm, then fold the rest in.
    // `no_prior_match` accumulates `NOT c_1 AND ... AND NOT c_i`; `masked_sum`
    // accumulates the guarded terms.
    let mut when_thens = when_thens.into_iter();
    let Some((first_condition, first_value)) = when_thens.next() else {
        // A CASE without any WHEN arm is not valid
        return Err(AnalyzeError::InvalidDataType {
            expr_type: result_type,
        });
    };
    let term = |guard: DynProofExpr, value: DynProofExpr| -> AnalyzeResult<DynProofExpr> {
        DynProofExpr::try_new_multiply(
            relabel_cast(guard, ColumnType::Scalar)?,
            relabel_cast(value, ColumnType::Scalar)?,
        )
    };
    let mut masked_sum = term(first_condition.clone(), first_value)?;
    let mut no_prior_match = DynProofExpr::try_new_not(first_condition)?;
    for (condition, value) in when_thens {
        let guard = DynProofExpr::try_new_and(no_prior_match.clone(), condition.clone())?;
        masked_sum = DynProofExpr::try_new_add(masked_sum, term(guard, value)?)?;
        no_prior_match =
            DynProofExpr::try_new_and(no_prior_match, DynProofExpr::try_new_not(condition)?)?;
    }
    let sum = DynProofExpr::try_new_add(masked_sum, term(no_prior_match, else_expr)?)?;
    relabel_cast(sum, result_type)
}
