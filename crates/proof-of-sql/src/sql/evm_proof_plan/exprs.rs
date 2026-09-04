use super::{EVMProofPlanError, EVMProofPlanResult};
use crate::{
    base::{
        database::{try_cast_types, ColumnRef, ColumnType, LiteralValue, TableRef},
        map::IndexSet,
    },
    sql::proof_exprs::{
        AddExpr, AndExpr, CastExpr, ColumnExpr, DynProofExpr, EqualsExpr, InequalityExpr,
        LiteralExpr, MultiplyExpr, NotExpr, OrExpr, PlaceholderExpr, ProofExpr, ScalingCastExpr,
        SubtractExpr,
    },
};
use alloc::boxed::Box;
use serde::{Deserialize, Serialize};

/// Represents an expression that can be serialized for EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum EVMDynProofExpr {
    Column(EVMColumnExpr),
    Literal(EVMLiteralExpr),
    Equals(EVMEqualsExpr),
    Add(EVMAddExpr),
    Subtract(EVMSubtractExpr),
    Multiply(EVMMultiplyExpr),
    And(EVMAndExpr),
    Or(EVMOrExpr),
    Not(EVMNotExpr),
    Cast(EVMCastExpr),
    Inequality(EVMInequalityExpr),
    Placeholder(EVMPlaceholderExpr),
    ScalingCast(EVMScalingCastExpr),
}
impl EVMDynProofExpr {
    /// Try to create an `EVMDynProofExpr` from a `DynProofExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &DynProofExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        match expr {
            DynProofExpr::Column(column_expr) => {
                EVMColumnExpr::try_from_proof_expr(column_expr, column_refs).map(Self::Column)
            }
            DynProofExpr::Literal(literal_expr) => {
                Ok(Self::Literal(EVMLiteralExpr::from_proof_expr(literal_expr)))
            }
            DynProofExpr::Equals(equals_expr) => {
                EVMEqualsExpr::try_from_proof_expr(equals_expr, column_refs).map(Self::Equals)
            }
            DynProofExpr::Inequality(inequality_expr) => {
                EVMInequalityExpr::try_from_proof_expr(inequality_expr, column_refs)
                    .map(Self::Inequality)
            }
            DynProofExpr::Add(add_expr) => {
                EVMAddExpr::try_from_proof_expr(add_expr, column_refs).map(Self::Add)
            }
            DynProofExpr::Subtract(subtract_expr) => {
                EVMSubtractExpr::try_from_proof_expr(subtract_expr, column_refs).map(Self::Subtract)
            }
            DynProofExpr::Multiply(multiply_expr) => {
                EVMMultiplyExpr::try_from_proof_expr(multiply_expr, column_refs).map(Self::Multiply)
            }
            DynProofExpr::And(and_expr) => {
                EVMAndExpr::try_from_proof_expr(and_expr, column_refs).map(Self::And)
            }
            DynProofExpr::Or(or_expr) => {
                EVMOrExpr::try_from_proof_expr(or_expr, column_refs).map(Self::Or)
            }
            DynProofExpr::Not(not_expr) => {
                EVMNotExpr::try_from_proof_expr(not_expr, column_refs).map(Self::Not)
            }
            DynProofExpr::Cast(cast_expr) => {
                EVMCastExpr::try_from_proof_expr(cast_expr, column_refs).map(Self::Cast)
            }
            DynProofExpr::ScalingCast(scaling_cast_expr) => {
                EVMScalingCastExpr::try_from_proof_expr(scaling_cast_expr, column_refs)
                    .map(Self::ScalingCast)
            }
            DynProofExpr::Placeholder(placeholder_expr) => Ok(Self::Placeholder(
                EVMPlaceholderExpr::from_proof_expr(placeholder_expr),
            )),
        }
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<DynProofExpr> {
        match self {
            EVMDynProofExpr::Column(column_expr) => Ok(DynProofExpr::Column(
                column_expr.try_into_proof_expr(column_refs)?,
            )),
            EVMDynProofExpr::Equals(equals_expr) => Ok(DynProofExpr::Equals(
                equals_expr.try_into_proof_expr(column_refs)?,
            )),
            EVMDynProofExpr::Inequality(inequality_expr) => Ok(DynProofExpr::Inequality(
                inequality_expr.try_into_proof_expr(column_refs)?,
            )),
            EVMDynProofExpr::Literal(literal_expr) => {
                Ok(DynProofExpr::Literal(literal_expr.to_proof_expr()))
            }
            EVMDynProofExpr::Add(add_expr) => Ok(DynProofExpr::Add(
                add_expr.try_into_proof_expr(column_refs)?,
            )),
            EVMDynProofExpr::Subtract(subtract_expr) => Ok(DynProofExpr::Subtract(
                subtract_expr.try_into_proof_expr(column_refs)?,
            )),
            EVMDynProofExpr::Multiply(multiply_expr) => Ok(DynProofExpr::Multiply(
                multiply_expr.try_into_proof_expr(column_refs)?,
            )),
            EVMDynProofExpr::And(and_expr) => Ok(DynProofExpr::And(
                and_expr.try_into_proof_expr(column_refs)?,
            )),
            EVMDynProofExpr::Or(or_expr) => {
                Ok(DynProofExpr::Or(or_expr.try_into_proof_expr(column_refs)?))
            }
            EVMDynProofExpr::Not(not_expr) => Ok(DynProofExpr::Not(
                not_expr.try_into_proof_expr(column_refs)?,
            )),
            EVMDynProofExpr::Cast(cast_expr) => Ok(DynProofExpr::Cast(
                cast_expr.try_into_proof_expr(column_refs)?,
            )),
            EVMDynProofExpr::ScalingCast(scaling_cast_expr) => Ok(DynProofExpr::ScalingCast(
                scaling_cast_expr.try_into_proof_expr(column_refs)?,
            )),
            EVMDynProofExpr::Placeholder(placeholder_expr) => {
                Ok(DynProofExpr::Placeholder(placeholder_expr.to_proof_expr()))
            }
        }
    }
}

/// Represents a column expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMColumnExpr {
    column_number: usize,
}

