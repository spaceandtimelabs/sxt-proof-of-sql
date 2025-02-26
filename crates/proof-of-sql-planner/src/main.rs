// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::{
    common::{plan_err, TableReference},
    config::ConfigOptions,
    error::Result,
    logical_expr::{
        AggregateUDF, Expr, LogicalPlan, ScalarUDF, TableProviderFilterPushDown, TableSource,
        WindowUDF,
    },
    optimizer::{
        Analyzer, AnalyzerRule, Optimizer, OptimizerConfig, OptimizerContext, OptimizerRule,
    },
    sql::{
        planner::{ContextProvider, SqlToRel},
        sqlparser::{dialect::GeneralDialect, parser::Parser},
    },
};
use proof_of_sql_planner::{visit_plan, PoSqlContextProvider, PoSqlTableSource};
use std::{any::Any, sync::Arc};

/// This example shows how to use DataFusion's SQL planner to parse SQL text and
/// build `LogicalPlan`s, convert them to `ProofPlan`s, and provably execute them.
pub fn main() -> Result<()> {
    let dialect = GeneralDialect {};
    let sql = "SELECT name FROM person WHERE age BETWEEN 21 AND 32";
    let statements = Parser::parse_sql(&dialect, sql)?;
    let context_provider = PoSqlContextProvider::default();
    let sql_to_rel = SqlToRel::new(&context_provider);
    let raw_plan = sql_to_rel.sql_statement_to_plan(statements[0].clone())?;
    let config = OptimizerContext::default().with_skip_failing_rules(false);
    let analyzed_plan =
        Analyzer::new().execute_and_check(raw_plan, config.options(), observe_analyzer)?;
    let optimized_plan = Optimizer::new().optimize(analyzed_plan, &config, observe_optimizer)?;
    let proof_plan = visit_plan(&optimized_plan)?;

    Ok(())
}

// Note that both the optimizer and the analyzer take a callback, called an
// "observer" that is invoked after each pass. We do not do anything with these
// callbacks in this example

fn observe_analyzer(_plan: &LogicalPlan, _rule: &dyn AnalyzerRule) {}
fn observe_optimizer(_plan: &LogicalPlan, _rule: &dyn OptimizerRule) {}
