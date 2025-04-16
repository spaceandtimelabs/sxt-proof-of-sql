use crate::{
    logical_plan_to_proof_plan, logical_plan_to_proof_plan_with_postprocessing, PlannerResult,
    ProofPlanWithPostprocessing,
};
use alloc::{sync::Arc, vec::Vec};
use datafusion::{
    config::ConfigOptions,
    logical_expr::LogicalPlan,
    optimizer::{Analyzer, Optimizer, OptimizerContext, OptimizerRule},
    sql::planner::{ContextProvider, SqlToRel},
};
use indexmap::IndexSet;
use proof_of_sql::{
    base::database::{ParseError, SchemaAccessor, TableRef},
    sql::proof_plans::DynProofPlan,
};
use sqlparser::ast::{visit_relations, Statement};
use std::ops::ControlFlow;

/// Get [`Optimizer`]
///
/// In order to support queries such as `select $1::varchar;` we have to temporarily disable
/// [`CommonSubexprEliminate`] rule in the optimizer in `DataFusion` 38. Once we upgrade to
/// `DataFusion` 46 we can remove this function and use `Optimizer::new()` directly.
pub fn optimizer() -> Optimizer {
    // Step 1: Grab the recommended set
    let recommended_rules: Vec<Arc<dyn OptimizerRule + Send + Sync>> = Optimizer::new().rules;

    // Step 2: Filter out [`CommonSubexprEliminate`]
    let filtered_rules = recommended_rules
        .into_iter()
        .filter(|rule| rule.name() != "common_sub_expression_eliminate")
        .collect::<Vec<_>>();

    // Step 3: Build an optimizer with the new list
    Optimizer::with_rules(filtered_rules)
}

/// Convert a SQL query to a Proof of SQL plan using schema from provided tables
///
/// This function does the following
/// 1. Parse the SQL query into AST using sqlparser
/// 2. Convert the AST into a `LogicalPlan` using `SqlToRel`
/// 3. Analyze the `LogicalPlan` using `Analyzer`
/// 4. Optimize the `LogicalPlan` using `Optimizer`
/// 5. Convert the optimized `LogicalPlan` into a Proof of SQL plan
fn sql_to_posql_plans<S, T, F, A>(
    statements: &[Statement],
    context_provider: &S,
    schemas: &A,
    config: &ConfigOptions,
    planner_converter: F,
) -> PlannerResult<Vec<T>>
where
    S: ContextProvider,
    F: Fn(&LogicalPlan, &A) -> PlannerResult<T>,
    A: SchemaAccessor,
{
    // 1. Parse the SQL query into AST using sqlparser
    statements
        .iter()
        .map(|ast| -> PlannerResult<T> {
            // 2. Convert the AST into a `LogicalPlan` using `SqlToRel`
            let raw_logical_plan =
                SqlToRel::new(context_provider).sql_statement_to_plan(ast.clone())?;
            // 3. Analyze the `LogicalPlan` using `Analyzer`
            let analyzer = Analyzer::new();
            let analyzed_logical_plan =
                analyzer.execute_and_check(raw_logical_plan, config, |_, _| {})?;
            // 4. Optimize the `LogicalPlan` using `Optimizer`
            let optimizer = optimizer();
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
pub fn sql_to_proof_plans<S: ContextProvider, A: SchemaAccessor>(
    statements: &[Statement],
    context_provider: &S,
    schemas: &A,
    config: &ConfigOptions,
) -> PlannerResult<Vec<DynProofPlan>> {
    sql_to_posql_plans(
        statements,
        context_provider,
        schemas,
        config,
        logical_plan_to_proof_plan,
    )
}

/// Convert a SQL query to a `ProofPlanWithPostprocessing` using schema from provided tables
///
/// See `sql_to_posql_plans` for more details
pub fn sql_to_proof_plans_with_postprocessing<S: ContextProvider, A: SchemaAccessor>(
    statements: &[Statement],
    context_provider: &S,
    schemas: &A,
    config: &ConfigOptions,
) -> PlannerResult<Vec<ProofPlanWithPostprocessing>> {
    sql_to_posql_plans(
        statements,
        context_provider,
        schemas,
        config,
        logical_plan_to_proof_plan_with_postprocessing,
    )
}

/// Given a `Statement` retrieves all unique tables in the query
pub fn get_table_refs_from_statement(
    statement: &Statement,
) -> Result<IndexSet<TableRef>, ParseError> {
    let mut table_refs: IndexSet<TableRef> = IndexSet::<TableRef>::new();
    visit_relations(statement, |object_name| {
        match object_name.to_string().as_str().try_into() {
            Ok(table_ref) => {
                table_refs.insert(table_ref);
                ControlFlow::Continue(())
            }
            e => ControlFlow::Break(e),
        }
    })
    .break_value()
    .transpose()?;
    Ok(table_refs)
}

#[cfg(test)]
mod tests {
    use super::get_table_refs_from_statement;
    use indexmap::IndexSet;
    use proof_of_sql::base::database::TableRef;
    use sqlparser::{dialect::GenericDialect, parser::Parser};

    #[test]
    fn we_can_get_table_references() {
        let statement = Parser::parse_sql(
            &GenericDialect {},
            "SELECT e.employee_id, e.employee_name, d.department_name, p.project_name, s.salary
FROM employees e
JOIN departments d ON e.department_id = d.department_id
JOIN management.projects p ON e.employee_id = p.employee_id
JOIN internal.salaries s ON e.employee_id = s.employee_id
WHERE e.department_id IN (
    SELECT department_id
    FROM departments
    WHERE department_name = 'Sales'
)
AND p.project_id IN (
    SELECT project_id
    FROM project_assignments
    WHERE employee_id = e.employee_id
)
AND s.salary > (
    SELECT AVG(salary)
    FROM internal.salaries
    WHERE department_id = e.department_id
);
",
        )
        .unwrap()[0]
            .clone();
        let table_refs = get_table_refs_from_statement(&statement).unwrap();
        let expected_table_refs: IndexSet<TableRef> = [
            ("", "departments"),
            ("", "employees"),
            ("management", "projects"),
            ("", "project_assignments"),
            ("internal", "salaries"),
        ]
        .map(|(s, t)| TableRef::new(s, t))
        .into_iter()
        .collect();
        assert_eq!(table_refs, expected_table_refs);
    }
}
