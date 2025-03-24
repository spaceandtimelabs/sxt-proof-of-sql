use super::{BNScalar, HyperKZGCommitment, HyperKZGEngine, HyperKZGPublicSetup};
use crate::{
    base::{commitment::CommitmentEvaluationProof, slice_ops},
    proof_primitive::hyperkzg::nova_commitment::NovaCommitment,
};
use ark_bn254::{G1Affine, G1Projective};
use ark_ec::AffineRepr as _;
use blitzar;
use core::ops::Add;
use ff::{Field, PrimeField as _};
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

#[expect(clippy::doc_markdown)]
/// Represents a commitment evaluation proof using the HyperKZG protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    from = "SerializableHyperKZGCommitmentEvaluationProof",
    into = "SerializableHyperKZGCommitmentEvaluationProof"
)]
pub struct HyperKZGCommitmentEvaluationProof {
    inner: EvaluationArgument<HyperKZGEngine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializableHyperKZGCommitmentEvaluationProof {
    com: Vec<HyperKZGCommitment>,
    v: Vec<[BNScalar; 3]>,
    w: [HyperKZGCommitment; 3],
}

impl From<SerializableHyperKZGCommitmentEvaluationProof> for HyperKZGCommitmentEvaluationProof {
    fn from(value: SerializableHyperKZGCommitmentEvaluationProof) -> Self {
        let SerializableHyperKZGCommitmentEvaluationProof { com, w, v } = value;
        let nova_com = com.into_iter().map(Into::into).collect();
        let nova_w = w.map(Into::into);
        let (nova_v0, nova_v1, nova_v2) = itertools::multiunzip(v.into_iter().map(|[a, b, c]| {
            (
                Into::<nova_snark::provider::bn256_grumpkin::bn256::Scalar>::into(a),
                Into::<nova_snark::provider::bn256_grumpkin::bn256::Scalar>::into(b),
                Into::<nova_snark::provider::bn256_grumpkin::bn256::Scalar>::into(c),
            )
        }));
        Self {
            inner: EvaluationArgument::new(nova_com, nova_w, [nova_v0, nova_v1, nova_v2]),
        }
    }
}
impl From<HyperKZGCommitmentEvaluationProof> for SerializableHyperKZGCommitmentEvaluationProof {
    fn from(value: HyperKZGCommitmentEvaluationProof) -> Self {
        let HyperKZGCommitmentEvaluationProof { inner } = value;
        let com = inner.com().iter().copied().map(Into::into).collect();
        let w = [0, 1, 2].map(|i| inner.w()[i].into());
        let v = itertools::izip!(
            inner.v()[0].iter(),
            inner.v()[1].iter(),
            inner.v()[2].iter(),
        )
        .map(|(a, b, c)| {
            [
                Into::<BNScalar>::into(a),
                Into::<BNScalar>::into(b),
                Into::<BNScalar>::into(c),
            ]
        })
        .collect();
        Self { com, w, v }
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
        Self {
            inner: transcript.wrap_transcript(|keccak_transcript| {
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
            }),
        }
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
                &self.inner,
            )
        })
    }
}

