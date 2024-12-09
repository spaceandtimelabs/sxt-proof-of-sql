use crate::{
    base::{
        database::{
            ColumnField, ColumnRef, OwnedTable, Table, TableEvaluation, TableOptions, TableRef,
        },
        map::{IndexMap, IndexSet},
        proof::ProofError,
        scalar::Scalar,
    },
    sql::{
        proof::{
            FinalRoundBuilder, FirstRoundBuilder, ProofPlan, ProverEvaluate, VerificationBuilder,
        },
        proof_exprs::{AliasedDynProofExpr, ProofExpr, TableExpr},
    },
    utils::log,
};
use alloc::vec::Vec;
use bumpalo::Bump;
use serde::{Deserialize, Serialize};

/// Provable expressions for queries of the form
/// ```ignore
///     SELECT <result_expr1>, ..., <result_exprN> FROM <table>
/// ```
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectionExec {
    pub(super) aliased_results: Vec<AliasedDynProofExpr>,
    pub(super) table: TableExpr,
}

impl ProjectionExec {
    /// Creates a new projection expression.
    pub fn new(aliased_results: Vec<AliasedDynProofExpr>, table: TableExpr) -> Self {
        Self {
            aliased_results,
            table,
        }
    }
}

impl ProofPlan for ProjectionExec {
    #[allow(unused_variables)]
    fn verifier_evaluate<S: Scalar>(
        &self,
        builder: &mut VerificationBuilder<S>,
        accessor: &IndexMap<ColumnRef, S>,
        _result: Option<&OwnedTable<S>>,
        one_eval_map: &IndexMap<TableRef, S>,
    ) -> Result<TableEvaluation<S>, ProofError> {
        // For projections input and output have the same length and hence the same one eval
        let one_eval = *one_eval_map
            .get(&self.table.table_ref)
            .expect("One eval not found");
        self.aliased_results
            .iter()
            .map(|aliased_expr| {
                aliased_expr
                    .expr
                    .verifier_evaluate(builder, accessor, one_eval)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let column_evals = builder.try_consume_mle_evaluations(self.aliased_results.len())?;
        Ok(TableEvaluation::new(column_evals, one_eval))
    }

    fn get_column_result_fields(&self) -> Vec<ColumnField> {
        self.aliased_results
            .iter()
            .map(|aliased_expr| ColumnField::new(aliased_expr.alias, aliased_expr.expr.data_type()))
            .collect()
    }

    fn get_column_references(&self) -> IndexSet<ColumnRef> {
        let mut columns = IndexSet::default();
        self.aliased_results.iter().for_each(|aliased_expr| {
            aliased_expr.expr.get_column_references(&mut columns);
        });
        columns
    }

    fn get_table_references(&self) -> IndexSet<TableRef> {
        IndexSet::from_iter([self.table.table_ref])
    }
}

impl ProverEvaluate for ProjectionExec {
    #[tracing::instrument(
        name = "ProjectionExec::first_round_evaluate",
        level = "debug",
        skip_all
    )]
    fn first_round_evaluate<'a, S: Scalar>(
        &self,
        _builder: &mut FirstRoundBuilder,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results.iter().map(|aliased_expr| {
                (
                    aliased_expr.alias,
                    aliased_expr.expr.result_evaluate(alloc, table),
                )
            }),
            TableOptions::new(Some(table.num_rows())),
        )
        .expect("Failed to create table from iterator");

        log::log_memory_usage("End");

        res
    }

    #[tracing::instrument(
        name = "ProjectionExec::final_round_evaluate",
        level = "debug",
        skip_all
    )]
    #[allow(unused_variables)]
    fn final_round_evaluate<'a, S: Scalar>(
        &self,
        builder: &mut FinalRoundBuilder<'a, S>,
        alloc: &'a Bump,
        table_map: &IndexMap<TableRef, Table<'a, S>>,
    ) -> Table<'a, S> {
        log::log_memory_usage("Start");

        let table = table_map
            .get(&self.table.table_ref)
            .expect("Table not found");
        // 1. Evaluate result expressions
        let res = Table::<'a, S>::try_from_iter_with_options(
            self.aliased_results.iter().map(|aliased_expr| {
                (
                    aliased_expr.alias,
                    aliased_expr.expr.prover_evaluate(builder, alloc, table),
                )
            }),
            TableOptions::new(Some(table.num_rows())),
        )
        .expect("Failed to create table from iterator");
        // 2. Produce MLEs
        for column in res.columns().copied() {
            builder.produce_intermediate_mle(column);
        }

        log::log_memory_usage("End");

        res
    }
}
