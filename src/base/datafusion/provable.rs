use super::DataFusionProof;
use crate::base::proof::{ProofError, ProofResult, Transcript};
use std::{fmt::Debug, sync::Arc};

/// Provides a connection between DataFusion structs and proofs
///
/// A Provable is a wrapped DF PhysicalExpr or ExecutionPlan
/// that can be proven and has provable children.

pub trait Provable: Sync + Send + Debug {
    fn children(&self) -> &[Arc<dyn Provable>];
    fn get_proof(&self) -> ProofResult<Arc<DataFusionProof>>;
    fn run_create_proof(&self, transcript: &mut Transcript) -> ProofResult<()>;
    fn run_verify(&self, transcript: &mut Transcript) -> ProofResult<()>;
    fn set_proof(&self, proof: &Arc<DataFusionProof>) -> ProofResult<()>;
    fn run_create_proof_with_children(&self, transcript: &mut Transcript) -> ProofResult<()> {
        for child in self.children() {
            child.run_create_proof_with_children(transcript)?;
        }
        self.run_create_proof(transcript)?;
        Ok(())
    }
    fn run_verify_with_children(&self, transcript: &mut Transcript) -> ProofResult<()> {
        for child in self.children() {
            child.run_verify_with_children(transcript)?;
        }
        self.run_verify(transcript)?;
        Ok(())
    }
    fn get_proof_with_children(&self) -> ProofResult<Vec<Arc<DataFusionProof>>> {
        let mut proofs: Vec<Arc<DataFusionProof>> = Vec::new();
        for child in self.children() {
            push_proof_with_children(child, &mut proofs)?;
        }
        proofs.push(self.get_proof()?);
        Ok(proofs)
    }
    fn set_proof_with_children(&self, proofs: &[Arc<DataFusionProof>]) -> ProofResult<()> {
        let mut proofs = proofs;
        for child in self.children() {
            proofs = load_proof_with_children(child, proofs)?;
        }
        if let Some(proof) = proofs.first() {
            self.set_proof(proof)?;
        } else {
            Err(ProofError::NoProofError)?;
        }
        proofs = &proofs[1..];
        if !proofs.is_empty() {
            Err(ProofError::VerificationError)
        } else {
            Ok(())
        }
    }
}

fn push_proof_with_children(
    provable: &Arc<dyn Provable>,
    proofs: &mut Vec<Arc<DataFusionProof>>,
) -> ProofResult<()> {
    for child in provable.children() {
        push_proof_with_children(child, proofs)?;
    }
    proofs.push(provable.get_proof()?);
    Ok(())
}

fn load_proof_with_children<'a>(
    provable: &Arc<dyn Provable>,
    proofs: &'a [Arc<DataFusionProof>],
) -> ProofResult<&'a [Arc<DataFusionProof>]> {
    let mut proofs = proofs;
    for child in provable.children() {
        proofs = load_proof_with_children(child, proofs)?;
    }
    if let Some(proof) = proofs.first() {
        provable.set_proof(proof)?;
    } else {
        Err(ProofError::NoProofError)?;
    }
    Ok(&proofs[1..])
}
