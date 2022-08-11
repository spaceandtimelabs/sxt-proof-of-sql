use crate::{
    base::proof::{Commitment, PipVerify, ProofResult},
    pip::{
        execution_plans::{ReaderProof, TrivialProof},
        expressions::{ColumnProof, NegativeProof},
    },
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
pub enum ExecutionPlanProof {
    ReaderProof(ReaderProof),
    TrivialProof(TrivialProof),
}

impl ExecutionPlanProof {
    // Fill in the ExecutionPlan proofs
    pub fn get_output_commitments(&self) -> ProofResult<Vec<Commitment>> {
        match &self {
            ExecutionPlanProof::ReaderProof(p) => Ok(p.get_output_commitments()),
            ExecutionPlanProof::TrivialProof(p) => Ok(p.get_output_commitments()),
        }
    }
}

/// Provides general datafusion proofs
#[derive(Debug)]
pub enum DataFusionProof {
    PhysicalExprProof(PhysicalExprProof),
    ExecutionPlanProof(ExecutionPlanProof),
}
