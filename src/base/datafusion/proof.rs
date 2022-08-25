use crate::{
    base::proof::{Commitment, PipVerify, ProofResult},
    pip::{
        addition::AdditionProof,
        equality::EqualityProof,
        execution_plans::{ReaderProof, TrivialProof},
        expressions::{ColumnProof, NegativeProof},
        inequality::InequalityProof,
        or::OrProof,
        subtraction::SubtractionProof,
    },
};

#[derive(Debug)]
pub enum PhysicalExprProof {
    ColumnProof(ColumnProof),
    NegativeProof(NegativeProof),
    EqualityProof(EqualityProof),
    InequalityProof(InequalityProof),
    OrProof(OrProof),
    AdditionProof(AdditionProof),
    SubtractionProof(SubtractionProof),
}

impl PhysicalExprProof {
    pub fn get_output_commitments(&self) -> ProofResult<Commitment> {
        match &self {
            PhysicalExprProof::NegativeProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::ColumnProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::EqualityProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::InequalityProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::OrProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::AdditionProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::SubtractionProof(p) => Ok(p.get_output_commitments()),
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
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum DataFusionProof {
    PhysicalExprProof(PhysicalExprProof),
    ExecutionPlanProof(ExecutionPlanProof),
}
