use crate::proof_primitive::hyrax::base::hyrax_public_setup::HyraxPublicSetup;
use curve25519_dalek::EdwardsPoint;

/// The public setup used for sp1's Hyrax implementation
pub type Sp1HyraxPublicSetup<'a> = HyraxPublicSetup<'a, EdwardsPoint>;
