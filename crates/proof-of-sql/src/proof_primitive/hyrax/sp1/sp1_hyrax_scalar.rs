use super::sp1_hyrax_configuration::Sp1HyraxConfiguration;
use crate::proof_primitive::hyrax::base::{
    hyrax_configuration::HyraxConfiguration, hyrax_scalar::HyraxScalarWrapper,
};

/// A wrapper for the underlying Scalar used for a given Hyrax implementation
pub type Sp1HyraxScalar =
    HyraxScalarWrapper<<Sp1HyraxConfiguration as HyraxConfiguration>::OperableScalar>;
