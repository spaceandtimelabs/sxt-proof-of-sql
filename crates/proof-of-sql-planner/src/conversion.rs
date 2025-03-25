use crate::{
    logical_plan_to_proof_plan, logical_plan_to_proof_plan_with_postprocessing, PlannerResult,
    ProofPlanWithPostprocessing,
};
use datafusion::{
    catalog::TableReference,
    common::DFSchema,
    config::ConfigOptions,
    logical_expr::LogicalPlan,
    optimizer::{Analyzer, Optimizer, OptimizerContext},
    sql::planner::{ContextProvider, SqlToRel},
};
use indexmap::IndexMap;
use proof_of_sql::sql::proof_plans::DynProofPlan;
use sqlparser::{dialect::GenericDialect, parser::Parser};

/// Convert a SQL query to a Proof of SQL plan using schema from provided tables
///
/// This function does the following
/// 1. Parse the SQL query into AST using sqlparser
/// 2. Convert the AST into a `LogicalPlan` using `SqlToRel`
/// 3. Analyze the `LogicalPlan` using `Analyzer`
/// 4. Optimize the `LogicalPlan` using `Optimizer`
/// 5. Convert the optimized `LogicalPlan` into a Proof of SQL plan
fn sql_to_posql_plans<S, T, F>(
    sql: &str,
    context_provider: &S,
    schemas: &IndexMap<TableReference, DFSchema>,
    config: &ConfigOptions,
    planner_converter: F,
) -> PlannerResult<Vec<T>>
where
    S: ContextProvider,
    F: Fn(&LogicalPlan, &IndexMap<TableReference, DFSchema>) -> PlannerResult<T>,
{
    // 1. Parse the SQL query into AST using sqlparser
    let dialect = GenericDialect {};
    let asts = Parser::parse_sql(&dialect, sql)?;
    asts.iter()
        .map(|ast| -> PlannerResult<T> {
            // 2. Convert the AST into a `LogicalPlan` using `SqlToRel`
            let raw_logical_plan =
                SqlToRel::new(context_provider).sql_statement_to_plan(ast.clone())?;
            // 3. Analyze the `LogicalPlan` using `Analyzer`
            let analyzer = Analyzer::new();
            let analyzed_logical_plan =
                analyzer.execute_and_check(raw_logical_plan, config, |_, _| {})?;
            // 4. Optimize the `LogicalPlan` using `Optimizer`
            let optimizer = Optimizer::new();
            let optimizer_context = OptimizerContext::default();
            let optimized_logical_plan =
                optimizer.optimize(analyzed_logical_plan, &optimizer_context, |_, _| {})?;
            // 5. Convert the optimized `LogicalPlan` into a Proof of SQL plan
            planner_converter(&optimized_logical_plan, schemas)
        })
        .collect::<PlannerResult<Vec<_>>>()
}

/// Convert a SQL query to a `DynProofPlan` using schema from provided tables
///
/// See `sql_to_posql_plans` for more details
pub fn sql_to_proof_plans<S: ContextProvider>(
    sql: &str,
    context_provider: &S,
    schemas: &IndexMap<TableReference, DFSchema>,
    config: &ConfigOptions,
) -> PlannerResult<Vec<DynProofPlan>> {
    sql_to_posql_plans(
        sql,
        context_provider,
        schemas,
        config,
        logical_plan_to_proof_plan,
    )
}

/// Convert a SQL query to a `ProofPlanWithPostprocessing` using schema from provided tables
///
/// See `sql_to_posql_plans` for more details
pub fn sql_to_proof_plans_with_postprocessing<S: ContextProvider>(
    sql: &str,
    context_provider: &S,
    schemas: &IndexMap<TableReference, DFSchema>,
    config: &ConfigOptions,
) -> PlannerResult<Vec<ProofPlanWithPostprocessing>> {
    sql_to_posql_plans(
        sql,
        context_provider,
        schemas,
        config,
        logical_plan_to_proof_plan_with_postprocessing,
    )
}
