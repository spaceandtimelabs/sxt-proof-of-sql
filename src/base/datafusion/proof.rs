use crate::{
    base::proof::{Commitment, PipVerify, ProofError, ProofResult},
    pip::expressions::{ColumnProof, NegativeProof},
};

#[derive(Debug)]
pub enum PhysicalExprProof {
    ColumnProof(ColumnProof),
    NegativeProof(NegativeProof),
}

impl PhysicalExprProof {
    pub fn get_output_commitments(&self) -> ProofResult<Commitment> {
        match &self {
            PhysicalExprProof::NegativeProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::ColumnProof(p) => Ok(p.get_output_commitments()),
        }
    }
}

/// Here is where Proj and Filter proofs go
#[derive(Debug)]
pub enum ExecutionPlanProof {}

impl ExecutionPlanProof {
    // Fill in the ExecutionPlan proofs
    pub fn get_output_commitments(&self) -> ProofResult<Vec<Commitment>> {
        Err(ProofError::TypeError)
    }
}

/// Provides general datafusion proofs
#[derive(Debug)]
pub enum DataFusionProof {
    PhysicalExprProof(PhysicalExprProof),
    ExecutionPlanProof(ExecutionPlanProof),
}