fn load_setups() -> (
    super::HyperKZGPublicSetupOwned,
    nova_snark::provider::hyperkzg::VerifierKey<HyperKZGEngine>,
) {
    let h: halo2curves::bn256::G1Affine = halo2curves::bn256::G1Affine::generator();
    let tau_H: halo2curves::bn256::G2Affine = halo2curves::bn256::G2Affine {
        x: halo2curves::bn256::Fq2::new(
            halo2curves::bn256::Fq::from_str_vartime(
                "18253511544609001572866960948873128266198935669250718031100637619547827597184",
            )
            .unwrap(),
            halo2curves::bn256::Fq::from_str_vartime(
                "10764647077472957448033591885865458661573660819003350325268673957890498500987",
            )
            .unwrap(),
        ),
        y: halo2curves::bn256::Fq2::new(
            halo2curves::bn256::Fq::from_str_vartime(
                "19756181390911900613508142947142748782977087973617411469215564659012323409872",
            )
            .unwrap(),
            halo2curves::bn256::Fq::from_str_vartime(
                "15207030507740967976352749097256929091435606784526748170016829002013506957017",
            )
            .unwrap(),
        ),
    };
    let (_, vk) = EvaluationEngine::<HyperKZGEngine>::setup(&CommitmentKey::new(vec![], h, tau_H));

    let file = std::fs::File::open("test_assets/ppot_0080_10.bin").unwrap();
    let mut ps = super::deserialize_flat_compressed_hyperkzg_public_setup_from_reader(
        &file,
        ark_serialize::Validate::Yes,
    )
    .unwrap();

    ps.insert(0, G1Affine::generator());

    (ps, vk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::{
            commitment::{
                commitment_evaluation_proof_test::{
                    test_commitment_evaluation_proof_with_length_1,
                    test_random_commitment_evaluation_proof,
                    test_simple_commitment_evaluation_proof,
                },
                VecCommitmentExt,
            },
            proof::{Keccak256Transcript, Transcript},
        },
        proof_primitive::hyperkzg::{
            deserialize_flat_compressed_hyperkzg_public_setup_from_reader,
            nova_commitment_key_to_hyperkzg_public_setup,
        },
    };
    use ark_ec::AffineRepr;
    use ark_serialize::Validate;
    use ff::PrimeField;
    use nova_snark::{
        provider::hyperkzg::CommitmentEngine, traits::commitment::CommitmentEngineTrait,
    };

    #[test]
    fn we_can_create_small_hyperkzg_evaluation_proofs() {
        let h: halo2curves::bn256::G1Affine = halo2curves::bn256::G1Affine::generator();
        let tau_H: halo2curves::bn256::G2Affine = halo2curves::bn256::G2Affine {
            x: halo2curves::bn256::Fq2::new(
                halo2curves::bn256::Fq::from_str_vartime(
                    "18253511544609001572866960948873128266198935669250718031100637619547827597184",
                )
                .unwrap(),
                halo2curves::bn256::Fq::from_str_vartime(
                    "10764647077472957448033591885865458661573660819003350325268673957890498500987",
                )
                .unwrap(),
            ),
            y: halo2curves::bn256::Fq2::new(
                halo2curves::bn256::Fq::from_str_vartime(
                    "19756181390911900613508142947142748782977087973617411469215564659012323409872",
                )
                .unwrap(),
                halo2curves::bn256::Fq::from_str_vartime(
                    "15207030507740967976352749097256929091435606784526748170016829002013506957017",
                )
                .unwrap(),
            ),
        };
        let (_, vk) =
            EvaluationEngine::<HyperKZGEngine>::setup(&CommitmentKey::new(vec![], h, tau_H));

        let file = std::fs::File::open("test_assets/ppot_0080_10.bin").unwrap();
        let mut ps =
            deserialize_flat_compressed_hyperkzg_public_setup_from_reader(&file, Validate::Yes)
                .unwrap();

        ps.insert(0, G1Affine::generator());

        let ck: CommitmentKey<HyperKZGEngine> = CommitmentEngine::setup(b"test", 32);
        let (_, vk) = EvaluationEngine::setup(&ck);
        let ps = nova_commitment_key_to_hyperkzg_public_setup(&ck);

        test_simple_commitment_evaluation_proof::<HyperKZGCommitmentEvaluationProof>(
            &&ps[..],
            &&vk,
        );
        test_commitment_evaluation_proof_with_length_1::<HyperKZGCommitmentEvaluationProof>(
            &&ps[..],
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

    fn hex(data: &[u8]) -> String {
        use std::fmt::Write;
        data.iter()
            .fold(String::with_capacity(data.len() * 2), |mut s, c| {
                write!(s, "{c:02x}").unwrap();
                s
            })
    }

    #[test]
    fn we_can_create_small_valid_proof_for_use_in_solidity_tests() {
        let (ps, vk) = load_setups();

        let mut transcript = Keccak256Transcript::new();
        let proof = <HyperKZGCommitmentEvaluationProof>::new(
            &mut transcript,
            &[
                BNScalar::from(0),
                BNScalar::from(1),
                BNScalar::from(2),
                BNScalar::from(3),
            ],
            &[BNScalar::from(7), BNScalar::from(5)],
            0,
            &&ps[..],
        );

        let commits = Vec::from_columns_with_offset(
            [crate::base::database::Column::Scalar(&[
                BNScalar::from(0),
                BNScalar::from(1),
                BNScalar::from(2),
                BNScalar::from(3),
            ])],
            0,
            &&ps[..],
        );

        let bincode_options = bincode::config::standard()
            .with_fixed_int_encoding()
            .with_big_endian();
        let commits_bytes = bincode::serde::encode_to_vec(&commits, bincode_options).unwrap();
        let proof_bytes = bincode::serde::encode_to_vec(&proof, bincode_options).unwrap();
        dbg!(hex(&commits_bytes));
        dbg!(hex(&proof_bytes));

        let mut transcript = Keccak256Transcript::new();
        let r = proof.verify_proof(
            &mut transcript,
            &commits[0],
            &BNScalar::from(17),
            &[BNScalar::from(7), BNScalar::from(5)],
            0,
            4,
            &&vk,
        );
        assert!(r.is_ok());
    }
}
