use crate::proof_primitive::hyrax::base::hyrax_public_setup::HyraxPublicSetup;
use curve25519_dalek::RistrettoPoint;

pub type TestHyraxPublicSetup<'a> = HyraxPublicSetup<'a, RistrettoPoint>;
