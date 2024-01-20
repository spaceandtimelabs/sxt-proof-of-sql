use super::{BoolExprPlan, FilterExpr};
use crate::{
    base::database::{RecordBatchTestAccessor, TableRef},
    sql::{
        ast::test_utility::{cols_result, tab},
        proof::{exercise_verification, VerifiableQueryResult},
    },
};
use arrow::record_batch::RecordBatch;
use polars::prelude::{Expr, *};

pub struct TestExprNode {
    pub table_ref: TableRef,
    pub results: Vec<Expr>,
    pub ast: FilterExpr,
    pub accessor: RecordBatchTestAccessor,
    pub df_filter: Expr,
}

impl TestExprNode {
    pub fn new(
        table_ref: TableRef,
        results: &[&str],
        filter_expr: BoolExprPlan,
        df_filter: Expr,
        accessor: RecordBatchTestAccessor,
    ) -> Self {
        let polar_results = results
            .iter()
            .map(|v| polars::prelude::col(v))
            .collect::<Vec<_>>();
        let ast = FilterExpr::new(
            cols_result(table_ref, results, &accessor),
            tab(table_ref),
            filter_expr,
        );

        Self {
            table_ref,
            df_filter,
            results: polar_results,
            ast,
            accessor,
        }
    }

    pub fn create_verifiable_result(&self) -> VerifiableQueryResult {
        VerifiableQueryResult::new(&self.ast, &self.accessor)
    }

    pub fn verify_expr(&self) -> RecordBatch {
        let res = VerifiableQueryResult::new(&self.ast, &self.accessor);
        exercise_verification(&res, &self.ast, &self.accessor, self.table_ref);
        res.verify(&self.ast, &self.accessor)
            .unwrap()
            .into_record_batch()
    }

    pub fn query_table(&self) -> RecordBatch {
        self.accessor.query_table(self.table_ref, |df| {
            df.clone()
                .lazy()
                .filter(self.df_filter.clone())
                .select(&self.results[..])
                .collect()
                .unwrap()
        })
    }
}