impl EVMColumnExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(column_number: usize) -> Self {
        Self { column_number }
    }

    /// Try to create a `EVMColumnExpr` from a `ColumnExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &ColumnExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(Self {
            column_number: column_refs
                .get_index_of(expr.column_ref())
                .or_else(|| {
                    column_refs.get_index_of(&ColumnRef::new(
                        TableRef::from_names(None, ""),
                        expr.column_id(),
                        expr.data_type(),
                    ))
                })
                .ok_or(EVMProofPlanError::ColumnNotFound)?,
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<ColumnExpr> {
        Ok(ColumnExpr::new(
            column_refs
                .get_index(self.column_number)
                .ok_or(EVMProofPlanError::ColumnNotFound)?
                .clone(),
        ))
    }
}

/// Represents a literal expression that can be serialized for EVM.
///
/// This enum corresponds to the variants in `LiteralValue` that can be represented in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMLiteralExpr(pub(super) LiteralValue);

impl EVMLiteralExpr {
    /// Create a `EVMLiteralExpr` from a `LiteralExpr`.
    pub(crate) fn from_proof_expr(expr: &LiteralExpr) -> Self {
        EVMLiteralExpr(expr.value().clone())
    }

    /// Convert back to a `LiteralExpr`
    pub(crate) fn to_proof_expr(&self) -> LiteralExpr {
        LiteralExpr::new(self.0.clone())
    }
}

/// Represents an equals expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMEqualsExpr {
    lhs: Box<EVMDynProofExpr>,
    rhs: Box<EVMDynProofExpr>,
}

impl EVMEqualsExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(lhs: EVMDynProofExpr, rhs: EVMDynProofExpr) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Try to create an `EVMEqualsExpr` from a `EqualsExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &EqualsExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(EVMEqualsExpr {
            lhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.lhs(),
                column_refs,
            )?),
            rhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.rhs(),
                column_refs,
            )?),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<EqualsExpr> {
        Ok(EqualsExpr::try_new(
            Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            Box::new(self.rhs.try_into_proof_expr(column_refs)?),
        )?)
    }
}

/// Represents an inequality expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMInequalityExpr {
    lhs: Box<EVMDynProofExpr>,
    rhs: Box<EVMDynProofExpr>,
    is_lt: bool,
}

impl EVMInequalityExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(lhs: EVMDynProofExpr, rhs: EVMDynProofExpr, is_lt: bool) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
            is_lt,
        }
    }

    /// Try to create an `EVMInequalityExpr` from a `InequalityExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &InequalityExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(EVMInequalityExpr {
            lhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.lhs(),
                column_refs,
            )?),
            rhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.rhs(),
                column_refs,
            )?),
            is_lt: expr.is_lt(),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<InequalityExpr> {
        Ok(InequalityExpr::try_new(
            Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            Box::new(self.rhs.try_into_proof_expr(column_refs)?),
            self.is_lt,
        )?)
    }
}

/// Represents an addition expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMAddExpr {
    lhs: Box<EVMDynProofExpr>,
    rhs: Box<EVMDynProofExpr>,
}

impl EVMAddExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(lhs: EVMDynProofExpr, rhs: EVMDynProofExpr) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Try to create an `EVMAddExpr` from a `AddExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &AddExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(EVMAddExpr {
            lhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.lhs(),
                column_refs,
            )?),
            rhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.rhs(),
                column_refs,
            )?),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<AddExpr> {
        Ok(AddExpr::try_new(
            Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            Box::new(self.rhs.try_into_proof_expr(column_refs)?),
        )?)
    }
}

/// Represents a subtraction expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMSubtractExpr {
    lhs: Box<EVMDynProofExpr>,
    rhs: Box<EVMDynProofExpr>,
}

impl EVMSubtractExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(lhs: EVMDynProofExpr, rhs: EVMDynProofExpr) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Try to create an `EVMSubtractExpr` from a `SubtractExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &SubtractExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(EVMSubtractExpr {
            lhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.lhs(),
                column_refs,
            )?),
            rhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.rhs(),
                column_refs,
            )?),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<SubtractExpr> {
        Ok(SubtractExpr::try_new(
            Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            Box::new(self.rhs.try_into_proof_expr(column_refs)?),
        )?)
    }
}

/// Represents a multiplication expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMMultiplyExpr {
    lhs: Box<EVMDynProofExpr>,
    rhs: Box<EVMDynProofExpr>,
}

impl EVMMultiplyExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(lhs: EVMDynProofExpr, rhs: EVMDynProofExpr) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Try to create an `EVMMultiplyExpr` from a `MultiplyExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &MultiplyExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(EVMMultiplyExpr {
            lhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.lhs(),
                column_refs,
            )?),
            rhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.rhs(),
                column_refs,
            )?),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<MultiplyExpr> {
        Ok(MultiplyExpr::try_new(
            Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            Box::new(self.rhs.try_into_proof_expr(column_refs)?),
        )?)
    }
}

/// Represents an AND expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMAndExpr {
    lhs: Box<EVMDynProofExpr>,
    rhs: Box<EVMDynProofExpr>,
}

impl EVMAndExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(lhs: EVMDynProofExpr, rhs: EVMDynProofExpr) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Try to create an `EVMAndExpr` from a `AndExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &AndExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(EVMAndExpr {
            lhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.lhs(),
                column_refs,
            )?),
            rhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.rhs(),
                column_refs,
            )?),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<AndExpr> {
        Ok(AndExpr::try_new(
            Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            Box::new(self.rhs.try_into_proof_expr(column_refs)?),
        )?)
    }
}

/// Represents an OR expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMOrExpr {
    lhs: Box<EVMDynProofExpr>,
    rhs: Box<EVMDynProofExpr>,
}

impl EVMOrExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(lhs: EVMDynProofExpr, rhs: EVMDynProofExpr) -> Self {
        Self {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// Try to create an `EVMOrExpr` from a `OrExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &OrExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(EVMOrExpr {
            lhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.lhs(),
                column_refs,
            )?),
            rhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.rhs(),
                column_refs,
            )?),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<OrExpr> {
        Ok(OrExpr::try_new(
            Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            Box::new(self.rhs.try_into_proof_expr(column_refs)?),
        )?)
    }
}

/// Represents a NOT expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMNotExpr {
    expr: Box<EVMDynProofExpr>,
}

impl EVMNotExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(expr: EVMDynProofExpr) -> Self {
        Self {
            expr: Box::new(expr),
        }
    }

    /// Try to create an `EVMNotExpr` from a `NotExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &NotExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(EVMNotExpr {
            expr: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.input(),
                column_refs,
            )?),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<NotExpr> {
        Ok(NotExpr::try_new(Box::new(
            self.expr.try_into_proof_expr(column_refs)?,
        ))?)
    }
}

