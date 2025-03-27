use super::{BNScalar, HyperKZGPublicSetupOwned};
use crate::base::{
    proof::{Keccak256Transcript, Transcript},
    slice_ops,
};
use nova_snark::{
    errors::NovaError,
    provider::{bn256_grumpkin::bn256::Scalar as NovaScalar, hyperkzg::CommitmentKey},
    traits::{Engine, TranscriptEngineTrait, TranscriptReprTrait},
};
use serde::{Deserialize, Serialize};

/// The `HyperKZG` engine that implements nova's `Engine` trait.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HyperKZGEngine;

impl Engine for HyperKZGEngine {
    type Base = nova_snark::provider::bn256_grumpkin::bn256::Base;
    type Scalar = NovaScalar;
    type GE = nova_snark::provider::bn256_grumpkin::bn256::Point;
    type RO = nova_snark::provider::poseidon::PoseidonRO<Self::Base>;
    type ROCircuit = nova_snark::provider::poseidon::PoseidonROCircuit<Self::Base>;
    type RO2 = nova_snark::provider::poseidon::PoseidonRO<Self::Scalar>;
    type RO2Circuit = nova_snark::provider::poseidon::PoseidonROCircuit<Self::Scalar>;
    type TE = Keccak256Transcript;
    type CE = nova_snark::provider::hyperkzg::CommitmentEngine<Self>;
}

impl TranscriptEngineTrait<HyperKZGEngine> for Keccak256Transcript {
    fn new(_label: &'static [u8]) -> Self {
        Transcript::new()
    }

    fn squeeze(&mut self, _label: &'static [u8]) -> Result<NovaScalar, NovaError> {
        Ok(Transcript::scalar_challenge_as_be::<BNScalar>(self).into())
    }

    fn absorb<T: TranscriptReprTrait<<HyperKZGEngine as Engine>::GE>>(
        &mut self,
        _label: &'static [u8],
        o: &T,
    ) {
        Transcript::extend_as_le_from_refs(
            self,
            o.to_transcript_bytes()
                .chunks(32)
                // Reverse the bytes in each 32 byte chunk, making them effectivelly big-endian
                .flat_map(|chunk| chunk.iter().rev()),
        );
    }

    fn dom_sep(&mut self, _bytes: &'static [u8]) {}
}

/// Utility converting a nova `CommitmentKey` to a [`HyperKZGPublicSetupOwned`].
pub fn nova_commitment_key_to_hyperkzg_public_setup(
    setup: &CommitmentKey<HyperKZGEngine>,
) -> HyperKZGPublicSetupOwned {
    slice_ops::slice_cast_with(setup.ck(), blitzar::compute::convert_to_ark_bn254_g1_affine)
}
