use crate::base::database::{dataframe_to_record_batch, record_batch_to_dataframe};
use crate::sql::proof::TransformExpr;
use crate::sql::transform::DataFrameExpr;

use arrow::datatypes::{Field, Schema};
use arrow::record_batch::RecordBatch;
use dyn_partial_eq::DynPartialEq;
use polars::prelude::IntoLazy;
use proofs_sql::intermediate_ast::ResultColumn;
use std::sync::Arc;

/// The result expression is used to transform the results of a query
///
/// Note: both the `transformation` and `result_schema` are
/// mutually exclusive operations. So they must not be set at the same time.
#[derive(Default, Debug, DynPartialEq, PartialEq)]
pub struct ResultExpr {
    transformation: Option<Box<dyn DataFrameExpr>>,
    result_schema: Option<Vec<ResultColumn>>,
}

impl ResultExpr {
    /// Create a new `ResultExpr` node with the provided transformation to be applied to the input record batch.
    pub fn new_with_transformation(transformation: Box<dyn DataFrameExpr>) -> Self {
        Self {
            transformation: Some(transformation),
            result_schema: None,
        }
    }

    /// Create a new `ResultExpr` node with the provided result schema to be used to project the result record batch.
    pub fn new_with_result_schema(result_schema: Vec<ResultColumn>) -> Self {
        Self {
            transformation: None,
            result_schema: Some(result_schema),
        }
    }
}

impl TransformExpr for ResultExpr {
    /// Transform the `RecordBatch` result of a query using the `transformation` expression
    fn transform_results(&self, result_batch: RecordBatch) -> RecordBatch {
        if let Some(result_schema) = &self.result_schema {
            // We need to map the column names in result_batch to their respective aliases in the result schema.
            //
            // Note: using a `SelectExpr` transformation would achieve the same result,
            //       but it is non copy-free (i.e. inefficient) when there is no transformations.
            return project_record_batch_to_result_schema_and_apply_alias(
                result_batch,
                result_schema,
            );
        }

        if let Some(transformation) = &self.transformation {
            // We need to transform the result batch using the provided transformation expression
            //
            // Note: here we expect the `SelectExpr` to transform the result batch at some point.
            return transform_the_record_batch(result_batch, transformation.as_ref());
        }

        result_batch
    }
}

/// Transform the input `RecordBatch` using the provided `transformation` expression
fn transform_the_record_batch(
    result_batch: RecordBatch,
    transformation: &dyn DataFrameExpr,
) -> RecordBatch {
    let lazy_frame = record_batch_to_dataframe(result_batch).lazy();
    let lazy_frame = transformation.apply_transformation(lazy_frame);

    dataframe_to_record_batch(
        lazy_frame
            .collect()
            .expect("All transformations must have been validated"),
    )
}

/// Project the `RecordBatch` to the result schema and apply the result schema aliases to the columns
fn project_record_batch_to_result_schema_and_apply_alias(
    old_batch: RecordBatch,
    result_schema: &[ResultColumn],
) -> RecordBatch {
    let new_fields = result_schema
        .iter()
        .map(|col| {
            let schema = old_batch.schema();
            let col_name = col.name.as_str();
            let old_field = schema
                .field_with_name(col_name)
                .expect("Old schema must contain the new column name.");
            Field::new(
                col.alias.as_str(),
                old_field.data_type().clone(),
                old_field.is_nullable(),
            )
        })
        .collect::<Vec<_>>();

    let new_columns = result_schema
        .iter()
        .map(|col| {
            let col_name = col.name.as_str();
            old_batch
                .column_by_name(col_name)
                .expect("Old schema must contain the new column name.")
                .clone()
        })
        .collect::<Vec<_>>();

    let new_schema = Arc::new(Schema::new(new_fields));
    RecordBatch::try_new(new_schema, new_columns).expect("Failed to create new RecordBatch.")
}
