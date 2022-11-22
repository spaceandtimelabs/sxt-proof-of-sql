use std::fmt::Debug;

/// Counters for different terms used within a proof
#[derive(Default, Debug, Clone, Copy)]
pub struct ProofCounts {
    pub table_length: usize,
    pub sumcheck_variables: usize,
    pub sumcheck_max_multiplicands: usize,
    pub result_columns: usize,
    pub anchored_mles: usize,
    pub intermediate_mles: usize,
    pub sumcheck_subpolynomials: usize,
}

impl ProofCounts {
    #[tracing::instrument(
        name = "proofs.sql.proof.proof_acounts.annotate_trace",
        level = "info",
        skip_all
    )]
    pub fn annotate_trace(&self) {
        tracing::info!("table_length = {:?}", self.table_length);
        tracing::info!("sumcheck_variables = {:?}", self.sumcheck_variables);
        tracing::info!(
            "sumcheck_max_multiplicands = {:?}",
            self.sumcheck_max_multiplicands
        );
        tracing::info!("result_columns = {:?}", self.result_columns);
        tracing::info!("anchored_mles = {:?}", self.anchored_mles);
        tracing::info!("intermediate_mles = {:?}", self.intermediate_mles);
        tracing::info!(
            "sumcheck_subpolynomials = {:?}",
            self.sumcheck_subpolynomials
        );
    }
}
