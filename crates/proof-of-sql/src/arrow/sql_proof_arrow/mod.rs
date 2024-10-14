/// Module for query results
pub mod query_result;
mod final_round_builder_test;
#[cfg(all(test, feature = "arrow"))]
mod provable_query_result_test;