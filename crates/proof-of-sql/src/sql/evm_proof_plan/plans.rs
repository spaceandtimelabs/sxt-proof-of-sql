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
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents a plan that can be serialized for EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum Plan {
    Filter(FilterExec),
}

impl Plan {
    /// Try to create a `Plan` from a `DynProofPlan`.
    pub(crate) fn try_from_proof_plan(
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

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: &IndexSet<String>,
    ) -> Result<DynProofPlan, Error> {
        match self {
            Plan::Filter(filter_exec) => Ok(DynProofPlan::Filter(
                filter_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
        }
    }
}

/// Represents a filter execution plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct FilterExec {
    table_number: usize,
    where_clause: Expr,
    results: Vec<Expr>,
}

impl FilterExec {
    /// Try to create a `FilterExec` from a `proof_plans::FilterExec`.
    pub(crate) fn try_from_proof_plan(
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

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: &IndexSet<String>,
    ) -> Result<proof_plans::FilterExec, Error> {
        Ok(proof_plans::FilterExec::new(
            self.results
                .iter()
                .zip(output_column_names.iter())
                .map(|(expr, name)| {
                    Ok(AliasedDynProofExpr {
                        expr: expr.try_into_proof_expr(column_refs)?,
                        alias: Ident::new(name),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            TableExpr {
                table_ref: table_refs
                    .get_index(self.table_number)
                    .cloned()
                    .ok_or(Error::TableNotFound)?,
            },
            self.where_clause.try_into_proof_expr(column_refs)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            database::{ColumnType, LiteralValue},
            map::indexset,
        },
        sql::{
            evm_proof_plan::exprs::{ColumnExpr, EqualsExpr, LiteralExpr},
            proof_exprs::{self, AliasedDynProofExpr, DynProofExpr},
            proof_plans::DynProofPlan,
        },
    };

    #[test]
    fn we_can_put_filter_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let identifier_a = "a".into();
        let identifier_b = "b".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), identifier_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), identifier_b, ColumnType::BigInt);

        let filter_exec = proof_plans::FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(proof_exprs::ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias.clone()),
            }],
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(proof_exprs::EqualsExpr::new(
                Box::new(DynProofExpr::Column(proof_exprs::ColumnExpr::new(
                    column_ref_a.clone(),
                ))),
                Box::new(DynProofExpr::Literal(proof_exprs::LiteralExpr::new(
                    LiteralValue::BigInt(5),
                ))),
            )),
        );

        let evm_filter_exec = FilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        let expected_evm_filter_exec = FilterExec {
            table_number: 0,
            where_clause: Expr::Equals(EqualsExpr::new(
                Expr::Column(ColumnExpr::new(0)),
                Expr::Literal(LiteralExpr::BigInt(5)),
            )),
            results: vec![Expr::Column(ColumnExpr::new(1))],
        };

        assert_eq!(evm_filter_exec, expected_evm_filter_exec);

        // Roundtrip
        let roundtripped_filter_exec = FilterExec::try_into_proof_plan(
            &evm_filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            &indexset![alias],
        )
        .unwrap();
        assert_eq!(roundtripped_filter_exec, filter_exec);
    }

    #[test]
    fn we_cannot_put_unsupported_proof_plan_in_evm() {
        let plan = DynProofPlan::new_empty();
        let table_refs = indexset![];
        let column_refs = indexset![];
        assert!(matches!(
            Plan::try_from_proof_plan(&plan, &table_refs, &column_refs),
            Err(Error::NotSupported)
        ));
    }
}
