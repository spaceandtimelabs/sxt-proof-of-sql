/// Proof of SQL Postprocessing. Used when the last step of the logical plan is an unprovable projection.
mod error;
pub use error::{PostprocessingError, PostprocessingResult};
mod expression_evaluation;
pub(crate) use expression_evaluation::evaluate_expr;
mod expression_evaluation_error;
pub use expression_evaluation_error::{ExpressionEvaluationError, ExpressionEvaluationResult};
#[cfg(test)]
mod expression_evaluation_test;
mod postprocessing_step;
pub use postprocessing_step::PostprocessingStep;
mod select_postprocessing;
pub use select_postprocessing::SelectPostprocessing;
#[cfg(test)]
mod select_postprocessing_test;
