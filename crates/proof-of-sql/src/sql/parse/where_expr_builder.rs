use super::{ConversionError, DynProofExprBuilder};
use crate::{
    base::{
        commitment::Commitment,
        database::{ColumnRef, ColumnType},
        map::IndexMap,
    },
    sql::proof_exprs::{DynProofExpr, ProofExpr},
};
use alloc::boxed::Box;
use sqlparser::ast::{Expr, Ident};

/// Builder that enables building a `proof_of_sql::sql::proof_exprs::DynProofExpr` from a `proof_of_sql_parser::intermediate_ast::Expression` that is
/// intended to be used as the where clause in a filter expression or group by expression.
pub struct WhereExprBuilder<'a> {
    builder: DynProofExprBuilder<'a>,
}
impl<'a> WhereExprBuilder<'a> {
    /// Creates a new `WhereExprBuilder` with the given column mapping.
    pub fn new(column_mapping: &'a IndexMap<Ident, ColumnRef>) -> Self {
        Self {
            builder: DynProofExprBuilder::new(column_mapping),
        }
    }
    /// Builds a `proof_of_sql::sql::proof_exprs::DynProofExpr` from a `proof_of_sql_parser::intermediate_ast::Expression` that is
    /// intended to be used as the where clause in a filter expression or group by expression.
    pub fn build<C: Commitment>(
        self,
        where_expr: Option<Box<Expr>>,
    ) -> Result<Option<DynProofExpr<C>>, ConversionError> {
        where_expr
            .map(|where_expr| {
                let expr_plan = self.builder.build(&where_expr)?;
                // Ensure that the expression is a boolean expression
                match expr_plan.data_type() {
                    ColumnType::Boolean => Ok(expr_plan),
                    _ => Err(ConversionError::NonbooleanWhereClause {
                        datatype: expr_plan.data_type(),
                    }),
                }
            })
            .transpose()
    }
}
