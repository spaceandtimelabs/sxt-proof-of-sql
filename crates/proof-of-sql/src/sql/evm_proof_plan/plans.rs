use super::{error::Error, exprs::Expr};
use crate::{
    base::{
        database::{ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        proof_exprs::{AliasedDynProofExpr, TableExpr},
        proof_plans::{self, DynProofPlan},
    },
};
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents a plan that can be serialized for EVM.
#[derive(Serialize, Deserialize)]
pub(super) enum Plan {
    Filter(FilterExec),
}

impl Plan {
    /// Try to create a `Plan` from a `DynProofPlan`.
    pub(super) fn try_from_proof_plan(
        plan: &DynProofPlan,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> Result<Self, Error> {
        match plan {
            DynProofPlan::Filter(filter_exec) => {
                FilterExec::try_from_proof_plan(filter_exec, table_refs, column_refs)
                    .map(Self::Filter)
            }
            _ => Err(Error::NotSupported),
        }
    }

    pub(super) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> Result<DynProofPlan, Error> {
        match self {
            Plan::Filter(filter_exec) => Ok(DynProofPlan::Filter(
                filter_exec.try_into_proof_plan(table_refs, column_refs)?,
            )),
        }
    }
}

/// Represents a filter execution plan.
#[derive(Serialize, Deserialize)]
pub(super) struct FilterExec {
    table_number: usize,
    where_clause: Expr,
    results: Vec<(Expr, Ident)>,
}

impl FilterExec {
    /// Try to create a `FilterExec` from a `proof_plans::FilterExec`.
    fn try_from_proof_plan(
        plan: &proof_plans::FilterExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> Result<Self, Error> {
        Ok(Self {
            table_number: table_refs
                .get_index_of(&plan.table.table_ref)
                .ok_or(Error::TableNotFound)?,
            results: plan
                .aliased_results
                .iter()
                .map(|result| {
                    Expr::try_from_proof_expr(&result.expr, column_refs)
                        .map(|expr| (expr, result.alias.clone()))
                })
                .collect::<Result<_, _>>()?,
            where_clause: Expr::try_from_proof_expr(&plan.where_clause, column_refs)?,
        })
    }

    fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> Result<proof_plans::FilterExec, Error> {
        Ok(proof_plans::FilterExec::new(
            self.results
                .iter()
                .map(|expr| AliasedDynProofExpr {
                    expr: expr.0.try_into_proof_expr(column_refs),
                    alias: expr.1.clone(),
                })
                .collect(),
            TableExpr {
                table_ref: table_refs
                    .get_index(self.table_number)
                    .cloned()
                    .ok_or(Error::TableNotFound)?,
            },
            self.where_clause.try_into_proof_expr(column_refs),
        ))
    }
}
