/// Proof of SQL Postprocessing. Used when the last step of the logical plan is an unprovable projection.
mod error;
#[cfg(test)]
pub use error::{PostprocessingError, PostprocessingResult};
mod expression_evaluation;
#[cfg(test)]
pub(crate) use expression_evaluation::evaluate_expr;
mod expression_evaluation_error;
pub use expression_evaluation_error::{ExpressionEvaluationError, ExpressionEvaluationResult};
#[cfg(test)]
mod expression_evaluation_test;
