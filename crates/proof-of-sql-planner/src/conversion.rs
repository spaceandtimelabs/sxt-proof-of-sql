use crate::{logical_plan_to_proof_plan, PlannerResult};
use datafusion::{
    catalog::TableReference,
    common::DFSchema,
    config::ConfigOptions,
    optimizer::{Analyzer, Optimizer, OptimizerContext},
    sql::planner::{ContextProvider, SqlToRel},
};
use indexmap::IndexMap;
use proof_of_sql::sql::proof_plans::DynProofPlan;
use sqlparser::{dialect::GenericDialect, parser::Parser};

/// Convert a SQL query to a `DynProofPlan` using schema from provided tables
///
/// This function does the following
/// 1. Parse the SQL query into AST using sqlparser
/// 2. Convert the AST into a `LogicalPlan` using `SqlToRel`
/// 3. Analyze the `LogicalPlan` using `Analyzer`
/// 4. Optimize the `LogicalPlan` using `Optimizer`
/// 5. Convert the optimized `LogicalPlan` into a `DynProofPlan`
pub fn sql_to_proof_plans<S: ContextProvider>(
    sql: &str,
    context_provider: &S,
    schemas: &IndexMap<TableReference, DFSchema>,
    config: &ConfigOptions,
) -> PlannerResult<Vec<DynProofPlan>> {
    // 1. Parse the SQL query into AST using sqlparser
    let dialect = GenericDialect {};
    let asts = Parser::parse_sql(&dialect, sql)?;
    asts.iter()
        .map(|ast| -> PlannerResult<DynProofPlan> {
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
            // 5. Convert the optimized `LogicalPlan` into a `DynProofPlan`
            logical_plan_to_proof_plan(&optimized_logical_plan, schemas)
        })
        .collect::<PlannerResult<Vec<_>>>()
}
