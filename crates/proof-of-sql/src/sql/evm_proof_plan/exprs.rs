use super::{EVMProofPlanError, EVMProofPlanResult};
use crate::{
    base::{
        database::{ColumnRef, ColumnType, LiteralValue},
        map::IndexSet,
    },
    sql::proof_exprs::{
        AddExpr, AndExpr, CastExpr, ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr,
        MultiplyExpr, NotExpr, OrExpr, SubtractExpr,
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
                EVMLiteralExpr::try_from_proof_expr(literal_expr).map(Self::Literal)
            }
            DynProofExpr::Equals(equals_expr) => {
                EVMEqualsExpr::try_from_proof_expr(equals_expr, column_refs).map(Self::Equals)
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
            _ => Err(EVMProofPlanError::NotSupported),
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
                .get_index_of(&expr.column_ref)
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

/// Represents a literal expression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum EVMLiteralExpr {
    BigInt(i64),
}
impl EVMLiteralExpr {
    #[expect(dead_code)]
    pub(crate) fn new(value: i64) -> Self {
        Self::BigInt(value)
    }

    /// Try to create a `EVMLiteralExpr` from a `LiteralExpr`.
    pub(crate) fn try_from_proof_expr(expr: &LiteralExpr) -> EVMProofPlanResult<Self> {
        match expr.value {
            LiteralValue::BigInt(value) => Ok(EVMLiteralExpr::BigInt(value)),
            _ => Err(EVMProofPlanError::NotSupported),
        }
    }

    pub(crate) fn to_proof_expr(&self) -> LiteralExpr {
        match self {
            EVMLiteralExpr::BigInt(value) => LiteralExpr::new(LiteralValue::BigInt(*value)),
        }
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
                &expr.lhs,
                column_refs,
            )?),
            rhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                &expr.rhs,
                column_refs,
            )?),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<EqualsExpr> {
        Ok(EqualsExpr {
            lhs: Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            rhs: Box::new(self.rhs.try_into_proof_expr(column_refs)?),
        })
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
                &expr.lhs,
                column_refs,
            )?),
            rhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                &expr.rhs,
                column_refs,
            )?),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<AddExpr> {
        Ok(AddExpr {
            lhs: Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            rhs: Box::new(self.rhs.try_into_proof_expr(column_refs)?),
        })
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
                &expr.lhs,
                column_refs,
            )?),
            rhs: Box::new(EVMDynProofExpr::try_from_proof_expr(
                &expr.rhs,
                column_refs,
            )?),
        })
    }

    pub(crate) fn try_into_proof_expr(
        &self,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<SubtractExpr> {
        Ok(SubtractExpr {
            lhs: Box::new(self.lhs.try_into_proof_expr(column_refs)?),
            rhs: Box::new(self.rhs.try_into_proof_expr(column_refs)?),
        })
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
    from_expr: Box<EVMDynProofExpr>,
    to_type: ColumnType,
}

impl EVMCastExpr {
    #[cfg_attr(not(test), expect(dead_code))]
    pub(crate) fn new(from_expr: EVMDynProofExpr, to_type: ColumnType) -> Self {
        Self {
            from_expr: Box::new(from_expr),
            to_type,
        }
    }

    /// Try to create an `EVMCastExpr` from a `CastExpr`.
    pub(crate) fn try_from_proof_expr(
        expr: &CastExpr,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{ColumnType, TableRef},
            map::indexset,
        },
        sql::proof_exprs::test_utility::*,
    };

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
        assert_eq!(roundtripped_column_expr.column_ref, column_ref);
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
    fn we_can_put_a_literal_expr_in_evm() {
        let evm_literal_expr =
            EVMLiteralExpr::try_from_proof_expr(&LiteralExpr::new(LiteralValue::BigInt(5)))
                .unwrap();
        assert_eq!(evm_literal_expr, EVMLiteralExpr::BigInt(5));

        // Roundtrip
        let roundtripped_literal_expr = evm_literal_expr.to_proof_expr();
        assert_eq!(roundtripped_literal_expr.value, LiteralValue::BigInt(5));
    }

    #[test]
    fn we_cannot_put_a_literal_expr_in_evm_if_not_supported() {
        assert!(matches!(
            EVMLiteralExpr::try_from_proof_expr(&LiteralExpr::new(LiteralValue::Boolean(true))),
            Err(EVMProofPlanError::NotSupported)
        ));
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
                rhs: Box::new(EVMDynProofExpr::Literal(EVMLiteralExpr::BigInt(5)))
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
                EVMDynProofExpr::Literal(EVMLiteralExpr::BigInt(5))
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
                EVMDynProofExpr::Literal(EVMLiteralExpr::BigInt(5))
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
                EVMDynProofExpr::Literal(EVMLiteralExpr::BigInt(10))
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
            lhs: Box::new(EVMDynProofExpr::Literal(EVMLiteralExpr::BigInt(5))),
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
            EVMDynProofExpr::Literal(EVMLiteralExpr::BigInt(3)),
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
            EVMDynProofExpr::Literal(EVMLiteralExpr::BigInt(2)),
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
            EVMDynProofExpr::Literal(EVMLiteralExpr::BigInt(4)),
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

    // Unsupported expressions
    #[test]
    fn we_cannot_put_a_proof_expr_in_evm_if_not_supported() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::Boolean);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::Boolean);

        assert!(matches!(
            EVMDynProofExpr::try_from_proof_expr(
                &DynProofExpr::try_new_inequality(
                    DynProofExpr::new_column(column_ref_a.clone()),
                    DynProofExpr::new_column(column_ref_b.clone()),
                    false,
                )
                .unwrap(),
                &indexset! {column_ref_a.clone(), column_ref_b.clone()}
            ),
            Err(EVMProofPlanError::NotSupported)
        ));
    }
}
