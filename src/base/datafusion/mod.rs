mod wrappers;
pub use wrappers::{collect, ProvableAggregateExpr, ProvableExecutionPlan, ProvablePhysicalExpr};
mod proof;
pub use proof::{AggregateExprProof, DataFusionProof, ExecutionPlanProof, PhysicalExprProof};
mod provable;
pub use provable::Provable;

// Shortcuts for datafusion integration to reduce repetition.
pub(crate) use provable::impl_provable;
pub(crate) use wrappers::{
    impl_aggregate_expr_for_provable, impl_debug_display_for_phys_expr_wrapper,
    impl_debug_for_provable, impl_execution_plan_for_provable, impl_physical_expr_for_provable,
    PhysicalExprTuple, ProvablePhysicalExprTuple,
};