/// Represents a CAST expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMCastExpr {
    to_type: ColumnType,
    from_expr: Box<EVMDynProofExpr>,
}

impl EVMCastExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(from_expr: EVMDynProofExpr, to_type: ColumnType) -> Self {
        Self {
            to_type,
            from_expr: Box::new(from_expr),
        }
    }

    /// Try to create an `EVMCastExpr` from a `CastExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &CastExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        // The EVM verifier only supports checked casts; unchecked relabeling casts
        // (e.g. from the CASE lowering) must not be serialized for it.
        try_cast_types(expr.get_from_expr().data_type(), *expr.to_type())
            .map_err(|_| EVMProofPlanError::NotSupported)?;
        Ok(EVMCastExpr {
            from_expr: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.get_from_expr(),
                column_refs,
            )?),
            to_type: *expr.to_type(),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<CastExpr> {
        Ok(CastExpr::try_new(
            Box::new(self.from_expr.try_into_proof_expr(column_refs)?),
            self.to_type,
        )?)
    }
}

/// Represents a scaling CAST expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMScalingCastExpr {
    to_type: ColumnType,
    from_expr: Box<EVMDynProofExpr>,
    scaling_factor: [u64; 4],
}

impl EVMScalingCastExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(
        from_expr: EVMDynProofExpr,
        to_type: ColumnType,
        scaling_factor: [u64; 4],
    ) -> Self {
        Self {
            to_type,
            from_expr: Box::new(from_expr),
            scaling_factor,
        }
    }

    /// Try to create an `EVMScalingCastExpr` from a `ScalingCastExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &ScalingCastExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        let scaling_factor = expr.scaling_factor();
        Ok(EVMScalingCastExpr {
            from_expr: Box::new(EVMDynProofExpr::try_from_proof_expr(
                expr.get_from_expr(),
                column_refs,
            )?),
            to_type: *expr.to_type(),
            scaling_factor: [
                scaling_factor[3],
                scaling_factor[2],
                scaling_factor[1],
                scaling_factor[0],
            ],
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<ScalingCastExpr> {
        let expr = ScalingCastExpr::try_new(
            Box::new(self.from_expr.try_into_proof_expr(column_refs)?),
            self.to_type,
        )?;
        let reversed_scaling_factor = [
            self.scaling_factor[3],
            self.scaling_factor[2],
            self.scaling_factor[1],
            self.scaling_factor[0],
        ];
        (reversed_scaling_factor == expr.scaling_factor())
            .then_some(expr)
            .ok_or(EVMProofPlanError::IncorrectScalingFactor)
    }
}

/// Represents a placeholder expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMPlaceholderExpr {
    index: usize,
    column_type: ColumnType,
}

impl EVMPlaceholderExpr {
    /// Create an `EVMPlaceholderExpr` from a `PlaceholderExpr`.
    pub(crate) fn from_proof_expr(expr: &PlaceholderExpr) -> Self {
        EVMPlaceholderExpr {
            index: expr.index(),
            column_type: expr.column_type(),
        }
    }

    pub(crate) fn to_proof_expr(&self) -> PlaceholderExpr {
        PlaceholderExpr::new_from_index(self.index, self.column_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{ColumnType, TableRef},
            map::indexset,
            math::{decimal::Precision, i256::I256},
            posql_time::{PoSQLTimeUnit, PoSQLTimeZone},
            try_standard_binary_serialization,
        },
        sql::proof_exprs::test_utility::*,
    };
    use bnum::types::U256;

    // EVMColumnExpr
    #[test]
    fn we_can_put_a_column_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident = "a".into();
        let column_ref = ColumnRef::new(table_ref.clone(), ident, ColumnType::BigInt);

        let evm_column_expr = EVMColumnExpr::try_from_proof_expr(
            &ColumnExpr::new(column_ref.clone()),
            &indexset! {column_ref.clone()},
        )
        .unwrap();
        assert_eq!(evm_column_expr.column_number, 0);

