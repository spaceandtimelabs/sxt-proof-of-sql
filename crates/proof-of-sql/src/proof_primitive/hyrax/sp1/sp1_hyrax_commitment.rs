use super::sp1_hyrax_configuration::Sp1HyraxConfiguration;
use crate::proof_primitive::hyrax::base::hyrax_commitment::HyraxCommitment;

/// The commitment scheme we use to implement Hyrax for sp1.
pub type Sp1HyraxCommitment = HyraxCommitment<Sp1HyraxConfiguration>;
