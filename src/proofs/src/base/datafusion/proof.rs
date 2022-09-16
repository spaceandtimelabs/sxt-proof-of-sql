use crate::{
    base::proof::{Commitment, PipVerify, ProofResult},
    pip::{
        addition::AdditionProof,
        aggregate_expr::CountProof,
        equality::EqualityProof,
        execution_plan::{ReaderProof, TrivialProof},
        inequality::InequalityProof,
        multiplication::MultiplicationProof,
        or::OrProof,
        physical_expr::{ColumnProof, LiteralProof, NegativeProof},
        subtraction::SubtractionProof,
    },
};

#[derive(Debug)]
pub enum AggregateExprProof {
    CountProof(CountProof),
}

impl AggregateExprProof {
    pub fn get_output_commitments(&self) -> ProofResult<Commitment> {
        match &self {
            AggregateExprProof::CountProof(p) => Ok(p.get_output_commitments()),
        }
    }
}

#[derive(Debug)]
pub enum PhysicalExprProof {
    ColumnProof(ColumnProof),
    LiteralProof(LiteralProof),
    NegativeProof(NegativeProof),
    EqualityProof(EqualityProof),
    InequalityProof(InequalityProof),
    OrProof(OrProof),
    AdditionProof(AdditionProof),
    SubtractionProof(SubtractionProof),
    MultiplicationProof(MultiplicationProof),
}

impl PhysicalExprProof {
    pub fn get_output_commitments(&self) -> ProofResult<Commitment> {
        match &self {
            PhysicalExprProof::ColumnProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::LiteralProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::NegativeProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::EqualityProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::InequalityProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::OrProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::AdditionProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::SubtractionProof(p) => Ok(p.get_output_commitments()),
            PhysicalExprProof::MultiplicationProof(p) => Ok(p.get_output_commitments()),
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
    AggregateExprProof(AggregateExprProof),
    PhysicalExprProof(PhysicalExprProof),
    ExecutionPlanProof(ExecutionPlanProof),
}
