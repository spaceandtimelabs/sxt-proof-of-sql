use crate::{
    base::{
        datafusion::Provable,
        proof::{IntoProofResult, ProofResult},
    },
    datafusion_integration::CoalescePartitionsExecWrapper,
};
use async_trait::async_trait;
use datafusion::{
    arrow::{array::ArrayRef, record_batch::RecordBatch},
    execution::context::TaskContext,
    physical_expr::{AggregateExpr, PhysicalExpr},
    physical_plan::{ColumnarValue, ExecutionPlan},
};
use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

pub(crate) type PhysicalExprTuple = (Arc<dyn PhysicalExpr>, String);
pub(crate) type ProvablePhysicalExprTuple = (Arc<dyn ProvablePhysicalExpr>, String);

/// This file contains provable versions of important DataFusion traits we have adapte
/// as well as some important functions related to such traits.

/// Since provables usually copies the underlying raw DataFusion structs in terms of behavior
/// for issues unrelated to crypto proofs a lot of required methods often have identical implementations
/// that essentially simply calls them on the underlying raw structs. Due to Rust not having fields
/// in traits we have to implement them for every struct which is quite repetitive. Hence we use macros
/// for that.

/// Make sure to manually implement evaluate
macro_rules! impl_physical_expr_for_provable {
    () => {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn data_type(&self, input_schema: &Schema) -> datafusion::common::Result<DataType> {
            self.raw.data_type(input_schema)
        }
        fn nullable(&self, input_schema: &Schema) -> datafusion::common::Result<bool> {
            self.raw.nullable(input_schema)
        }
    };
}

pub trait ProvablePhysicalExpr: PhysicalExpr + Provable + Debug + Display {
    // Return the raw expression
    fn try_raw(&self) -> ProofResult<Arc<dyn PhysicalExpr>>;
    // Set num of rows to convert ScalarValues into ArrayRefs
    fn set_num_rows(&self, num_rows: usize) -> ProofResult<()>;
    // Output of a physical expression as ArrayRef
    fn array_output(&self) -> ProofResult<ArrayRef>;
}

/// Make sure to manually implement expressions
macro_rules! impl_aggregate_expr_for_provable {
    () => {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn field(&self) -> datafusion::error::Result<Field> {
            self.raw.field()
        }

        fn create_accumulator(&self) -> datafusion::error::Result<Box<dyn Accumulator>> {
            self.raw.create_accumulator()
        }

        fn state_fields(&self) -> datafusion::error::Result<Vec<Field>> {
            self.raw.state_fields()
        }

        fn name(&self) -> &str {
            self.raw.name()
        }

        fn row_accumulator_supported(&self) -> bool {
            self.raw.row_accumulator_supported()
        }

        fn create_row_accumulator(
            &self,
            start_index: usize,
        ) -> datafusion::error::Result<Box<dyn RowAccumulator>> {
            self.raw.create_row_accumulator(start_index)
        }
    };
}

pub trait ProvableAggregateExpr: AggregateExpr + Provable + Debug {
    // Return the raw expression
    fn try_raw(&self) -> ProofResult<Arc<dyn AggregateExpr>>;
    // Pass output from AggregateExec to ProvableAggregateExpr
    fn set_output(&self, output: &ColumnarValue) -> ProofResult<()>;
    // Pass input num of rows to underlying physical exprs
    fn evaluate_and_set_num_rows_for_physicals(&self, input: &RecordBatch) -> ProofResult<()>;
    // Output of an aggregate expression as ScalarValue
    fn array_output(&self) -> ProofResult<ArrayRef>;
}

/// Make sure to manually implement output_ordering, children and with_new_children
macro_rules! impl_execution_plan_for_provable {
    () => {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn schema(&self) -> SchemaRef {
            self.raw.schema()
        }

        fn output_partitioning(&self) -> Partitioning {
            self.raw.output_partitioning()
        }

        fn required_child_distribution(&self) -> Distribution {
            self.raw.required_child_distribution()
        }

        fn relies_on_input_order(&self) -> bool {
            self.raw.relies_on_input_order()
        }

        fn maintains_input_order(&self) -> bool {
            self.raw.maintains_input_order()
        }

        fn benefits_from_input_partitioning(&self) -> bool {
            self.raw.benefits_from_input_partitioning()
        }

        fn execute(
            &self,
            partition: usize,
            context: Arc<TaskContext>,
        ) -> datafusion::error::Result<SendableRecordBatchStream> {
            self.raw.execute(partition, context)
        }

        fn metrics(&self) -> Option<MetricsSet> {
            self.raw.metrics()
        }

        fn fmt_as(&self, t: DisplayFormatType, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            self.raw.fmt_as(t, f)
        }

        fn statistics(&self) -> Statistics {
            self.raw.statistics()
        }
    };
}

#[async_trait]
pub trait ProvableExecutionPlan: ExecutionPlan + Provable + Debug {
    // Return the raw plan
    fn try_raw(&self) -> ProofResult<Arc<dyn ExecutionPlan>>;
    // Compute output of an execution plan and store it
    async fn execute_and_collect(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> ProofResult<()>;
    // Return output of an execution plan
    fn output(&self) -> ProofResult<RecordBatch>;
}

/// Execute the [ProvableExecutionPlan], coalesce results into one partition
/// and collect them in memory
pub async fn collect(
    plan: &Arc<dyn ProvableExecutionPlan>,
    context: Arc<TaskContext>,
) -> ProofResult<RecordBatch> {
    match (*plan).output_partitioning().partition_count() {
        0 => RecordBatch::try_new((*plan).schema(), vec![]).into_proof_result(),
        1 => {
            (*plan).execute_and_collect(0, context).await?;
            (*plan).output()
        }
        _ => {
            // merge into a single partition
            let new_plan = CoalescePartitionsExecWrapper::try_new_from_children((*plan).clone())?;
            // CoalescePartitionsExecWrapper must produce a single partition
            assert_eq!(1, new_plan.output_partitioning().partition_count());
            new_plan.execute_and_collect(0, context).await?;
            new_plan.output()
        }
    }
}

// Debug and display

macro_rules! impl_debug_display_for_phys_expr_wrapper {
    ($provable:ty) => {
        impl Display for $provable {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.raw, f)
            }
        }
        impl Debug for $provable {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($provable))
                    .field("raw", &self.raw)
                    .finish()
            }
        }
    };
}

macro_rules! impl_debug_for_provable {
    ($provable: ty) => {
        impl Debug for $provable {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($provable))
                    .field("raw", &self.raw)
                    .finish()
            }
        }
    };
}

pub(crate) use impl_aggregate_expr_for_provable;
pub(crate) use impl_debug_display_for_phys_expr_wrapper;
pub(crate) use impl_debug_for_provable;
pub(crate) use impl_execution_plan_for_provable;
pub(crate) use impl_physical_expr_for_provable;
