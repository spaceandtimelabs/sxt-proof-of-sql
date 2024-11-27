use super::{
    dynamic_build_vmv_state::{build_dynamic_vmv_prover_state, build_dynamic_vmv_verifier_state},
    dynamic_dory_helper::{compute_dynamic_T_vec_prime, compute_dynamic_nu, fold_dynamic_tensors},
    eval_vmv_re_prove, eval_vmv_re_verify, extended_dory_inner_product_prove,
    extended_dory_inner_product_verify, DeferredGT, DoryMessages, DoryScalar,
    DynamicDoryCommitment, ProverSetup, VerifierSetup, F,
};
use crate::base::{commitment::CommitmentEvaluationProof, proof::Transcript};
use serde::{Deserialize, Serialize};
use snafu::Snafu;

/// The `CommitmentEvaluationProof` for the Dory PCS.
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct DynamicDoryEvaluationProof(pub(super) DoryMessages);

/// The error type for the Dory PCS.
#[derive(Snafu, Debug)]
pub enum DoryError {
    /// This error occurs when the generators offset is invalid.
    #[snafu(display("invalid generators offset: {offset}"))]
    InvalidGeneratorsOffset { offset: u64 },
    /// This error occurs when the proof fails to verify.
    #[snafu(display("verification error"))]
    VerificationError,
    /// This error occurs when the setup is too small.
    #[snafu(display("setup is too small: the setup is {actual}, but the proof requires a setup of size {required}"))]
    SmallSetup { actual: usize, required: usize },
}

impl CommitmentEvaluationProof for DynamicDoryEvaluationProof {
    type Scalar = DoryScalar;
    type Commitment = DynamicDoryCommitment;
    type Error = DoryError;
    type ProverPublicSetup<'a> = &'a ProverSetup<'a>;
    type VerifierPublicSetup<'a> = &'a VerifierSetup;

    #[tracing::instrument(name = "DoryEvaluationProof::new", level = "debug", skip_all)]
    fn new(
        transcript: &mut impl Transcript,
        a: &[Self::Scalar],
        b_point: &[Self::Scalar],
        generators_offset: u64,
        setup: &Self::ProverPublicSetup<'_>,
    ) -> Self {
        // Dory PCS Logic
        if generators_offset != 0 {
            // TODO: support offsets other than 0.
            // Note: this will always result in a verification error.
            return DynamicDoryEvaluationProof::default();
        }
        let a: &[F] = bytemuck::TransparentWrapper::peel_slice(a);
        let b_point: &[F] = bytemuck::TransparentWrapper::peel_slice(b_point);
        let nu = compute_dynamic_nu(b_point.len());
        if nu > setup.max_nu {
            return DynamicDoryEvaluationProof::default(); // Note: this will always result in a verification error.
        }
        let T_vec_prime = compute_dynamic_T_vec_prime(a, nu, setup);
        let state = build_dynamic_vmv_prover_state(a, b_point, T_vec_prime, nu);

        let mut messages = DoryMessages::default();
        let extended_state = eval_vmv_re_prove(&mut messages, transcript, state, setup);
        extended_dory_inner_product_prove(&mut messages, transcript, extended_state, setup);
        Self(messages)
    }

    #[tracing::instrument(
        name = "DoryEvaluationProof::verify_batched_proof",
        level = "debug",
        skip_all
    )]
    fn verify_batched_proof(
        &self,
        transcript: &mut impl Transcript,
        commit_batch: &[Self::Commitment],
        batching_factors: &[Self::Scalar],
        product: &Self::Scalar,
        b_point: &[Self::Scalar],
        generators_offset: u64,
        _table_length: usize,
        setup: &Self::VerifierPublicSetup<'_>,
    ) -> Result<(), Self::Error> {
        let a_commit = DeferredGT::new(
            commit_batch.iter().map(|c| c.0),
            batching_factors.iter().map(|f| f.0),
        );
        // Dory PCS Logic
        if generators_offset != 0 {
            return Err(DoryError::InvalidGeneratorsOffset {
                offset: generators_offset,
            });
        }
        let b_point: &[F] = bytemuck::TransparentWrapper::peel_slice(b_point);
        let mut messages = self.0.clone();
        let nu = compute_dynamic_nu(b_point.len());
        if nu > setup.max_nu {
            return Err(DoryError::SmallSetup {
                actual: setup.max_nu,
                required: nu,
            });
        }
        let state = build_dynamic_vmv_verifier_state(product.0, b_point, a_commit, nu);
        let extended_state = eval_vmv_re_verify(&mut messages, transcript, state, setup)
            .ok_or(DoryError::VerificationError)?;
        if !extended_dory_inner_product_verify(
            &mut messages,
            transcript,
            extended_state,
            setup,
            fold_dynamic_tensors,
        ) {
            Err(DoryError::VerificationError)?;
        }
        Ok(())
    }
}
