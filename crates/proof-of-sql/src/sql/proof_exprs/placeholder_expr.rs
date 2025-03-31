use super::ProofExpr;
use crate::{
    base::{
        database::{Column, ColumnRef, ColumnType, LiteralValue, Table},
        map::{IndexMap, IndexSet},
        proof::{PlaceholderError, PlaceholderResult, ProofError},
        scalar::Scalar,
    },
    sql::proof::{FinalRoundBuilder, VerificationBuilder},
    utils::log,
};
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable placeholder expression
///
/// This node allows us to easily represent queries like
///    select $0, $1 from T
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceholderExpr {
    pub(crate) id: usize,
    pub(crate) column_type: ColumnType,
}

impl PlaceholderExpr {
    /// Creates a new `PlaceholderExpr`
    pub fn new(id: usize, column_type: ColumnType) -> Self {
        Self { id, column_type }
    }

    /// Replace the placeholder with the correct value in `params`
    ///
    /// Note that this function will return an error if
    /// 1. The placeholder id is out of bounds
    /// 2. The placeholder type does not match the type of the value in `params`
    fn interpolate<'a>(
        &self,
        params: &'a [LiteralValue],
    ) -> Result<&'a LiteralValue, PlaceholderError> {
        if self.id >= params.len() {
            return Err(PlaceholderError::InvalidPlaceholderId {
                id: self.id,
                num_params: params.len(),
            });
        }
        let param_value = &params[self.id];
        if param_value.column_type() != self.column_type {
            return Err(PlaceholderError::InvalidPlaceholderType {
                id: self.id,
                expected: self.column_type,
                actual: params[self.id].column_type(),
            });
        }
        Ok(param_value)
    }
}

impl ProofExpr for PlaceholderExpr {
    fn data_type(&self) -> ColumnType {
        self.column_type
    }

    #[tracing::instrument(
        name = "PlaceholderExpr::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let param_value = self.interpolate(params)?;
        let res = Column::from_literal_with_length(param_value, table.num_rows(), alloc);

        log::log_memory_usage("End");

        Ok(res)
    }

    #[tracing::instrument(
        name = "PlaceholderExpr::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table: &Table<'a, S>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<Column<'a, S>> {
        log::log_memory_usage("Start");

        let param_value = self.interpolate(params)?;
        let res = Column::from_literal_with_length(param_value, table.num_rows(), alloc);

        log::log_memory_usage("End");

        Ok(res)
    }

    fn verifier_evaluate<S: Scalar>(
        &self,
        _builder: &mut impl VerificationBuilder<S>,
        _accessor: &IndexMap<ColumnRef, S>,
        chi_eval: S,
        params: &[LiteralValue],
    ) -> Result<S, ProofError> {
        let param_value = self.interpolate(params)?;
        Ok(chi_eval * param_value.to_scalar::<S>())
    }

    fn get_column_references(&self, _columns: &mut IndexSet<ColumnRef>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    // interpolate
    #[test]
    fn we_cannot_interpolate_placeholder_if_id_is_out_of_bounds() {
        // Empty params
        let placeholder_expr = PlaceholderExpr::new(0, ColumnType::Boolean);
        let params = vec![];
        let res = placeholder_expr.interpolate(&params);
        assert!(matches!(
            res,
            Err(PlaceholderError::InvalidPlaceholderId { .. })
        ));

        // Params exist but not enough of them
        let placeholder_expr = PlaceholderExpr::new(2, ColumnType::Boolean);
        let params = vec![LiteralValue::Boolean(true), LiteralValue::Boolean(false)];
        let res = placeholder_expr.interpolate(&params);
        assert!(matches!(
            res,
            Err(PlaceholderError::InvalidPlaceholderId { .. })
        ));
    }

    #[test]
    fn we_cannot_interpolate_placeholder_if_types_do_not_match() {
        let placeholder_expr = PlaceholderExpr::new(0, ColumnType::Boolean);
        let params = vec![LiteralValue::BigInt(123)];
        let res = placeholder_expr.interpolate(&params);
        assert!(matches!(
            res,
            Err(PlaceholderError::InvalidPlaceholderType { .. })
        ));
    }

    #[test]
    fn we_can_interpolate_placeholder_if_id_is_in_bounds_and_types_match() {
        let placeholder_expr = PlaceholderExpr::new(0, ColumnType::Boolean);
        let params = vec![LiteralValue::Boolean(true)];
        let res = placeholder_expr.interpolate(&params);
        assert_eq!(res.unwrap(), &LiteralValue::Boolean(true));
    }
}
