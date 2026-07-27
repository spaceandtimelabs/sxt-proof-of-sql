use super::{EVMDynProofExpr, EVMProofPlanError, EVMProofPlanResult};
use crate::{
    base::{
        database::{ColumnField, ColumnRef, TableRef},
        map::IndexSet,
    },
    sql::{
        proof_exprs::{AliasedDynProofExpr, ColumnExpr, TableExpr},
        proof_plans::{
            AggregateExec, DynProofPlan, EmptyExec, FilterExec, GroupByExec, LegacyFilterExec,
            ProjectionExec, SliceExec, SortMergeJoinExec, TableExec, UnionExec,
        },
    },
};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use core::iter;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;

/// Represents a plan that can be serialized for EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) enum EVMDynProofPlan {
    LegacyFilter(EVMLegacyFilterExec),
    Empty(EVMEmptyExec),
    Table(EVMTableExec),
    Projection(EVMProjectionExec),
    Slice(EVMSliceExec),
    GroupBy(EVMGroupByExec),
    Union(EVMUnionExec),
    SortMergeJoin(EVMSortMergeJoinExec),
    Filter(EVMFilterExec),
    Aggregate(EVMAggregateExec),
}

impl EVMDynProofPlan {
    /// Try to create a `EVMDynProofPlan` from a `DynProofPlan`.
    pub(crate) fn try_from_proof_plan(
        plan: &DynProofPlan,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        match plan {
            DynProofPlan::Empty(empty_exec) => {
                Ok(Self::Empty(EVMEmptyExec::try_from_proof_plan(empty_exec)))
            }
            DynProofPlan::Table(table_exec) => {
                EVMTableExec::try_from_proof_plan(table_exec, table_refs, column_refs)
                    .map(Self::Table)
            }
            DynProofPlan::LegacyFilter(filter_exec) => {
                EVMLegacyFilterExec::try_from_proof_plan(filter_exec, table_refs, column_refs)
                    .map(Self::LegacyFilter)
            }
            DynProofPlan::Projection(projection_exec) => {
                EVMProjectionExec::try_from_proof_plan(projection_exec, table_refs, column_refs)
                    .map(Self::Projection)
            }
            DynProofPlan::Slice(slice_exec) => {
                EVMSliceExec::try_from_proof_plan(slice_exec, table_refs, column_refs)
                    .map(Self::Slice)
            }
            DynProofPlan::GroupBy(group_by_exec) => {
                EVMGroupByExec::try_from_proof_plan(group_by_exec, table_refs, column_refs)
                    .map(Self::GroupBy)
            }
            DynProofPlan::Union(union_exec) => {
                EVMUnionExec::try_from_proof_plan(union_exec, table_refs, column_refs)
                    .map(Self::Union)
            }
            DynProofPlan::SortMergeJoin(sort_merge_join_exec) => {
                EVMSortMergeJoinExec::try_from_proof_plan(
                    sort_merge_join_exec,
                    table_refs,
                    column_refs,
                )
                .map(Self::SortMergeJoin)
            }
            DynProofPlan::Filter(filter_exec) => {
                EVMFilterExec::try_from_proof_plan(filter_exec, table_refs, column_refs)
                    .map(Self::Filter)
            }
            DynProofPlan::Aggregate(aggregate_exec) => {
                EVMAggregateExec::try_from_proof_plan(aggregate_exec, table_refs, column_refs)
                    .map(Self::Aggregate)
            }
        }
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<DynProofPlan> {
        match self {
            EVMDynProofPlan::Empty(_empty_exec) => {
                Ok(DynProofPlan::Empty(EVMEmptyExec::try_into_proof_plan()))
            }
            EVMDynProofPlan::Table(table_exec) => Ok(DynProofPlan::Table(
                table_exec.try_into_proof_plan(table_refs, column_refs)?,
            )),
            EVMDynProofPlan::LegacyFilter(filter_exec) => Ok(DynProofPlan::LegacyFilter(
                filter_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
            EVMDynProofPlan::Projection(projection_exec) => Ok(DynProofPlan::Projection(
                projection_exec.try_into_proof_plan(
                    table_refs,
                    column_refs,
                    output_column_names,
                )?,
            )),
            EVMDynProofPlan::Slice(slice_exec) => Ok(DynProofPlan::Slice(
                slice_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
            EVMDynProofPlan::GroupBy(group_by_exec) => Ok(DynProofPlan::GroupBy(
                group_by_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
            EVMDynProofPlan::Union(union_exec) => Ok(DynProofPlan::Union(
                union_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
            EVMDynProofPlan::SortMergeJoin(sort_merge_join_exec) => {
                Ok(DynProofPlan::SortMergeJoin(
                    sort_merge_join_exec.try_into_proof_plan(table_refs, column_refs)?,
                ))
            }
            EVMDynProofPlan::Filter(filter_exec) => Ok(DynProofPlan::Filter(
                filter_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
            EVMDynProofPlan::Aggregate(aggregate_exec) => Ok(DynProofPlan::Aggregate(
                aggregate_exec.try_into_proof_plan(table_refs, column_refs, output_column_names)?,
            )),
        }
    }
}

/// Represents a empty execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMEmptyExec {}

impl EVMEmptyExec {
    /// Create a `EVMEmptyExec` from a `EmptyExec`.
    pub(crate) fn try_from_proof_plan(_plan: &EmptyExec) -> Self {
        Self {}
    }

    /// Convert into a proof plan
    pub(crate) fn try_into_proof_plan() -> EmptyExec {
        EmptyExec::new()
    }
}

/// Represents a table execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMTableExec {
    table_number: usize,
    column_numbers: Vec<usize>,
}

impl EVMTableExec {
    /// Try to create a `EVMTableExec` from a `TableExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &TableExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(Self {
            table_number: table_refs
                .get_index_of(plan.table_ref())
                .ok_or(EVMProofPlanError::TableNotFound)?,
            column_numbers: column_refs
                .iter()
                .enumerate()
                .filter_map(|(i, col_ref)| (&col_ref.table_ref() == plan.table_ref()).then_some(i))
                .collect(),
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<TableExec> {
        let table_ref = table_refs
            .get_index(self.table_number)
            .cloned()
            .ok_or(EVMProofPlanError::TableNotFound)?;

        // Extract column fields for this table reference
        let schema = column_refs
            .iter()
            .filter(|col_ref| col_ref.table_ref() == table_ref.clone())
            .map(|col_ref| ColumnField::new(col_ref.column_id(), *col_ref.column_type()))
            .collect();

        Ok(TableExec::new(table_ref, schema))
    }
}

fn try_unwrap_output_column_names(
    output_column_names: Option<&IndexSet<String>>,
    length: usize,
) -> EVMProofPlanResult<IndexSet<String>> {
    let output_column_names = match output_column_names {
        Some(output_column_names) => {
            if length > output_column_names.len() {
                return Err(EVMProofPlanError::InvalidOutputColumnName);
            }
            output_column_names.clone()
        }
        None => (0..length).map(|i| i.to_string()).collect::<IndexSet<_>>(),
    };
    Ok(output_column_names)
}

fn try_unwrap_output_column_names_with_count_alias(
    output_column_names: Option<&IndexSet<String>>,
    length: usize,
    count_alias: &String,
) -> EVMProofPlanResult<IndexSet<String>> {
    let output_column_names = match output_column_names {
        Some(output_column_names) => {
            if length > output_column_names.len() {
                return Err(EVMProofPlanError::InvalidOutputColumnName);
            }
            output_column_names.clone()
        }
        None => (0..length)
            .map(|i| i.to_string())
            .filter(|name| name != count_alias)
            .take(length - 1)
            .chain(iter::once(count_alias.clone()))
            .collect::<IndexSet<_>>(),
    };
    Ok(output_column_names)
}

/// Represents a filter execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMLegacyFilterExec {
    table_number: usize,
    where_clause: EVMDynProofExpr,
    results: Vec<EVMDynProofExpr>,
}

impl EVMLegacyFilterExec {
    /// Try to create a `LegacyFilterExec` from a `proof_plans::LegacyFilterExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &LegacyFilterExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(Self {
            table_number: table_refs
                .get_index_of(&plan.table().table_ref)
                .ok_or(EVMProofPlanError::TableNotFound)?,
            results: plan
                .aliased_results()
                .iter()
                .map(|result| EVMDynProofExpr::try_from_proof_expr(&result.expr, column_refs))
                .collect::<Result<_, _>>()?,
            where_clause: EVMDynProofExpr::try_from_proof_expr(plan.where_clause(), column_refs)?,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<LegacyFilterExec> {
        let output_column_names =
            try_unwrap_output_column_names(output_column_names, self.results.len())?;
        Ok(LegacyFilterExec::new(
            self.results
                .iter()
                .zip(output_column_names.iter())
                .map(|(expr, name)| {
                    Ok(AliasedDynProofExpr {
                        expr: expr.try_into_proof_expr(column_refs)?,
                        alias: Ident::new(name),
                    })
                })
                .collect::<EVMProofPlanResult<Vec<_>>>()?,
            TableExpr {
                table_ref: table_refs
                    .get_index(self.table_number)
                    .cloned()
                    .ok_or(EVMProofPlanError::TableNotFound)?,
            },
            self.where_clause.try_into_proof_expr(column_refs)?,
        ))
    }
}

/// Represents a filter execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMFilterExec {
    input_plan: Box<EVMDynProofPlan>,
    where_clause: EVMDynProofExpr,
    results: Vec<EVMDynProofExpr>,
}

impl EVMFilterExec {
    /// Try to create a `EVMFilterExec` from a `FilterExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &FilterExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        let input_result_column_refs = plan.input().get_column_result_fields_as_references();
        Ok(Self {
            input_plan: Box::new(EVMDynProofPlan::try_from_proof_plan(
                plan.input(),
                table_refs,
                column_refs,
            )?),
            where_clause: EVMDynProofExpr::try_from_proof_expr(
                plan.where_clause(),
                &input_result_column_refs,
            )?,
            results: plan
                .aliased_results()
                .iter()
                .map(|result| {
                    EVMDynProofExpr::try_from_proof_expr(&result.expr, &input_result_column_refs)
                })
                .collect::<Result<_, _>>()?,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<FilterExec> {
        let output_column_names =
            try_unwrap_output_column_names(output_column_names, self.results.len())?;
        let input = self
            .input_plan
            .try_into_proof_plan(table_refs, column_refs, None)?;
        let input_result_column_refs = input.get_column_result_fields_as_references();
        Ok(FilterExec::new(
            self.results
                .iter()
                .zip(output_column_names.iter())
                .map(|(expr, name)| {
                    Ok(AliasedDynProofExpr {
                        expr: expr.try_into_proof_expr(&input_result_column_refs)?,
                        alias: Ident::new(name),
                    })
                })
                .collect::<EVMProofPlanResult<Vec<_>>>()?,
            Box::new(input),
            self.where_clause
                .try_into_proof_expr(&input_result_column_refs)?,
        ))
    }
}

/// Represents a projection execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMProjectionExec {
    input_plan: Box<EVMDynProofPlan>,
    results: Vec<EVMDynProofExpr>,
}

impl EVMProjectionExec {
    /// Try to create a `EVMProjectionExec` from a `ProjectionExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &ProjectionExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        let input_result_column_refs = plan.input().get_column_result_fields_as_references();
        Ok(Self {
            input_plan: Box::new(EVMDynProofPlan::try_from_proof_plan(
                plan.input(),
                table_refs,
                column_refs,
            )?),
            results: plan
                .aliased_results()
                .iter()
                .map(|result| {
                    EVMDynProofExpr::try_from_proof_expr(&result.expr, &input_result_column_refs)
                })
                .collect::<Result<_, _>>()?,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<ProjectionExec> {
        let output_column_names =
            try_unwrap_output_column_names(output_column_names, self.results.len())?;
        let input = self
            .input_plan
            .try_into_proof_plan(table_refs, column_refs, None)?;
        let input_result_column_refs = input.get_column_result_fields_as_references();
        Ok(ProjectionExec::new(
            self.results
                .iter()
                .zip(output_column_names)
                .map(|(expr, name)| {
                    Ok(AliasedDynProofExpr {
                        expr: expr.try_into_proof_expr(&input_result_column_refs)?,
                        alias: Ident::new(name),
                    })
                })
                .collect::<EVMProofPlanResult<Vec<_>>>()?,
            Box::new(input),
        ))
    }
}

/// Represents a slice execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMSliceExec {
    input_plan: Box<EVMDynProofPlan>,
    skip: usize,
    fetch: Option<usize>,
}

impl EVMSliceExec {
    /// Try to create a `EVMSliceExec` from a `SliceExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &SliceExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        Ok(Self {
            input_plan: Box::new(EVMDynProofPlan::try_from_proof_plan(
                plan.input(),
                table_refs,
                column_refs,
            )?),
            skip: plan.skip(),
            fetch: plan.fetch(),
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<SliceExec> {
        Ok(SliceExec::new(
            Box::new(self.input_plan.try_into_proof_plan(
                table_refs,
                column_refs,
                output_column_names,
            )?),
            self.skip,
            self.fetch,
        ))
    }
}

/// Represents a group by execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMGroupByExec {
    table_number: usize,
    group_by_exprs: Vec<usize>,
    where_clause: EVMDynProofExpr,
    sum_expr: Vec<EVMDynProofExpr>,
    count_alias_name: String,
}

impl EVMGroupByExec {
    /// Try to create a `EVMGroupByExec` from a `GroupByExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &GroupByExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        // Map column expressions to their indices in column_refs
        let group_by_exprs = plan
            .group_by_exprs()
            .iter()
            .map(|col_expr| {
                column_refs
                    .get_index_of(&col_expr.get_column_reference())
                    .ok_or(EVMProofPlanError::ColumnNotFound)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            table_number: table_refs
                .get_index_of(&plan.table().table_ref)
                .ok_or(EVMProofPlanError::TableNotFound)?,
            group_by_exprs: group_by_exprs.clone(),
            sum_expr: plan
                .sum_expr()
                .iter()
                .map(|aliased_expr| {
                    EVMDynProofExpr::try_from_proof_expr(&aliased_expr.expr, column_refs)
                })
                .collect::<Result<_, _>>()?,
            count_alias_name: plan.count_alias().value.clone(),
            where_clause: EVMDynProofExpr::try_from_proof_expr(plan.where_clause(), column_refs)?,
        })
    }

    #[expect(
        clippy::missing_panics_doc,
        reason = "There is a check before unwrapping"
    )]
    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<GroupByExec> {
        let grouping_column_count = self.group_by_exprs.len();
        let required_alias_count = grouping_column_count + self.sum_expr.len() + 1;
        let output_column_names =
            try_unwrap_output_column_names(output_column_names, required_alias_count)?;
        if grouping_column_count > column_refs.len() {
            Err(EVMProofPlanError::ColumnNotFound)?;
        }
        // Convert indices back to ColumnExpr objects
        let group_by_exprs = column_refs
            .iter()
            .take(grouping_column_count)
            .cloned()
            .map(ColumnExpr::new)
            .collect::<Vec<_>>();

        let mut output_column_names = output_column_names.iter().skip(grouping_column_count);

        // Map sum expressions to AliasedDynProofExpr objects
        let sum_expr = self
            .sum_expr
            .iter()
            .zip(&mut output_column_names)
            .map(
                |(expr, alias_name)| -> EVMProofPlanResult<AliasedDynProofExpr> {
                    Ok(AliasedDynProofExpr {
                        expr: expr.try_into_proof_expr(column_refs)?,
                        alias: Ident::new(alias_name),
                    })
                },
            )
            .collect::<Result<Vec<_>, _>>()?;

        // For safety, check if the provided count_alias_name matches
        if &self.count_alias_name
            != output_column_names
                .next()
                .expect("Value confirmed to exist")
        {
            Err(EVMProofPlanError::InvalidOutputColumnName)?;
        }

        GroupByExec::try_new(
            group_by_exprs,
            sum_expr,
            Ident::new(&self.count_alias_name),
            TableExpr {
                table_ref: table_refs
                    .get_index(self.table_number)
                    .cloned()
                    .ok_or(EVMProofPlanError::TableNotFound)?,
            },
            self.where_clause.try_into_proof_expr(column_refs)?,
        )
        .ok_or(EVMProofPlanError::NotSupported)
    }
}

/// Represents an aggregate execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMAggregateExec {
    input_plan: Box<EVMDynProofPlan>,
    group_by_exprs: Vec<EVMDynProofExpr>,
    where_clause: EVMDynProofExpr,
    sum_expr: Vec<EVMDynProofExpr>,
    count_alias_name: String,
}

impl EVMAggregateExec {
    /// Try to create a `EVMAggregateExec` from an `AggregateExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &AggregateExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        // Get the input result columns to use for expression conversion
        let input_result_column_refs = plan.input().get_column_result_fields_as_references();

        let group_by_exprs = plan
            .group_by_exprs()
            .iter()
            .map(|aliased_expr| {
                EVMDynProofExpr::try_from_proof_expr(&aliased_expr.expr, &input_result_column_refs)
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            input_plan: Box::new(EVMDynProofPlan::try_from_proof_plan(
                plan.input(),
                table_refs,
                column_refs,
            )?),
            group_by_exprs,
            sum_expr: plan
                .sum_expr()
                .iter()
                .map(|aliased_expr| {
                    EVMDynProofExpr::try_from_proof_expr(
                        &aliased_expr.expr,
                        &input_result_column_refs,
                    )
                })
                .collect::<Result<_, _>>()?,
            count_alias_name: plan.count_alias().value.clone(),
            where_clause: EVMDynProofExpr::try_from_proof_expr(
                plan.where_clause(),
                &input_result_column_refs,
            )?,
        })
    }

    #[expect(
        clippy::missing_panics_doc,
        reason = "There is a check before unwrapping"
    )]
    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<AggregateExec> {
        let required_alias_count = self.group_by_exprs.len() + self.sum_expr.len() + 1;
        let output_column_names = try_unwrap_output_column_names_with_count_alias(
            output_column_names,
            required_alias_count,
            &self.count_alias_name,
        )?;
        let input = self
            .input_plan
            .try_into_proof_plan(table_refs, column_refs, None)?;
        let input_result_column_refs = input.get_column_result_fields_as_references();

        let mut output_column_names = output_column_names.iter();
        // Map group by expressions to AliasedDynProofExpr objects
        let group_by_exprs = self
            .group_by_exprs
            .iter()
            .zip(&mut output_column_names)
            .map(|(expr, alias_name)| {
                Ok(AliasedDynProofExpr {
                    expr: expr.try_into_proof_expr(&input_result_column_refs)?,
                    alias: Ident::new(alias_name),
                })
            })
            .collect::<EVMProofPlanResult<Vec<_>>>()?;

        // Map sum expressions to AliasedDynProofExpr objects
        let sum_expr = self
            .sum_expr
            .iter()
            .zip(&mut output_column_names)
            .map(|(expr, alias_name)| {
                Ok(AliasedDynProofExpr {
                    expr: expr.try_into_proof_expr(&input_result_column_refs)?,
                    alias: Ident::new(alias_name),
                })
            })
            .collect::<EVMProofPlanResult<Vec<_>>>()?;

        // For safety, check if the provided count_alias_name matches
        if &self.count_alias_name
            != output_column_names
                .next()
                .expect("Value confirmed to exist")
        {
            Err(EVMProofPlanError::InvalidOutputColumnName)?;
        }

        AggregateExec::try_new(
            group_by_exprs,
            sum_expr,
            Ident::new(&self.count_alias_name),
            Box::new(input),
            self.where_clause
                .try_into_proof_expr(&input_result_column_refs)?,
        )
        .map_err(|_| EVMProofPlanError::NotSupported)
    }
}

/// Represents a union execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMUnionExec {
    pub(super) inputs: Vec<EVMDynProofPlan>,
}

impl EVMUnionExec {
    /// Try to create a `EVMUnionExec` from a `UnionExec`.
    pub(crate) fn try_from_proof_plan(
        plan: &UnionExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        // Map column expressions to their indices in column_refs
        Ok(Self {
            inputs: plan
                .input_plans()
                .iter()
                .map(|plan| EVMDynProofPlan::try_from_proof_plan(plan, table_refs, column_refs))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
        output_column_names: Option<&IndexSet<String>>,
    ) -> EVMProofPlanResult<UnionExec> {
        // We need not supply the output column names to anything other than the first input plan
        let output_column_names_collection = core::iter::once(output_column_names)
            .chain(core::iter::repeat_with(|| None))
            .take(self.inputs.len());
        Ok(UnionExec::try_new(
            self.inputs
                .iter()
                .zip(output_column_names_collection)
                .map(|(plan, output_column_names)| {
                    plan.try_into_proof_plan(table_refs, column_refs, output_column_names)
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?)
    }
}

/// Represents a group by execution plan in EVM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(crate) struct EVMSortMergeJoinExec {
    left: Box<EVMDynProofPlan>,
    right: Box<EVMDynProofPlan>,
    left_join_column_indexes: Vec<usize>,
    right_join_column_indexes: Vec<usize>,
    result_aliases: Vec<String>,
}

impl EVMSortMergeJoinExec {
    pub(crate) fn try_from_proof_plan(
        plan: &SortMergeJoinExec,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<Self> {
        let left = Box::new(EVMDynProofPlan::try_from_proof_plan(
            plan.left_plan(),
            table_refs,
            column_refs,
        )?);
        let right = Box::new(EVMDynProofPlan::try_from_proof_plan(
            plan.right_plan(),
            table_refs,
            column_refs,
        )?);
        let left_join_column_indexes = plan.left_join_column_indexes().clone();
        let right_join_column_indexes = plan.right_join_column_indexes().clone();
        let result_aliases = plan
            .result_idents()
            .iter()
            .map(|id| id.value.clone())
            .collect();

        Ok(Self {
            left,
            right,
            left_join_column_indexes,
            right_join_column_indexes,
            result_aliases,
        })
    }

    pub(crate) fn try_into_proof_plan(
        &self,
        table_refs: &IndexSet<TableRef>,
        column_refs: &IndexSet<ColumnRef>,
    ) -> EVMProofPlanResult<SortMergeJoinExec> {
        let left = Box::new(
            self.left
                .try_into_proof_plan(table_refs, column_refs, None)?,
        );
        let right = Box::new(
            self.right
                .try_into_proof_plan(table_refs, column_refs, None)?,
        );
        let left_join_column_indexes = self.left_join_column_indexes.clone();
        let right_join_column_indexes = self.right_join_column_indexes.clone();
        let result_idents = self.result_aliases.iter().map(Ident::new).collect();

        Ok(SortMergeJoinExec::new(
            left,
            right,
            left_join_column_indexes,
            right_join_column_indexes,
            result_idents,
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
            evm_proof_plan::exprs::{EVMColumnExpr, EVMEqualsExpr, EVMLiteralExpr},
            proof::ProofPlan,
            proof_exprs::{
                AddExpr, AliasedDynProofExpr, ColumnExpr, DynProofExpr, EqualsExpr, LiteralExpr,
            },
            proof_plans::{DynProofPlan, SortMergeJoinExec},
        },
    };
    use indexmap::IndexSet;

    #[test]
    fn we_can_put_projection_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a projection exec
        let projection_exec = ProjectionExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias.clone()),
            }],
            Box::new(DynProofPlan::Table(table_exec)),
        );

        // Convert to EVM plan
        let evm_projection_exec = EVMProjectionExec::try_from_proof_plan(
            &projection_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the structure
        assert_eq!(evm_projection_exec.results.len(), 1);
        assert!(matches!(
            evm_projection_exec.results[0],
            EVMDynProofExpr::Column(_)
        ));
        assert!(matches!(
            *evm_projection_exec.input_plan,
            EVMDynProofPlan::Table(_)
        ));

        // Roundtrip
        let roundtripped_projection_exec = EVMProjectionExec::try_into_proof_plan(
            &evm_projection_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![alias]),
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_projection_exec.aliased_results().len(), 1);
        assert!(matches!(
            roundtripped_projection_exec.aliased_results()[0].expr,
            DynProofExpr::Column(_)
        ));
        assert!(matches!(
            *roundtripped_projection_exec.input(),
            DynProofPlan::Table(_)
        ));

        assert!(matches!(
            EVMProjectionExec::try_into_proof_plan(
                &evm_projection_exec,
                &indexset![],
                &indexset![],
                Some(&indexset![]),
            )
            .unwrap_err(),
            EVMProofPlanError::InvalidOutputColumnName
        ));
    }

    #[test]
    fn we_can_put_slice_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a slice exec
        let skip = 10;
        let fetch = Some(5);
        let slice_exec = SliceExec::new(Box::new(DynProofPlan::Table(table_exec)), skip, fetch);

        // Convert to EVM plan
        let evm_slice_exec = EVMSliceExec::try_from_proof_plan(
            &slice_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the structure
        assert_eq!(evm_slice_exec.skip, skip);
        assert_eq!(evm_slice_exec.fetch, fetch);
        assert!(matches!(
            *evm_slice_exec.input_plan,
            EVMDynProofPlan::Table(_)
        ));

        // Roundtrip
        let roundtripped_slice_exec = EVMSliceExec::try_into_proof_plan(
            &evm_slice_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&IndexSet::default()),
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_slice_exec.skip(), skip);
        assert_eq!(roundtripped_slice_exec.fetch(), fetch);
        assert!(matches!(
            *roundtripped_slice_exec.input(),
            DynProofPlan::Table(_)
        ));

        let evm_dyn_slice_exec = EVMDynProofPlan::Slice(evm_slice_exec);
        let dyn_slice_exec = evm_dyn_slice_exec
            .try_into_proof_plan(
                &indexset![table_ref.clone()],
                &indexset![column_ref_a.clone(), column_ref_b.clone()],
                Some(&IndexSet::default()),
            )
            .unwrap();

        assert_eq!(dyn_slice_exec, DynProofPlan::Slice(slice_exec));
    }

