use super::{EVMProofPlanError, EVMProofPlanResult};
use crate::{
    base::{
        database::{ColumnRef, LiteralValue},
        map::IndexSet,
    },
    sql::proof_exprs::{ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr},
};
use alloc::boxed::Box;
use serde::{Deserialize, Serialize};

/// Represents an expression that can be serialized for EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum EVMDynProofExpr {
    Column(EVMColumnExpr),
    Literal(EVMLiteralExpr),
    Equals(EVMEqualsExpr),
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

        let equals_expr = EqualsExpr::new(
            Box::new(DynProofExpr::new_column(column_ref_b.clone())),
            Box::new(DynProofExpr::new_literal(LiteralValue::BigInt(5))),
        );

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

    // EVMDynProofExpr
    #[test]
    fn we_can_put_a_proof_expr_in_evm() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        let expr = equal(
            DynProofExpr::new_column(column_ref_b.clone()),
            DynProofExpr::new_literal(LiteralValue::BigInt(5)),
        );
        let evm_expr = EVMDynProofExpr::try_from_proof_expr(
            &expr,
            &indexset! {column_ref_a.clone(), column_ref_b.clone()},
        )
        .unwrap();
        let expected_evm_expr = EVMDynProofExpr::Equals(EVMEqualsExpr {
            lhs: Box::new(EVMDynProofExpr::Column(EVMColumnExpr { column_number: 1 })),
            rhs: Box::new(EVMDynProofExpr::Literal(EVMLiteralExpr::BigInt(5))),
        });
        assert_eq!(evm_expr, expected_evm_expr);

        // Roundtrip
        let roundtripped_expr = evm_expr
            .try_into_proof_expr(&indexset! {column_ref_a.clone(), column_ref_b.clone()})
            .unwrap();
        assert_eq!(roundtripped_expr, expr);
    }

    #[test]
    fn we_cannot_put_a_proof_expr_in_evm_if_not_supported() {
        let table_ref: TableRef = TableRef::try_from("namespace.table").unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        assert!(matches!(
            EVMDynProofExpr::try_from_proof_expr(
                &DynProofExpr::try_new_add(
                    DynProofExpr::new_column(column_ref_a.clone()),
                    DynProofExpr::new_column(column_ref_b.clone())
                )
                .unwrap(),
                &indexset! {column_ref_a.clone(), column_ref_b.clone()}
            ),
            Err(EVMProofPlanError::NotSupported)
        ));
    }
}
