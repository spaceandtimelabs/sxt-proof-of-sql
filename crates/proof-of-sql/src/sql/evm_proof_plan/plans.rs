use super::{error::Error, exprs::Expr};
use crate::{
    base::{
        database::{ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::proof_plans::{self, DynProofPlan},
};
use alloc::vec::Vec;
use serde::Serialize;

/// Represents a plan that can be serialized for EVM.
#[derive(Serialize)]
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
}

/// Represents a filter execution plan.
#[derive(Serialize)]
pub(super) struct FilterExec {
    table_number: usize,
    where_clause: Expr,
    results: Vec<Expr>,
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
                .map(|result| Expr::try_from_proof_expr(&result.expr, column_refs))
                .collect::<Result<_, _>>()?,
            where_clause: Expr::try_from_proof_expr(&plan.where_clause, column_refs)?,
        })
    }
}