    #[test]
    fn we_can_put_empty_exec_in_evm() {
        let empty_exec = EmptyExec::new();

        // Roundtrip
        let roundtripped_empty_exec = EVMEmptyExec::try_into_proof_plan();
        assert_eq!(roundtripped_empty_exec, empty_exec);
    }

    #[test]
    fn we_can_put_table_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        let column_fields = vec![
            ColumnField::new(ident_a, ColumnType::BigInt),
            ColumnField::new(ident_b, ColumnType::BigInt),
        ];

        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        let evm_table_exec = EVMTableExec::try_from_proof_plan(
            &table_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        let expected_evm_table_exec = EVMTableExec {
            table_number: 0,
            column_numbers: vec![0, 1],
        };

        assert_eq!(evm_table_exec, expected_evm_table_exec);

        // Roundtrip
        let roundtripped_table_exec = EVMTableExec::try_into_proof_plan(
            &evm_table_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        assert_eq!(
            *roundtripped_table_exec.table_ref(),
            *table_exec.table_ref()
        );
        assert_eq!(roundtripped_table_exec.schema().len(), 2);
    }

    #[test]
    fn table_exec_fails_with_table_not_found_from_proof_plan() {
        let missing_table_ref: TableRef = "namespace.missing".parse().unwrap();

        let column_fields = vec![
            ColumnField::new(Ident::new("a"), ColumnType::BigInt),
            ColumnField::new(Ident::new("b"), ColumnType::BigInt),
        ];

        let table_exec = TableExec::new(missing_table_ref, column_fields);

        let result = EVMTableExec::try_from_proof_plan(&table_exec, &indexset![], &indexset![]);

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn table_exec_fails_with_table_not_found_into_proof_plan() {
        let evm_table_exec = EVMTableExec {
            table_number: 0,
            column_numbers: Vec::new(),
        };

        // Use an empty table_refs to trigger TableNotFound
        let result = EVMTableExec::try_into_proof_plan(&evm_table_exec, &indexset![], &indexset![]);

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn we_can_put_filter_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        let filter_exec = LegacyFilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias.clone()),
            }],
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        );

