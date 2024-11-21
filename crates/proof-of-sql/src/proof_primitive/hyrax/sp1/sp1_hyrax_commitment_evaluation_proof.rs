use super::sp1_hyrax_configuration::Sp1HyraxConfiguration;
use crate::proof_primitive::hyrax::base::hyrax_commitment_evaluation_proof::HyraxCommitmentEvaluationProof;

/// The evaluation proof scheme we use to implement Hyrax for sp1.
pub type Sp1HyraxCommitmentEvaluationProof = HyraxCommitmentEvaluationProof<Sp1HyraxConfiguration>;
