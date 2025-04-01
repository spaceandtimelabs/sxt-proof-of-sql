use super::{BNScalar, HyperKZGCommitment, HyperKZGEngine, HyperKZGPublicSetup};
use crate::{
    base::{commitment::CommitmentEvaluationProof, slice_ops},
    proof_primitive::hyperkzg::nova_commitment::NovaCommitment,
};
use ark_bn254::{G1Affine, G1Projective};
use blitzar;
use core::ops::Add;
use ff::Field;
use halo2curves::bn256::G2Affine;
use nova_snark::{
    errors::NovaError,
    provider::{
        bn256_grumpkin::bn256::{Affine, Scalar as NovaScalar},
        hyperkzg::{CommitmentKey, EvaluationArgument, EvaluationEngine, VerifierKey},
    },
    traits::evaluation::EvaluationEngineTrait,
};
use serde::{Deserialize, Serialize};
use tracing::{span, Level};

/// The evaluation proof for the `HyperKZG` PCS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HyperKZGCommitmentEvaluationProof {
    com: Vec<HyperKZGCommitment>,
    v: Vec<[BNScalar; 3]>,
    w: [HyperKZGCommitment; 3],
}

impl From<&HyperKZGCommitmentEvaluationProof> for EvaluationArgument<HyperKZGEngine> {
    fn from(value: &HyperKZGCommitmentEvaluationProof) -> Self {
        let nova_com = value.com.iter().map(Into::into).collect();
        let nova_w = value.w.map(Into::into);
        let nova_v = value.v.iter().map(|vj| vj.map(Into::into)).collect();
        EvaluationArgument::new(nova_com, nova_w, nova_v)
    }
}
impl From<EvaluationArgument<HyperKZGEngine>> for HyperKZGCommitmentEvaluationProof {
    fn from(value: EvaluationArgument<HyperKZGEngine>) -> Self {
        let com = value.com().iter().copied().map(Into::into).collect();
        let w = [0, 1, 2].map(|i| value.w()[i].into());
        let v = value.v().iter().map(|vj| vj.map(Into::into)).collect();
        Self { com, v, w }
    }
}

impl CommitmentEvaluationProof for HyperKZGCommitmentEvaluationProof {
    type Scalar = BNScalar;
    type Commitment = HyperKZGCommitment;
    type Error = NovaError;
    type ProverPublicSetup<'a> = HyperKZGPublicSetup<'a>;
    type VerifierPublicSetup<'a> = &'a VerifierKey<HyperKZGEngine>;

    fn new(
        transcript: &mut impl crate::base::proof::Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        setup: &Self::ProverPublicSetup<'_>,
    ) -> Self {
        assert_eq!(generators_offset, 0);
        let mut nova_point = slice_ops::slice_cast(b_point);
        nova_point.reverse();
        if nova_point.is_empty() {
            nova_point.push(NovaScalar::ZERO);
        }
        let mut nova_a = slice_ops::slice_cast(a);
        nova_a.extend(itertools::repeat_n(
            NovaScalar::ZERO,
            (1 << nova_point.len()) - nova_a.len(),
        ));
        let nova_ck: CommitmentKey<HyperKZGEngine> = CommitmentKey::new(
            slice_ops::slice_cast_with(setup, blitzar::compute::convert_to_halo2_bn256_g1_affine),
            Affine::default(),   // I'm pretty sure this is unused in the proof
            G2Affine::default(), // I'm pretty sure this is unused in the proof
        );
        transcript
            .wrap_transcript(|keccak_transcript| {
                let span = span!(Level::DEBUG, "EvaluationEngine::prove").entered();
                let eval_eng = EvaluationEngine::prove(
                    &nova_ck,
                    &EvaluationEngine::setup(&nova_ck).0, // This parameter is unused
                    keccak_transcript,
                    &NovaCommitment::default(), // This parameter is unused
                    &nova_a,
                    &nova_point,
                    &NovaScalar::default(), // This parameter is unused
                )
                .unwrap();
                span.exit();
                eval_eng
            })
            .into()
    }

    fn verify_batched_proof(
        &self,
        transcript: &mut impl crate::base::proof::Transcript,
        commit_batch: &[Self::Commitment],
        batching_factors: &[Self::Scalar],
        evaluations: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _table_length: usize,
        setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        if generators_offset != 0 {
            Err(NovaError::InvalidPCS)?;
        }
        let commit: G1Affine = commit_batch
            .iter()
            .zip(batching_factors)
            .map(|(c, m)| c.commitment * m.0)
            .fold(G1Projective::default(), Add::add)
            .into();
        let nova_commit = nova_snark::provider::hyperkzg::Commitment::new(
            blitzar::compute::convert_to_halo2_bn256_g1_affine(&commit).into(),
        );
        let nova_eval = evaluations
            .iter()
            .zip(batching_factors)
            .map(|(&e, &f)| e * f)
            .sum::<Self::Scalar>();
        let mut nova_point = slice_ops::slice_cast(b_point);
        nova_point.reverse();
        if nova_point.is_empty() {
            nova_point.push(NovaScalar::ZERO);
        }
        transcript.wrap_transcript(|keccak_transcript| {
            EvaluationEngine::verify(
                setup,
                keccak_transcript,
                &nova_commit,
                &nova_point,
                &nova_eval.into(),
                &self.into(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::commitment::commitment_evaluation_proof_test::{
            test_commitment_evaluation_proof_with_length_1,
            test_random_commitment_evaluation_proof, test_simple_commitment_evaluation_proof,
        },
        proof_primitive::hyperkzg::{
            nova_commitment_key_to_hyperkzg_public_setup,
            public_setup::load_small_setup_for_testing,
        },
    };
    use nova_snark::{
        provider::hyperkzg::CommitmentEngine, traits::commitment::CommitmentEngineTrait,
    };

    #[test]
    fn we_can_create_small_hyperkzg_evaluation_proofs() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 32);
        let (_, vk) = EvaluationEngine::setup(&ck);
        test_simple_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_commitment_evaluation_proof_with_length_1::<HyperKZGCommitmentEvaluationProof>(
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
    }

    #[test]
    fn we_can_create_hyperkzg_evaluation_proofs_with_various_lengths() {
        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 128);
        let (_, vk) = EvaluationEngine::setup(&ck);
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            2,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            3,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            4,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            5,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            8,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            10,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            16,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            20,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            32,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            50,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            64,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            100,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            128,
            0,
            &&nova_commitment_key_to_hyperkzg_public_setup(&ck)[..],
            &&vk,
        );
    }

    #[test]
    fn we_create_hyperkzg_proof_using_setup_from_file() {
        let (pk, vk) = load_small_setup_for_testing();
        test_random_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            23,
            0,
            &&pk[..],
            &&vk,
        );
    }
}