        let evm_filter_exec = EVMLegacyFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        let expected_evm_filter_exec = EVMLegacyFilterExec {
            table_number: 0,
            where_clause: EVMDynProofExpr::Equals(EVMEqualsExpr::new(
                EVMDynProofExpr::Column(EVMColumnExpr::new(0)),
                EVMDynProofExpr::Literal(EVMLiteralExpr(LiteralValue::BigInt(5))),
            )),
            results: vec![EVMDynProofExpr::Column(EVMColumnExpr::new(1))],
        };

        assert_eq!(evm_filter_exec, expected_evm_filter_exec);

        // Roundtrip
        let roundtripped_filter_exec = EVMLegacyFilterExec::try_into_proof_plan(
            &evm_filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![alias]),
        )
        .unwrap();
        assert_eq!(roundtripped_filter_exec, filter_exec);

        assert!(matches!(
            EVMLegacyFilterExec::try_into_proof_plan(
                &evm_filter_exec,
                &indexset![],
                &indexset![],
                Some(&indexset![]),
            )
            .unwrap_err(),
            EVMProofPlanError::InvalidOutputColumnName
        ));
    }

    #[test]
    fn we_can_put_group_by_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a group by exec
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(column_ref_a.clone())],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        // Convert to EVM plan
        let evm_group_by_exec = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the structure
        assert_eq!(evm_group_by_exec.table_number, 0);
        assert_eq!(evm_group_by_exec.group_by_exprs, vec![0]); // column_ref_a is at index 0
        assert_eq!(evm_group_by_exec.sum_expr.len(), 1);
        assert!(matches!(
            evm_group_by_exec.sum_expr[0],
            EVMDynProofExpr::Column(_)
        ));
        assert_eq!(evm_group_by_exec.count_alias_name, count_alias);
        assert!(matches!(
            evm_group_by_exec.where_clause,
            EVMDynProofExpr::Equals(_)
        ));

        // Roundtrip
        let roundtripped_group_by_exec = EVMGroupByExec::try_into_proof_plan(
            &evm_group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![
                ident_a.value.clone(),
                sum_alias.clone(),
                count_alias.clone()
            ]),
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_group_by_exec.group_by_exprs().len(), 1);
        assert_eq!(
            roundtripped_group_by_exec.group_by_exprs()[0].get_column_reference(),
            column_ref_a
        );
        assert_eq!(roundtripped_group_by_exec.sum_expr().len(), 1);
        assert!(matches!(
            roundtripped_group_by_exec.sum_expr()[0].expr,
            DynProofExpr::Column(_)
        ));
        assert_eq!(roundtripped_group_by_exec.count_alias().value, count_alias);
        assert_eq!(roundtripped_group_by_exec.table().table_ref, table_ref);
        assert!(matches!(
            roundtripped_group_by_exec.where_clause(),
            DynProofExpr::Equals(_)
        ));
    }

    #[test]
    fn group_by_exec_fails_with_column_not_found_from_proof_plan() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let missing_ident: Ident = "missing".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);
        let missing_column =
            ColumnRef::new(table_ref.clone(), missing_ident.clone(), ColumnType::BigInt);

        // Create a group by exec with a column that doesn't exist in column_refs
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(missing_column)],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias),
            }],
            Ident::new(count_alias),
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let result = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::ColumnNotFound)));
    }

    #[test]
    fn group_by_exec_fails_with_table_not_found_from_proof_plan() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let missing_table_ref: TableRef = "namespace.missing".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a group by exec with a table that doesn't exist in table_refs
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(column_ref_a.clone())],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias),
            }],
            Ident::new(count_alias),
            TableExpr {
                table_ref: missing_table_ref,
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let result = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn group_by_exec_fails_with_column_not_found_into_proof_plan() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a valid group by exec first
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(column_ref_a.clone())],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let evm_group_by_exec = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Now try to convert back with an empty column_refs
        let result = EVMGroupByExec::try_into_proof_plan(
            &evm_group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![],
            Some(&indexset![
                ident_a.value.clone(),
                sum_alias.clone(),
                count_alias.clone()
            ]),
        );

        assert!(matches!(result, Err(EVMProofPlanError::ColumnNotFound)));
    }

    #[test]
    fn group_by_exec_fails_with_table_not_found_into_proof_plan() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a valid group by exec first
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(column_ref_a.clone())],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let evm_group_by_exec = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Now try to convert back with an empty table_refs
        let result = EVMGroupByExec::try_into_proof_plan(
            &evm_group_by_exec,
            &indexset![],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![
                ident_a.value.clone(),
                sum_alias.clone(),
                count_alias.clone()
            ]),
        );

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn group_by_exec_fails_with_invalid_output_column_name_into_proof_plan() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a valid group by exec first
        let group_by_exec = GroupByExec::try_new(
            vec![ColumnExpr::new(column_ref_a.clone())],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            TableExpr {
                table_ref: table_ref.clone(),
            },
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let evm_group_by_exec = EVMGroupByExec::try_from_proof_plan(
            &group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Now try to convert back with incorrect output column names
        let result = EVMGroupByExec::try_into_proof_plan(
            &evm_group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![ident_a.value.clone(), sum_alias.clone()]), // Missing count_alias
        );

        assert!(matches!(
            result,
            Err(EVMProofPlanError::InvalidOutputColumnName)
        ));

        // Try with wrong count alias name
        let wrong_count_alias = "wrong_count".to_string();
        let result = EVMGroupByExec::try_into_proof_plan(
            &evm_group_by_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![
                ident_a.value.clone(),
                sum_alias.clone(),
                wrong_count_alias
            ]),
        );

        assert!(matches!(
            result,
            Err(EVMProofPlanError::InvalidOutputColumnName)
        ));
    }

    #[test]
    fn we_can_put_union_exec_in_evm() {
        let top_table_ref: TableRef = "namespace.top_table".parse().unwrap();
        let bottom_table_ref: TableRef = "namespace.bottom_table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();

        let top_column_ref_a =
            ColumnRef::new(top_table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let top_column_ref_b =
            ColumnRef::new(top_table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        let bottom_column_ref_a = ColumnRef::new(
            bottom_table_ref.clone(),
            ident_a.clone(),
            ColumnType::BigInt,
        );
        let bottom_column_ref_b = ColumnRef::new(
            bottom_table_ref.clone(),
            ident_b.clone(),
            ColumnType::BigInt,
        );

        // Create columns fields to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];

        // Create a union exec
        let union_exec = UnionExec::try_new(vec![
            DynProofPlan::Projection(ProjectionExec::new(
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::Add(
                        AddExpr::try_new(
                            Box::new(DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                                TableRef::from_names(None, ""),
                                ident_a.clone(),
                                ColumnType::BigInt,
                            )))),
                            Box::new(DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                                TableRef::from_names(None, ""),
                                ident_b.clone(),
                                ColumnType::BigInt,
                            )))),
                        )
                        .unwrap(),
                    ),
                    alias: "ab_sum".into(),
                }],
                Box::new(DynProofPlan::Table(TableExec::new(
                    top_table_ref.clone(),
                    column_fields.clone(),
                ))),
            )),
            DynProofPlan::Projection(ProjectionExec::new(
                vec![AliasedDynProofExpr {
                    expr: DynProofExpr::Add(
                        AddExpr::try_new(
                            Box::new(DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                                TableRef::from_names(None, ""),
                                ident_a.clone(),
                                ColumnType::BigInt,
                            )))),
                            Box::new(DynProofExpr::Column(ColumnExpr::new(ColumnRef::new(
                                TableRef::from_names(None, ""),
                                ident_b.clone(),
                                ColumnType::BigInt,
                            )))),
                        )
                        .unwrap(),
                    ),
                    alias: "ab_sum".into(),
                }],
                Box::new(DynProofPlan::Table(TableExec::new(
                    bottom_table_ref.clone(),
                    column_fields.clone(),
                ))),
            )),
        ])
        .unwrap();
        let output_column_names = union_exec
            .get_column_result_fields()
            .iter()
            .map(|cr| cr.name().to_string())
            .collect();

        let table_refs = &indexset![top_table_ref, bottom_table_ref];
        let column_refs = &indexset![
            top_column_ref_a,
            top_column_ref_b,
            bottom_column_ref_a,
            bottom_column_ref_b
        ];

        // Convert to EVM plan
        let evm_union_exec =
            EVMUnionExec::try_from_proof_plan(&union_exec, table_refs, column_refs).unwrap();

        assert_eq!(evm_union_exec.inputs.len(), 2);

        let round_tripped_union_exec = evm_union_exec
            .try_into_proof_plan(table_refs, column_refs, Some(&output_column_names))
            .unwrap();
        assert_eq!(
            union_exec.get_column_result_fields(),
            round_tripped_union_exec.get_column_result_fields()
        );
    }

    #[test]
    fn we_can_put_sort_merge_join_exec_in_evm() {
        let left_table_ref: TableRef = "namespace.left_table".parse().unwrap();
        let right_table_ref: TableRef = "namespace.right_table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let ident_c: Ident = "c".into();

        let left_column_ref_a =
            ColumnRef::new(left_table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let left_column_ref_b =
            ColumnRef::new(left_table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        let right_column_ref_a =
            ColumnRef::new(right_table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let right_column_ref_b =
            ColumnRef::new(right_table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create columns fields to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];

        // Create a sort merge join exec
        let sort_merge_join_exec = DynProofPlan::SortMergeJoin(SortMergeJoinExec::new(
            Box::new(DynProofPlan::new_table(
                left_table_ref.clone(),
                column_fields.clone(),
            )),
            Box::new(DynProofPlan::new_table(
                right_table_ref.clone(),
                column_fields,
            )),
            vec![0],
            vec![0],
            vec![ident_a, ident_b, ident_c],
        ));
        let output_column_names = sort_merge_join_exec
            .get_column_result_fields()
            .iter()
            .map(|cr| cr.name().to_string())
            .collect();

        let table_refs = &indexset![left_table_ref, right_table_ref];
        let column_refs = &indexset![
            left_column_ref_a,
            left_column_ref_b,
            right_column_ref_a,
            right_column_ref_b
        ];

        // Convert to EVM plan
        let evm_sort_merge_join_exec =
            EVMDynProofPlan::try_from_proof_plan(&sort_merge_join_exec, table_refs, column_refs)
                .unwrap();

        let round_tripped_sort_merge_join_exec = evm_sort_merge_join_exec
            .try_into_proof_plan(table_refs, column_refs, Some(&output_column_names))
            .unwrap();
        assert_eq!(sort_merge_join_exec, round_tripped_sort_merge_join_exec);
    }

    #[test]
    fn we_can_put_simple_filter_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a = "a".into();
        let ident_b = "b".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a, ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b, ColumnType::BigInt);

        // Create a table exec to use as the input
        let column_fields = vec![
            ColumnField::new(Ident::new("a"), ColumnType::BigInt),
            ColumnField::new(Ident::new("b"), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a filter exec
        let filter_exec = FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias.clone()),
            }],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        );

        // Convert to EVM plan
        let evm_filter_exec = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the structure
        assert_eq!(evm_filter_exec.results.len(), 1);
        assert!(matches!(
            evm_filter_exec.results[0],
            EVMDynProofExpr::Column(_)
        ));
        assert!(matches!(
            evm_filter_exec.where_clause,
            EVMDynProofExpr::Equals(_)
        ));
        assert!(matches!(
            *evm_filter_exec.input_plan,
            EVMDynProofPlan::Table(_)
        ));

        // Roundtrip
        let roundtripped_filter_exec = EVMFilterExec::try_into_proof_plan(
            &evm_filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![alias]),
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_filter_exec.aliased_results().len(), 1);
        assert!(matches!(
            roundtripped_filter_exec.aliased_results()[0].expr,
            DynProofExpr::Column(_)
        ));
        assert!(matches!(
            roundtripped_filter_exec.where_clause(),
            DynProofExpr::Equals(_)
        ));
        assert!(matches!(
            *roundtripped_filter_exec.input(),
            DynProofPlan::Table(_)
        ));
    }

    #[test]
    fn we_can_put_complex_filter_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let ident_c: Ident = "c".into();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a slice exec as the input (to test nested plans)
        let slice_exec = SliceExec::new(Box::new(DynProofPlan::Table(table_exec)), 5, Some(10));

        // Create a filter exec with the slice as input
        let filter_exec = FilterExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone())),
                    alias: ident_a.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                    alias: ident_c.clone(),
                },
            ],
            Box::new(DynProofPlan::Slice(slice_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(42),
                    ))),
                )
                .unwrap(),
            ),
        );

        // Convert to EVM plan
        let evm_filter_exec = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify nested structure
        assert!(matches!(
            *evm_filter_exec.input_plan,
            EVMDynProofPlan::Slice(_)
        ));

        // Roundtrip
        let roundtripped = evm_filter_exec
            .try_into_proof_plan(
                &indexset![table_ref],
                &indexset![column_ref_a, column_ref_b],
                Some(&indexset![ident_a.value, ident_c.value]),
            )
            .unwrap();

        // Verify the roundtripped plan
        assert_eq!(roundtripped.aliased_results().len(), 2);
        assert!(matches!(*roundtripped.input(), DynProofPlan::Slice(_)));

        assert!(matches!(
            evm_filter_exec
                .try_into_proof_plan(&indexset![], &indexset![], Some(&indexset![]),)
                .unwrap_err(),
            EVMProofPlanError::InvalidOutputColumnName
        ));
    }

    #[expect(clippy::too_many_lines)]
    #[test]
    fn we_can_put_nested_filter_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let ident_c: Ident = "c".into();
        let alias_1 = "result_1";
        let alias_2 = "result_2";
        let alias_3 = "result_3";

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);
        let column_ref_c = ColumnRef::new(table_ref.clone(), ident_c.clone(), ColumnType::BigInt);

        let column_ref_1 = ColumnRef::new(table_ref.clone(), alias_1.into(), ColumnType::BigInt);
        let column_ref_2 = ColumnRef::new(table_ref.clone(), alias_2.into(), ColumnType::BigInt);

        // Create a table exec as the base
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
            ColumnField::new(ident_c.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // First filter: filter where a = 10
        let filter_1 = FilterExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone())),
                    alias: Ident::new(alias_1),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                    alias: ident_b.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_c.clone())),
                    alias: ident_c.clone(),
                },
            ],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(10),
                    ))),
                )
                .unwrap(),
            ),
        );

        // Second filter: filter where b > 20
        let filter_2 = FilterExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_1.clone())),
                    alias: ident_a.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                    alias: Ident::new(alias_2),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_c.clone())),
                    alias: ident_c.clone(),
                },
            ],
            Box::new(DynProofPlan::Filter(filter_1)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(20),
                    ))),
                )
                .unwrap(),
            ),
        );

        // Third filter: filter where c = 30
        let filter_3 = FilterExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone())),
                    alias: ident_a.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_2.clone())),
                    alias: ident_b.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_c.clone())),
                    alias: Ident::new(alias_3),
                },
            ],
            Box::new(DynProofPlan::Filter(filter_2)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_c.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(30),
                    ))),
                )
                .unwrap(),
            ),
        );

        // Convert to EVM plan
        let evm_filter_3 = EVMFilterExec::try_from_proof_plan(
            &filter_3,
            &indexset![table_ref.clone()],
            &indexset![
                column_ref_a.clone(),
                column_ref_b.clone(),
                column_ref_c.clone()
            ],
        )
        .unwrap();

        // Verify nested structure: should have Filter containing Filter containing Table
        assert!(matches!(
            *evm_filter_3.input_plan,
            EVMDynProofPlan::Filter(_)
        ));
        if let EVMDynProofPlan::Filter(ref evm_filter_2) = *evm_filter_3.input_plan {
            assert!(matches!(
                *evm_filter_2.input_plan,
                EVMDynProofPlan::Filter(_)
            ));
            if let EVMDynProofPlan::Filter(ref evm_filter_1) = *evm_filter_2.input_plan {
                assert!(matches!(
                    *evm_filter_1.input_plan,
                    EVMDynProofPlan::Table(_)
                ));
            }
        }

        // Roundtrip
        let roundtripped = evm_filter_3
            .try_into_proof_plan(
                &indexset![table_ref],
                &indexset![column_ref_a, column_ref_b, column_ref_c],
                Some(&indexset![
                    ident_a.value,
                    ident_b.value,
                    alias_3.to_string()
                ]),
            )
            .unwrap();

        // Verify the roundtripped plan has the expected nested structure
        assert_eq!(roundtripped.aliased_results().len(), 3);
        assert!(matches!(*roundtripped.input(), DynProofPlan::Filter(_)));

        // Verify second level
        if let DynProofPlan::Filter(ref filter_2_roundtripped) = *roundtripped.input() {
            assert!(matches!(
                *filter_2_roundtripped.input(),
                DynProofPlan::Filter(_)
            ));

            // Verify third level (innermost)
            if let DynProofPlan::Filter(ref filter_1_roundtripped) = *filter_2_roundtripped.input()
            {
                assert!(matches!(
                    *filter_1_roundtripped.input(),
                    DynProofPlan::Table(_)
                ));
            }
        }
    }

    #[test]
    fn filter_exec_fails_with_column_not_found_in_where_clause() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let missing_ident: Ident = "missing".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);
        let missing_column =
            ColumnRef::new(table_ref.clone(), missing_ident.clone(), ColumnType::BigInt);

        // Create a table exec to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a filter exec with a where clause that references a missing column
        let filter_exec = FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias),
            }],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(missing_column))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        );

        let result = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::ColumnNotFound)));
    }

    #[test]
    fn filter_exec_fails_with_column_not_found_in_results() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let missing_ident: Ident = "missing".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);
        let missing_column =
            ColumnRef::new(table_ref.clone(), missing_ident.clone(), ColumnType::BigInt);

        // Create a table exec to use as the input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a filter exec with a result that references a missing column
        let filter_exec = FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(missing_column)),
                alias: Ident::new(alias),
            }],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        );

        let result = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::ColumnNotFound)));
    }

    #[test]
    fn filter_exec_fails_with_table_not_found_in_input() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let missing_table_ref: TableRef = "namespace.missing".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let alias = "alias".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec with a missing table reference
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(missing_table_ref, column_fields);

        // Create a filter exec
        let filter_exec = FilterExec::new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                alias: Ident::new(alias),
            }],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Equals(
                EqualsExpr::try_new(
                    Box::new(DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone()))),
                    Box::new(DynProofExpr::Literal(LiteralExpr::new(
                        LiteralValue::BigInt(5),
                    ))),
                )
                .unwrap(),
            ),
        );

        let result = EVMFilterExec::try_from_proof_plan(
            &filter_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn we_can_put_simple_aggregate_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec as input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create output column refs for aggregate (these come from the table's output)
        let output_col_a = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_a.clone(),
            ColumnType::BigInt,
        );
        let output_col_b = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_b.clone(),
            ColumnType::BigInt,
        );

        // Create an aggregate exec
        let aggregate_exec = AggregateExec::try_new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_a.clone())),
                alias: ident_a.clone(),
            }],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_b.clone())),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
        )
        .unwrap();

        // Convert to EVM plan
        let evm_aggregate_exec = EVMAggregateExec::try_from_proof_plan(
            &aggregate_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Verify the structure
        assert_eq!(evm_aggregate_exec.group_by_exprs.len(), 1);
        assert_eq!(evm_aggregate_exec.sum_expr.len(), 1);
        assert_eq!(evm_aggregate_exec.count_alias_name, count_alias);
        assert!(matches!(
            *evm_aggregate_exec.input_plan,
            EVMDynProofPlan::Table(_)
        ));

        // Roundtrip
        let roundtripped_aggregate_exec = EVMAggregateExec::try_into_proof_plan(
            &evm_aggregate_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
            Some(&indexset![ident_a.value, sum_alias, count_alias]),
        )
        .unwrap();

        // Verify the roundtripped plan has the expected structure
        assert_eq!(roundtripped_aggregate_exec.group_by_exprs().len(), 1);
        assert_eq!(roundtripped_aggregate_exec.sum_expr().len(), 1);
        assert!(matches!(
            roundtripped_aggregate_exec.group_by_exprs()[0].expr,
            DynProofExpr::Column(_)
        ));
        assert!(matches!(
            *roundtripped_aggregate_exec.input(),
            DynProofPlan::Table(_)
        ));
    }

    #[test]
    fn we_can_put_complex_aggregate_exec_in_evm() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let ident_c: Ident = "c".into();
        let sum_alias_b = "sum_b".to_string();
        let sum_alias_c = "sum_c".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);
        let column_ref_c = ColumnRef::new(table_ref.clone(), ident_c.clone(), ColumnType::BigInt);

        // Create a table exec
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
            ColumnField::new(ident_c.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Create a filter exec as the input (to test nested plans)
        let filter_exec = FilterExec::new(
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_a.clone())),
                    alias: ident_a.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_b.clone())),
                    alias: ident_b.clone(),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(column_ref_c.clone())),
                    alias: ident_c.clone(),
                },
            ],
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
        );

        // Output columns from filter (used by aggregate)
        let filter_output_col_a = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_a.clone(),
            ColumnType::BigInt,
        );
        let filter_output_col_b = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_b.clone(),
            ColumnType::BigInt,
        );
        let filter_output_col_c = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_c.clone(),
            ColumnType::BigInt,
        );

        // Create an aggregate exec with the filter as input
        let aggregate_exec = AggregateExec::try_new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(filter_output_col_a.clone())),
                alias: ident_a.clone(),
            }],
            vec![
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(filter_output_col_b.clone())),
                    alias: Ident::new(sum_alias_b.clone()),
                },
                AliasedDynProofExpr {
                    expr: DynProofExpr::Column(ColumnExpr::new(filter_output_col_c.clone())),
                    alias: Ident::new(sum_alias_c.clone()),
                },
            ],
            Ident::new(count_alias.clone()),
            Box::new(DynProofPlan::Filter(filter_exec)),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
        )
        .unwrap();

        // Convert to EVM plan
        let evm_aggregate_exec = EVMAggregateExec::try_from_proof_plan(
            &aggregate_exec,
            &indexset![table_ref.clone()],
            &indexset![
                column_ref_a.clone(),
                column_ref_b.clone(),
                column_ref_c.clone()
            ],
        )
        .unwrap();

        // Verify nested structure
        assert!(matches!(
            *evm_aggregate_exec.input_plan,
            EVMDynProofPlan::Filter(_)
        ));
        assert_eq!(evm_aggregate_exec.group_by_exprs.len(), 1);
        assert_eq!(evm_aggregate_exec.sum_expr.len(), 2);

        // Roundtrip
        let roundtripped = evm_aggregate_exec
            .try_into_proof_plan(
                &indexset![table_ref],
                &indexset![column_ref_a, column_ref_b, column_ref_c],
                Some(&indexset![
                    ident_a.value,
                    sum_alias_b.clone(),
                    sum_alias_c.clone(),
                    count_alias.clone()
                ]),
            )
            .unwrap();

        // Verify the roundtripped plan
        assert_eq!(roundtripped.group_by_exprs().len(), 1);
        assert_eq!(roundtripped.sum_expr().len(), 2);
        assert!(matches!(*roundtripped.input(), DynProofPlan::Filter(_)));
    }

    #[test]
    fn aggregate_exec_fails_with_table_not_found_in_input() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let missing_table_ref: TableRef = "namespace.missing".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Output columns (from table's perspective)
        let output_col_a = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_a.clone(),
            ColumnType::BigInt,
        );
        let output_col_b = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_b.clone(),
            ColumnType::BigInt,
        );

        // Create a table exec with a missing table reference
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(missing_table_ref, column_fields);

        // Create an aggregate exec
        let aggregate_exec = AggregateExec::try_new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_a)),
                alias: ident_a.clone(),
            }],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_b)),
                alias: Ident::new(sum_alias),
            }],
            Ident::new(count_alias),
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
        )
        .unwrap();

        let result = EVMAggregateExec::try_from_proof_plan(
            &aggregate_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
        );

        assert!(matches!(result, Err(EVMProofPlanError::TableNotFound)));
    }

    #[test]
    fn aggregate_exec_fails_with_invalid_output_column_names() {
        let table_ref: TableRef = "namespace.table".parse().unwrap();
        let ident_a: Ident = "a".into();
        let ident_b: Ident = "b".into();
        let sum_alias = "sum_b".to_string();
        let count_alias = "count".to_string();

        let column_ref_a = ColumnRef::new(table_ref.clone(), ident_a.clone(), ColumnType::BigInt);
        let column_ref_b = ColumnRef::new(table_ref.clone(), ident_b.clone(), ColumnType::BigInt);

        // Create a table exec as input
        let column_fields = vec![
            ColumnField::new(ident_a.clone(), ColumnType::BigInt),
            ColumnField::new(ident_b.clone(), ColumnType::BigInt),
        ];
        let table_exec = TableExec::new(table_ref.clone(), column_fields);

        // Output columns
        let output_col_a = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_a.clone(),
            ColumnType::BigInt,
        );
        let output_col_b = ColumnRef::new(
            TableRef::from_names(None, ""),
            ident_b.clone(),
            ColumnType::BigInt,
        );

        // Create an aggregate exec
        let aggregate_exec = AggregateExec::try_new(
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_a)),
                alias: ident_a.clone(),
            }],
            vec![AliasedDynProofExpr {
                expr: DynProofExpr::Column(ColumnExpr::new(output_col_b)),
                alias: Ident::new(sum_alias.clone()),
            }],
            Ident::new(count_alias.clone()),
            Box::new(DynProofPlan::Table(table_exec)),
            DynProofExpr::Literal(LiteralExpr::new(LiteralValue::Boolean(true))),
        )
        .unwrap();

        let evm_aggregate_exec = EVMAggregateExec::try_from_proof_plan(
            &aggregate_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
        )
        .unwrap();

        // Try to convert back with incorrect output column names (missing count_alias)
        let result = EVMAggregateExec::try_into_proof_plan(
            &evm_aggregate_exec,
            &indexset![table_ref.clone()],
            &indexset![column_ref_a.clone(), column_ref_b.clone()],
            Some(&indexset![ident_a.value.clone(), sum_alias.clone()]), // Missing count_alias
        );

        assert!(matches!(
            result,
            Err(EVMProofPlanError::InvalidOutputColumnName)
        ));

        // Try with wrong count alias name
        let wrong_count_alias = "wrong_count".to_string();
        let result = EVMAggregateExec::try_into_proof_plan(
            &evm_aggregate_exec,
            &indexset![table_ref],
            &indexset![column_ref_a, column_ref_b],
            Some(&indexset![ident_a.value, sum_alias, wrong_count_alias]),
        );

        assert!(matches!(
            result,
            Err(EVMProofPlanError::InvalidOutputColumnName)
        ));
    }

    #[test]
    fn we_can_unwrap_correct_output_column_names_when_none() {
        let output_column_names =
            try_unwrap_output_column_names_with_count_alias(None, 2, &"0".to_string()).unwrap();
        let expected_output_column_names: IndexSet<
            String,
            core::hash::BuildHasherDefault<ahash::AHasher>,
        > = vec!["1".to_string(), "0".to_string()].into_iter().collect();
        assert_eq!(output_column_names, expected_output_column_names);
    }

    #[test]
    fn we_can_unwrap_correct_output_column_names_when_some() {
        let expected_output_column_names: IndexSet<String, _> =
            vec!["a".to_string(), "b".to_string()].into_iter().collect();
        let output_column_names = try_unwrap_output_column_names_with_count_alias(
            Some(&expected_output_column_names),
            2,
            &"b".to_string(),
        )
        .unwrap();

        assert_eq!(output_column_names, expected_output_column_names);
    }

    #[test]
    fn we_can_unwrap_err_when_mismatching_count_alias() {
        let expected_output_column_names: IndexSet<String, _> =
            vec!["a".to_string(), "b".to_string()].into_iter().collect();
        let err = try_unwrap_output_column_names_with_count_alias(
            Some(&expected_output_column_names),
            3,
            &"b".to_string(),
        )
        .unwrap_err();

        assert!(matches!(err, EVMProofPlanError::InvalidOutputColumnName));
    }
}