        // Roundtrip
        let roundtripped_column_expr = evm_column_expr
            .try_into_proof_expr(&indexset! {column_ref.clone()})
            .unwrap();
        assert_eq!(*roundtripped_column_expr.column_ref(), column_ref);
    }

    #[test]
    fn we_cannot_put_a_column_expr_in_evm_if_column_not_found() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident = "a".into();
        let column_ref = ColumnRef::new(table_ref.clone(), ident, ColumnType::BigInt);

        assert_eq!(
            EVMColumnExpr::try_from_proof_expr(&ColumnExpr::new(column_ref.clone()), &indexset! {}),
            Err(EVMProofPlanError::ColumnNotFound)
        );
    }

    #[test]
    fn we_cannot_get_a_column_expr_from_evm_if_column_number_out_of_bounds() {
        let evm_column_expr = EVMColumnExpr { column_number: 0 };
        let column_refs = IndexSet::<ColumnRef>::default();
        assert_eq!(
            evm_column_expr
                .try_into_proof_expr(&column_refs)
                .unwrap_err(),
            EVMProofPlanError::ColumnNotFound
        );
    }

    // EVMLiteralExpr
    #[test]
    fn we_can_put_an_integer_literal_expr_in_evm() {
        // Test Uint8
        let evm_literal_expr =
            EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(LiteralValue::Uint8(42)));
        assert_eq!(evm_literal_expr, EVMLiteralExpr(LiteralValue::Uint8(42)));
        let roundtripped = evm_literal_expr.to_proof_expr();
        assert_eq!(*roundtripped.value(), LiteralValue::Uint8(42));

        // Test TinyInt
        let evm_literal_expr =
            EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(LiteralValue::TinyInt(-42)));
        assert_eq!(evm_literal_expr, EVMLiteralExpr(LiteralValue::TinyInt(-42)));
        let roundtripped = evm_literal_expr.to_proof_expr();
        assert_eq!(*roundtripped.value(), LiteralValue::TinyInt(-42));

        // Test SmallInt
        let evm_literal_expr =
            EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(LiteralValue::SmallInt(1234)));
        assert_eq!(
            evm_literal_expr,
            EVMLiteralExpr(LiteralValue::SmallInt(1234))
        );
        let roundtripped = evm_literal_expr.to_proof_expr();
        assert_eq!(*roundtripped.value(), LiteralValue::SmallInt(1234));

        // Test Int
        let evm_literal_expr =
            EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(LiteralValue::Int(-12345)));
        assert_eq!(evm_literal_expr, EVMLiteralExpr(LiteralValue::Int(-12345)));
        let roundtripped = evm_literal_expr.to_proof_expr();
        assert_eq!(*roundtripped.value(), LiteralValue::Int(-12345));

        // Test BigInt
        let evm_literal_expr =
            EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(LiteralValue::BigInt(5)));
        assert_eq!(evm_literal_expr, EVMLiteralExpr(LiteralValue::BigInt(5)));
        let roundtripped = evm_literal_expr.to_proof_expr();
        assert_eq!(*roundtripped.value(), LiteralValue::BigInt(5));

        // Test Int128
        let evm_literal_expr = EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(
            LiteralValue::Int128(1_234_567_890_123_456_789),
        ));
        assert_eq!(
            evm_literal_expr,
            EVMLiteralExpr(LiteralValue::Int128(1_234_567_890_123_456_789))
        );
        let roundtripped = evm_literal_expr.to_proof_expr();
        assert_eq!(
            *roundtripped.value(),
            LiteralValue::Int128(1_234_567_890_123_456_789)
        );
    }

    #[test]
    fn we_can_put_a_boolean_literal_expr_in_evm() {
        let evm_literal_expr =
            EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(LiteralValue::Boolean(true)));
        assert_eq!(
            evm_literal_expr,
            EVMLiteralExpr(LiteralValue::Boolean(true))
        );

        // Roundtrip
        let roundtripped_literal_expr = evm_literal_expr.to_proof_expr();
        assert_eq!(
            *roundtripped_literal_expr.value(),
            LiteralValue::Boolean(true)
        );
    }

    #[test]
    fn we_can_put_a_string_literal_expr_in_evm() {
        // Test VarChar
        let test_string = "Hello, SQL World!".to_string();
        let evm_literal_expr = EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(
            LiteralValue::VarChar(test_string.clone()),
        ));
        assert_eq!(
            evm_literal_expr,
            EVMLiteralExpr(LiteralValue::VarChar(test_string.clone()))
        );
        let roundtripped = evm_literal_expr.to_proof_expr();
        assert_eq!(
            *roundtripped.value(),
            LiteralValue::VarChar(test_string.clone())
        );
    }

    #[test]
    fn we_can_put_a_binary_literal_expr_in_evm() {
        // Test VarBinary
        let test_bytes = vec![0x01, 0x02, 0x03, 0xFF];
        let evm_literal_expr = EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(
            LiteralValue::VarBinary(test_bytes.clone()),
        ));
        assert_eq!(
            evm_literal_expr,
            EVMLiteralExpr(LiteralValue::VarBinary(test_bytes.clone()))
        );
        let roundtripped = evm_literal_expr.to_proof_expr();
        assert_eq!(
            *roundtripped.value(),
            LiteralValue::VarBinary(test_bytes.clone())
        );
    }

    #[test]
    fn we_can_put_a_decimal_literal_expr_in_evm() {
        // Test Decimal75
        let precision = Precision::new(10).unwrap();
        let scale: i8 = 2;
        let value = I256::from(12345i32); // 123.45 with scale 2

        let evm_literal_expr = EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(
            LiteralValue::Decimal75(precision, scale, value),
        ));

        if let EVMLiteralExpr(LiteralValue::Decimal75(p, s, i256)) = evm_literal_expr {
            assert_eq!(p, precision);
            assert_eq!(s, scale);
            assert_eq!(i256, value);

            let roundtripped = EVMLiteralExpr(LiteralValue::Decimal75(p, s, i256)).to_proof_expr();
            if let LiteralValue::Decimal75(rp, rs, rv) = *roundtripped.value() {
                assert_eq!(rp, precision);
                assert_eq!(rs, scale);
                assert_eq!(rv.limbs(), value.limbs());
            } else {
                panic!("Expected Decimal75 value after roundtrip");
            }
        } else {
            panic!("Expected Decimal75 variant");
        }
    }

    #[test]
    fn we_can_put_a_scalar_literal_expr_in_evm() {
        // Test Scalar
        let limbs: [u64; 4] = [1, 2, 3, 4];
        let evm_literal_expr =
            EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(LiteralValue::Scalar(limbs)));
        assert_eq!(
            evm_literal_expr,
            EVMLiteralExpr(LiteralValue::Scalar(limbs))
        );
        let roundtripped = evm_literal_expr.to_proof_expr();
        assert_eq!(*roundtripped.value(), LiteralValue::Scalar(limbs));
    }

    #[test]
    fn we_can_put_a_timestamp_literal_expr_in_evm() {
        // Test TimeStampTZ
        let unit = PoSQLTimeUnit::Millisecond;
        let timezone = PoSQLTimeZone::new(3600); // UTC+1
        let value: i64 = 1_619_712_000_000; // Some timestamp

        let evm_literal_expr = EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(
            LiteralValue::TimeStampTZ(unit, timezone, value),
        ));

        if let EVMLiteralExpr(LiteralValue::TimeStampTZ(u, tz, ts)) = evm_literal_expr {
            assert_eq!(u, unit);
            assert_eq!(tz, timezone);
            assert_eq!(ts, value);

            let roundtripped = EVMLiteralExpr(LiteralValue::TimeStampTZ(u, tz, ts)).to_proof_expr();
            if let LiteralValue::TimeStampTZ(ru, rtz, rts) = *roundtripped.value() {
                assert_eq!(ru, PoSQLTimeUnit::Millisecond);
                assert_eq!(rtz, timezone);
                assert_eq!(rts, value);
            } else {
                panic!("Expected TimeStampTZ value after roundtrip");
            }
        } else {
            panic!("Expected TimeStampTZ variant");
        }

        // Test another TimeStampTZ with different unit and timezone
        let unit2 = PoSQLTimeUnit::Nanosecond;
        let timezone2 = PoSQLTimeZone::new(-7200); // UTC-2
        let value2: i64 = 1_619_712_000_000_000_000; // Some timestamp in nanoseconds

        let evm_literal_expr2 = EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(
            LiteralValue::TimeStampTZ(unit2, timezone2, value2),
        ));

        if let EVMLiteralExpr(LiteralValue::TimeStampTZ(u, tz, ts)) = evm_literal_expr2 {
            assert_eq!(u, unit2);
            assert_eq!(tz, timezone2);
            assert_eq!(ts, value2);
        } else {
            panic!("Expected TimeStampTZ variant");
        }
    }

    // EVMEqualsExpr
    #[test]
    fn we_can_put_an_equals_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        let equals_expr = EqualsExpr::try_new(
            Box::new(DynProofExpr::new_column(column_ref_b.clone())),
            Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(5))),
        )
        .unwrap();

        let evm_equals_expr = EVMEqualsExpr::try_from_proof_expr(
            &equals_expr,
            &indexset! {column_ref_a.clone(), column_ref_b.clone()},
        )
        .unwrap();
        assert_eq!(
            evm_equals_expr,
            EVMEqualsExpr {
                lhs: Box::new(EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 })),
                rhs: Box::new(EVMDynProofExpr::Literal(EVMLiteralExpr(
                    LiteralValue::BigInt(5)
                )))
            }
        );

        // Roundtrip
        let roundtripped_equals_expr = evm_equals_expr
            .try_into_proof_expr(&indexset! {column_ref_a.clone(), column_ref_b.clone()})
            .unwrap();
        assert_eq!(roundtripped_equals_expr, equals_expr);
    }

    // EVMAddExpr
    #[test]
    fn we_can_put_an_add_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        let add_expr = AddExpr::try_new(
            Box::new(DynProofExpr::new_column(column_ref_b.clone())),
            Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(5))),
        )
        .unwrap();

        let evm_add_expr = EVMAddExpr::try_from_proof_expr(
            &add_expr,
            &indexset! {column_ref_a.clone(), column_ref_b.clone()},
        )
        .unwrap();
        assert_eq!(
            evm_add_expr,
            EVMAddExpr::new(
                EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 }),
                EVMDynProofExpr::Literal(EVMLiteralExpr(LiteralValue::BigInt(5)))
            )
        );

        // Roundtrip
        let roundtripped_add_expr = evm_add_expr
            .try_into_proof_expr(&indexset! {column_ref_a.clone(), column_ref_b.clone()})
            .unwrap();
        assert_eq!(roundtripped_add_expr, add_expr);
    }

    // EVMSubtractExpr
    #[test]
    fn we_can_put_a_subtract_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        let subtract_expr = SubtractExpr::try_new(
            Box::new(DynProofExpr::new_column(column_ref_b.clone())),
            Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(5))),
        )
        .unwrap();

        let evm_subtract_expr = EVMSubtractExpr::try_from_proof_expr(
            &subtract_expr,
            &indexset! {column_ref_a.clone(), column_ref_b.clone()},
        )
        .unwrap();
        assert_eq!(
            evm_subtract_expr,
            EVMSubtractExpr::new(
                EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 }),
                EVMDynProofExpr::Literal(EVMLiteralExpr(LiteralValue::BigInt(5)))
            )
        );

        // Roundtrip
        let roundtripped_subtract_expr = evm_subtract_expr
            .try_into_proof_expr(&indexset! {column_ref_a.clone(), column_ref_b.clone()})
            .unwrap();
        assert_eq!(roundtripped_subtract_expr, subtract_expr);
    }

    // EVMMultiplyExpr
    #[test]
    fn we_can_put_a_multiply_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        // b * 10 so we see column_number = 1
        let multiply_expr = MultiplyExpr::try_new(
            Box::new(DynProofExpr::new_column(column_ref_b.clone())),
            Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(10))),
        )
        .unwrap();

        let evm_multiply_expr = EVMMultiplyExpr::try_from_proof_expr(
            &multiply_expr,
            &indexset! { column_ref_a.clone(), column_ref_b.clone() },
        )
        .unwrap();
        assert_eq!(
            evm_multiply_expr,
            EVMMultiplyExpr::new(
                EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 }),
                EVMDynProofExpr::Literal(EVMLiteralExpr(LiteralValue::BigInt(10)))
            )
        );

        // Roundtrip
        let roundtripped = evm_multiply_expr
            .try_into_proof_expr(&indexset! { column_ref_a, column_ref_b })
            .unwrap();
        assert_eq!(roundtripped, multiply_expr);
    }

    #[test]
    fn we_cannot_get_a_multiply_expr_from_evm_if_column_number_out_of_bounds() {
        let evm_column_expr = EVMMultiplyExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 }),
        );
        let column_refs = IndexSet::<ColumnRef>::default();
        assert_eq!(
            evm_column_expr
                .try_into_proof_expr(&column_refs)
                .unwrap_err(),
            EVMProofPlanError::ColumnNotFound
        );
    }

    // EVMAndExpr
    #[test]
    fn we_can_put_an_and_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_x = "x".into();
        let ident_y = "y".into();
        let column_ref_x = ColumnRef::new(table_ref.clone(), ident_x, ColumnType::Boolean);
        let column_ref_y = ColumnRef::new(table_ref.clone(), ident_y, ColumnType::Boolean);

        let and_expr = AndExpr::try_new(
            Box::new(DynProofExpr::new_column(column_ref_x.clone())),
            Box::new(DynProofExpr::new_column(column_ref_y.clone())),
        )
        .unwrap();

        let evm_and_expr = EVMAndExpr::try_from_proof_expr(
            &and_expr,
            &indexset! { column_ref_x.clone(), column_ref_y.clone() },
        )
        .unwrap();
        assert_eq!(
            evm_and_expr,
            EVMAndExpr::new(
                EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
                EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 })
            )
        );

        // Roundtrip
        let roundtripped = evm_and_expr
            .try_into_proof_expr(&indexset! { column_ref_x, column_ref_y })
            .unwrap();
        assert_eq!(roundtripped, and_expr);
    }

    #[test]
    fn we_cannot_get_an_and_expr_from_evm_if_column_number_out_of_bounds() {
        let evm_and_expr = EVMAndExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 }),
        );
        let column_refs = IndexSet::<ColumnRef>::default();
        assert_eq!(
            evm_and_expr.try_into_proof_expr(&column_refs).unwrap_err(),
            EVMProofPlanError::ColumnNotFound
        );
    }

    // EVMOrExpr
    #[test]
    fn we_can_put_an_or_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_x = "x".into();
        let ident_y = "y".into();
        let column_ref_x = ColumnRef::new(table_ref.clone(), ident_x, ColumnType::Boolean);
        let column_ref_y = ColumnRef::new(table_ref.clone(), ident_y, ColumnType::Boolean);

        let or_expr = OrExpr::try_new(
            Box::new(DynProofExpr::new_column(column_ref_x.clone())),
            Box::new(DynProofExpr::new_column(column_ref_y.clone())),
        )
        .unwrap();

        let evm_or_expr = EVMOrExpr::try_from_proof_expr(
            &or_expr,
            &indexset! { column_ref_x.clone(), column_ref_y.clone() },
        )
        .unwrap();
        assert_eq!(
            evm_or_expr,
            EVMOrExpr::new(
                EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
                EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 })
            )
        );

        // Roundtrip
        let roundtripped = evm_or_expr
            .try_into_proof_expr(&indexset! { column_ref_x, column_ref_y })
            .unwrap();
        assert_eq!(roundtripped, or_expr);
    }

    #[test]
    fn we_cannot_get_an_or_expr_from_evm_if_column_number_out_of_bounds() {
        let evm_or_expr = EVMOrExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 }),
        );
        let column_refs = IndexSet::<ColumnRef>::default();
        assert_eq!(
            evm_or_expr.try_into_proof_expr(&column_refs).unwrap_err(),
            EVMProofPlanError::ColumnNotFound
        );
    }

    // EVMNotExpr
    #[test]
    fn we_can_put_a_not_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_flag = "flag".into();
        let column_ref_flag = ColumnRef::new(table_ref.clone(), ident_flag, ColumnType::Boolean);

        let not_expr =
            NotExpr::try_new(Box::new(DynProofExpr::new_column(column_ref_flag.clone()))).unwrap();

        let evm_not_expr =
            EVMNotExpr::try_from_proof_expr(&not_expr, &indexset! { column_ref_flag.clone() })
                .unwrap();
        assert_eq!(
            evm_not_expr,
            EVMNotExpr::new(EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }))
        );

        // Roundtrip
        let roundtripped = evm_not_expr
            .try_into_proof_expr(&indexset! { column_ref_flag })
            .unwrap();
        assert_eq!(roundtripped, not_expr);
    }

    #[test]
    fn we_cannot_get_a_not_expr_from_evm_if_column_number_out_of_bounds() {
        let evm_not_expr =
            EVMNotExpr::new(EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }));
        let column_refs = IndexSet::<ColumnRef>::default();
        assert_eq!(
            evm_not_expr.try_into_proof_expr(&column_refs).unwrap_err(),
            EVMProofPlanError::ColumnNotFound
        );
    }

    // EVMCastExpr
    #[test]
    fn we_can_put_a_cast_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::Int);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::Int);

        let cast_expr = CastExpr::try_new(
            Box::new(DynProofExpr::new_column(column_ref_b.clone())),
            ColumnType::BigInt,
        )
        .unwrap();

        let evm_cast_expr = EVMCastExpr::try_from_proof_expr(
            &cast_expr,
            &indexset! {column_ref_a.clone(), column_ref_b.clone()},
        )
        .unwrap();
        assert_eq!(
            evm_cast_expr,
            EVMCastExpr {
                from_expr: Box::new(EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 })),
                to_type: ColumnType::BigInt,
            }
        );

        // Roundtrip
        let roundtripped_cast_expr = evm_cast_expr
            .try_into_proof_expr(&indexset! {column_ref_a.clone(), column_ref_b.clone()})
            .unwrap();
        assert_eq!(roundtripped_cast_expr, cast_expr);
    }

    #[test]
    fn we_cannot_get_a_cast_expr_from_evm_if_column_number_out_of_bounds() {
        let evm_cast_expr = EVMCastExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            ColumnType::BigInt,
        );
        let column_refs = IndexSet::<ColumnRef>::default();
        assert_eq!(
            evm_cast_expr.try_into_proof_expr(&column_refs).unwrap_err(),
            EVMProofPlanError::ColumnNotFound
        );
    }

    // EVMPlaceholderExpr
    #[test]
    fn we_can_convert_placeholderexpr_to_and_from_evm_compatible_plan() {
        let placeholder_expr = PlaceholderExpr::try_new(1, ColumnType::Boolean).unwrap();

        let evm_placeholder_expr = EVMPlaceholderExpr::from_proof_expr(&placeholder_expr);
        assert_eq!(
            evm_placeholder_expr,
            EVMPlaceholderExpr {
                index: 0, // PlaceholderExpr stores index as id - 1
                column_type: ColumnType::Boolean,
            }
        );

        // Roundtrip
        let roundtripped = evm_placeholder_expr.to_proof_expr();
        assert_eq!(roundtripped, placeholder_expr);
    }

    #[test]
    fn we_can_convert_bigint_placeholderexpr_to_and_from_evm_compatible_plan() {
        // Test with BigInt
        let placeholder_expr = PlaceholderExpr::try_new(42, ColumnType::BigInt).unwrap();
        let evm_placeholder_expr = EVMPlaceholderExpr::from_proof_expr(&placeholder_expr);
        assert_eq!(evm_placeholder_expr.index, 41); // 42 - 1
        assert_eq!(evm_placeholder_expr.column_type, ColumnType::BigInt);

        // Test roundtrip
        let roundtripped = evm_placeholder_expr.to_proof_expr();
        assert_eq!(roundtripped, placeholder_expr);
    }

    // EVMDynProofExpr with placeholder
    #[test]
    fn we_can_convert_dynproofexpr_placeholderexpr_to_and_from_evm_compatible_plan() {
        let expr = DynProofExpr::try_new_placeholder(1, ColumnType::Int).unwrap();
        let evm = EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! {}).unwrap();
        let expected = EVMDynProofExpr::Placeholder(EVMPlaceholderExpr {
            index: 0,
            column_type: ColumnType::Int,
        });
        assert_eq!(evm, expected);
        assert_eq!(evm.try_into_proof_expr(&indexset! {}).unwrap(), expr);
    }

    // EVMInequalityExpr
    #[test]
    fn we_can_put_an_inequality_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        let inequality_expr = InequalityExpr::try_new(
            Box::new(DynProofExpr::new_column(column_ref_b.clone())),
            Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(5))),
            true,
        )
        .unwrap();

        let evm_inquality_expr = EVMInequalityExpr::try_from_proof_expr(
            &inequality_expr,
            &indexset! {column_ref_a.clone(), column_ref_b.clone()},
        )
        .unwrap();
        assert_eq!(
            evm_inquality_expr,
            EVMInequalityExpr::new(
                EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 }),
                EVMDynProofExpr::Literal(EVMLiteralExpr(LiteralValue::BigInt(5))),
                true
            )
        );

        // Roundtrip
        let roundtripped_add_expr = evm_inquality_expr
            .try_into_proof_expr(&indexset! {column_ref_a.clone(), column_ref_b.clone()})
            .unwrap();
        assert_eq!(roundtripped_add_expr, inequality_expr);
    }

    // EVMScalingCastExpr
    #[test]
    fn we_can_put_a_scaling_cast_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::Int);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::Int);

        let scaling_cast_expr = ScalingCastExpr::try_new(
            Box::new(DynProofExpr::new_column(column_ref_b.clone())),
            ColumnType::Decimal75(Precision::new(15).unwrap(), 2),
        )
        .unwrap();

        let evm_scaling_cast_expr = EVMScalingCastExpr::try_from_proof_expr(
            &scaling_cast_expr,
            &indexset! {column_ref_a.clone(), column_ref_b.clone()},
        )
        .unwrap();
        assert_eq!(
            evm_scaling_cast_expr,
            EVMScalingCastExpr {
                from_expr: Box::new(EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 })),
                to_type: ColumnType::Decimal75(Precision::new(15).unwrap(), 2),
                scaling_factor: [0, 0, 0, 100],
            }
        );

        // Roundtrip
        let roundtripped_scaling_cast_expr = evm_scaling_cast_expr
            .try_into_proof_expr(&indexset! {column_ref_a.clone(), column_ref_b.clone()})
            .unwrap();
        assert_eq!(roundtripped_scaling_cast_expr, scaling_cast_expr);
    }

    #[test]
    fn we_cannot_get_a_scaling_cast_expr_from_evm_if_column_number_out_of_bounds() {
        let evm_scaling_cast_expr = EVMScalingCastExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            ColumnType::Decimal75(Precision::new(15).unwrap(), 2),
            [0, 0, 0, 100],
        );
        let column_refs = IndexSet::<ColumnRef>::default();
        assert_eq!(
            evm_scaling_cast_expr
                .try_into_proof_expr(&column_refs)
                .unwrap_err(),
            EVMProofPlanError::ColumnNotFound
        );
    }

    #[test]
    fn we_cannot_get_a_scaling_cast_expr_from_evm_if_scaling_factor_incorrect() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::Int);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::Int);

        let evm_scaling_cast_expr = EVMScalingCastExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            ColumnType::Decimal75(Precision::new(15).unwrap(), 2),
            [0, 0, 0, 1000],
        );
        assert_eq!(
            evm_scaling_cast_expr
                .try_into_proof_expr(&indexset! {column_ref_a.clone(), column_ref_b.clone()})
                .unwrap_err(),
            EVMProofPlanError::IncorrectScalingFactor
        );
    }

    // EVMDynProofExpr
    #[test]
    fn we_can_put_into_evm_a_dyn_proof_expr_equals_expr() {
        let table_ref = TableRef::try_from("namespace.table").unwrap();
        let column_b = ColumnRef::new(table_ref.clone(), "b".into(), ColumnType::BigInt);

        let expr = equal(
            DynProofExpr::new_literal(LiteralValue::BigInt(5)),
            DynProofExpr::new_column(column_b.clone()),
        );
        let evm =
            EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! { column_b.clone() }).unwrap();
        let expected = EVMDynProofExpr::Equals(EVMEqualsExpr {
            lhs: Box::new(EVMDynProofExpr::Literal(EVMLiteralExpr(
                LiteralValue::BigInt(5),
            ))),
            rhs: Box::new(EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 })),
        });
        assert_eq!(evm, expected);
        assert_eq!(
            evm.try_into_proof_expr(&indexset! { column_b }).unwrap(),
            expr
        );
    }

    #[test]
    fn we_can_put_into_evm_a_dyn_proof_expr_add_expr() {
        let table_ref = TableRef::try_from("namespace.table").unwrap();
        let column_b = ColumnRef::new(table_ref.clone(), "b".into(), ColumnType::BigInt);

        let expr = add(
            DynProofExpr::new_column(column_b.clone()),
            DynProofExpr::new_literal(LiteralValue::BigInt(3)),
        );
        let evm =
            EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! { column_b.clone() }).unwrap();
        let expected = EVMDynProofExpr::Add(EVMAddExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            EVMDynProofExpr::Literal(EVMLiteralExpr(LiteralValue::BigInt(3))),
        ));
        assert_eq!(evm, expected);
        assert_eq!(
            evm.try_into_proof_expr(&indexset! { column_b }).unwrap(),
            expr
        );
    }

    #[test]
    fn we_can_put_into_evm_a_dyn_proof_expr_subtract_expr() {
        let table_ref = TableRef::try_from("namespace.table").unwrap();
        let column_b = ColumnRef::new(table_ref.clone(), "b".into(), ColumnType::BigInt);

        let expr = subtract(
            DynProofExpr::new_column(column_b.clone()),
            DynProofExpr::new_literal(LiteralValue::BigInt(2)),
        );
        let evm =
            EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! { column_b.clone() }).unwrap();
        let expected = EVMDynProofExpr::Subtract(EVMSubtractExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            EVMDynProofExpr::Literal(EVMLiteralExpr(LiteralValue::BigInt(2))),
        ));
        assert_eq!(evm, expected);
        assert_eq!(
            evm.try_into_proof_expr(&indexset! { column_b }).unwrap(),
            expr
        );
    }

    #[test]
    fn we_can_put_into_evm_a_dyn_proof_expr_multiply_expr() {
        let table_ref = TableRef::try_from("namespace.table").unwrap();
        let column_b = ColumnRef::new(table_ref.clone(), "b".into(), ColumnType::BigInt);

        let expr = multiply(
            DynProofExpr::new_column(column_b.clone()),
            DynProofExpr::new_literal(LiteralValue::BigInt(4)),
        );
        let evm =
            EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! { column_b.clone() }).unwrap();
        let expected = EVMDynProofExpr::Multiply(EVMMultiplyExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            EVMDynProofExpr::Literal(EVMLiteralExpr(LiteralValue::BigInt(4))),
        ));
        assert_eq!(evm, expected);
        assert_eq!(
            evm.try_into_proof_expr(&indexset! { column_b }).unwrap(),
            expr
        );
    }

    #[test]
    fn we_can_put_into_evm_a_dyn_proof_expr_and_expr() {
        let table_ref = TableRef::try_from("namespace.table").unwrap();
        let c = ColumnRef::new(table_ref.clone(), "c".into(), ColumnType::Boolean);
        let d = ColumnRef::new(table_ref.clone(), "d".into(), ColumnType::Boolean);

        let expr = and(
            DynProofExpr::new_column(c.clone()),
            DynProofExpr::new_column(d.clone()),
        );
        let evm = EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! { c.clone(), d.clone() })
            .unwrap();
        let expected = EVMDynProofExpr::And(EVMAndExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 }),
        ));
        assert_eq!(evm, expected);
        assert_eq!(evm.try_into_proof_expr(&indexset! { c, d }).unwrap(), expr);
    }

    #[test]
    fn we_can_put_into_evm_a_dyn_proof_expr_or_expr() {
        let table_ref = TableRef::try_from("namespace.table").unwrap();
        let c = ColumnRef::new(table_ref.clone(), "c".into(), ColumnType::Boolean);
        let d = ColumnRef::new(table_ref.clone(), "d".into(), ColumnType::Boolean);

        let expr = or(
            DynProofExpr::new_column(c.clone()),
            DynProofExpr::new_column(d.clone()),
        );
        let evm = EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! { c.clone(), d.clone() })
            .unwrap();
        let expected = EVMDynProofExpr::Or(EVMOrExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 }),
        ));
        assert_eq!(evm, expected);
        assert_eq!(evm.try_into_proof_expr(&indexset! { c, d }).unwrap(), expr);
    }

    #[test]
    fn we_can_put_into_evm_a_dyn_proof_expr_not_expr() {
        let table_ref = TableRef::try_from("namespace.table").unwrap();
        let c = ColumnRef::new(table_ref.clone(), "c".into(), ColumnType::Boolean);

        let expr = not(DynProofExpr::new_column(c.clone()));
        let evm = EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! { c.clone() }).unwrap();
        let expected =
            EVMDynProofExpr::Not(EVMNotExpr::new(EVMDynProofExpr::Column(EVMColumnExpr {
                column_number: 0,
            })));
        assert_eq!(evm, expected);
        assert_eq!(evm.try_into_proof_expr(&indexset! { c }).unwrap(), expr);
    }

    #[test]
    fn we_can_put_into_evm_a_dyn_proof_expr_cast_expr() {
        let table_ref = TableRef::try_from("namespace.table").unwrap();
        let c = ColumnRef::new(table_ref.clone(), "c".into(), ColumnType::Int);

        let expr = cast(DynProofExpr::new_column(c.clone()), ColumnType::BigInt);
        let evm = EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! { c.clone() }).unwrap();
        let expected = EVMDynProofExpr::Cast(EVMCastExpr {
            from_expr: Box::new(EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 })),
            to_type: ColumnType::BigInt,
        });
        assert_eq!(evm, expected);
        assert_eq!(evm.try_into_proof_expr(&indexset! { c }).unwrap(), expr);
    }

    #[test]
    fn we_can_put_into_evm_a_dyn_proof_expr_inequality_expr() {
        let table_ref = TableRef::try_from("namespace.table").unwrap();
        let column_b = ColumnRef::new(table_ref.clone(), "b".into(), ColumnType::BigInt);

        let expr = lt(
            DynProofExpr::new_column(column_b.clone()),
            DynProofExpr::new_literal(LiteralValue::BigInt(4)),
        );
        let evm =
            EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! { column_b.clone() }).unwrap();
        let expected = EVMDynProofExpr::Inequality(EVMInequalityExpr::new(
            EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 }),
            EVMDynProofExpr::Literal(EVMLiteralExpr(LiteralValue::BigInt(4))),
            true,
        ));
        assert_eq!(evm, expected);
        assert_eq!(
            evm.try_into_proof_expr(&indexset! { column_b }).unwrap(),
            expr
        );
    }

    #[test]
    fn we_can_put_into_evm_a_dyn_proof_expr_scaling_cast_expr() {
        let table_ref = TableRef::try_from("namespace.table").unwrap();
        let c = ColumnRef::new(table_ref.clone(), "c".into(), ColumnType::Int);

        let expr = DynProofExpr::try_new_scaling_cast(
            DynProofExpr::new_column(c.clone()),
            ColumnType::Decimal75(Precision::new(30).unwrap(), 2),
        )
        .unwrap();
        let evm = EVMDynProofExpr::try_from_proof_expr(&expr, &indexset! { c.clone() }).unwrap();
        let expected = EVMDynProofExpr::ScalingCast(EVMScalingCastExpr {
            from_expr: Box::new(EVMDynProofExpr::Column(EVMColumnExpr { column_number: 0 })),
            to_type: ColumnType::Decimal75(Precision::new(30).unwrap(), 2),
            scaling_factor: [0, 0, 0, 100],
        });
        assert_eq!(evm, expected);
        assert_eq!(evm.try_into_proof_expr(&indexset! { c }).unwrap(), expr);
    }

    #[test]
    fn we_can_catch_evm_literal_expr_serialization_change() {
        let literal_values = vec![
            LiteralValue::Boolean(true),
            LiteralValue::TinyInt(2),
            LiteralValue::SmallInt(3),
            LiteralValue::Int(4),
            LiteralValue::BigInt(5),
            LiteralValue::VarChar("6".to_string()),
            LiteralValue::Scalar(U256::SEVEN.into()),
            LiteralValue::Decimal75(
                Precision::new(10).unwrap(),
                0,
                I256::new(U256::EIGHT.into()),
            ),
            LiteralValue::VarBinary(vec![9]),
            LiteralValue::TimeStampTZ(PoSQLTimeUnit::Millisecond, PoSQLTimeZone::utc(), 10),
        ];
        let literal_values_bytes =
            hex::encode(try_standard_binary_serialization(literal_values.clone()).unwrap());

        let evm_literal_exprs: Vec<_> = literal_values
            .into_iter()
            .map(|lv| EVMLiteralExpr::from_proof_expr(&LiteralExpr::new(lv)))
            .collect();
        let evm_literal_exprs_bytes =
            hex::encode(try_standard_binary_serialization(evm_literal_exprs).unwrap());
        assert_eq!(evm_literal_exprs_bytes, literal_values_bytes);
        assert_eq!(evm_literal_exprs_bytes,
            "000000000000000a000000000100000002020000000300030000000400000004000000050000000000000005000000070000000000000001360000000a0000000000000000000000000000000000000000000000000000000000000007000000080a0000000000000000000000000000000000000000000000000000000000000000080000000b000000000000000109000000090000000100000000000000000000000a".to_string());
    }
}
