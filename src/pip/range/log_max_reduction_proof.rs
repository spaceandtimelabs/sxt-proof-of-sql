use crate::{
    base::{
        proof::{Column, Commitment, PipProve, PipVerify, ProofError, Transcript},
        scalar::SafeIntColumn,
    },
    pip::range::BinaryRangeProof,
};
use curve25519_dalek::scalar::Scalar;
use serde::{Deserialize, Serialize};
use std::iter::repeat;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogMaxReductionProof<const B: u8> {
    pub bin_range_proof: BinaryRangeProof<B>,
    pub c_reduced: Commitment,
}

impl<const B: u8> PipProve<(SafeIntColumn,), SafeIntColumn> for LogMaxReductionProof<B> {
    fn prove(
        //The merlin transcript for the prover
        transcript: &mut Transcript,
        //The inputs to the PIP
        (input,): (SafeIntColumn,),
        //The output of the PIP. Note: these are not computed by the PIP itself. The PIP simply produces a proof that these are correct.
        output: SafeIntColumn,
        //The commitments of the inputs to the PIP. This is redundant since it can be computed from input_columns, but they will already have been computed
        (input_commitment,): (Commitment,),
    ) -> Self {
        assert_eq!(output.log_max(), B);

        transcript.log_max_reduction_domain_sep(B);

        let true_column: Column<bool> = repeat(true).take(input.len()).collect::<Vec<_>>().into();

        let bin_range_proof =
            BinaryRangeProof::<B>::prove(transcript, (input,), true_column, (input_commitment,));

        let c_reduced = input_commitment.with_log_max(B);

        LogMaxReductionProof::<B> {
            bin_range_proof,
            c_reduced,
        }
    }
}

impl<const B: u8> PipVerify<(Commitment,), Commitment> for LogMaxReductionProof<B> {
    fn verify(
        &self,
        //The merlin transcript for the verifier
        transcript: &mut Transcript,
        //The commitments of the inputs to the PIP. Typically, these are known by the verifier.
        (input_commitment,): (Commitment,),
    ) -> Result<(), ProofError> {
        transcript.log_max_reduction_domain_sep(B);

        self.bin_range_proof
            .verify(transcript, (input_commitment,))?;

        // verify that the proof is actually verifying against a column of trues
        let c_one: Commitment = repeat(Scalar::one())
            .take(input_commitment.length)
            .collect::<Vec<Scalar>>()
            .as_slice()
            .into();

        if self.bin_range_proof.get_output_commitments() == c_one {
            Ok(())
        } else {
            Err(ProofError::VerificationError)
        }
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_reduced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::base::proof::Commit;

    #[test]
    fn test_log_max_reduction() {
        let values = vec![
            Scalar::from(1u32),
            Scalar::from(2u32),
            -Scalar::from(4u32),
            Scalar::from(5u32),
            Scalar::from(4u32),
            -Scalar::from(1u32),
            -Scalar::from(5u32),
            Scalar::from(0u32),
        ];
        let a = SafeIntColumn::try_new(values.clone(), 8).unwrap();
        let reduced = SafeIntColumn::try_new(values, 4).unwrap();

        let c_a = a.commit();
        let c_reduced = reduced.commit();

        let mut transcript = Transcript::new(b"logmaxreductiontest");
        let proof = LogMaxReductionProof::<4>::prove(&mut transcript, (a,), reduced, (c_a,));

        let mut transcript = Transcript::new(b"logmaxreductiontest");
        assert!(proof.verify(&mut transcript, (c_a,)).is_ok());

        // correct output commitment
        assert_eq!(proof.get_output_commitments(), c_reduced);
        assert_eq!(proof.get_output_commitments().log_max, c_reduced.log_max);

        let b = SafeIntColumn::try_new(
            vec![
                Scalar::from(1u32),
                Scalar::from(2u32),
                -Scalar::from(4u32),
                Scalar::from(5u32),
                Scalar::from(4u32),
                -Scalar::from(1u32),
                -Scalar::from(5u32),
                Scalar::from(1u32),
            ],
            8,
        )
        .unwrap();
        let c_b = b.commit();

        // wrong input commitments
        let mut transcript = Transcript::new(b"logmaxreductiontest");

        assert!(proof.verify(&mut transcript, (c_b,)).is_err());
    }

    #[test]
    fn test_log_max_reduction_wrong() {
        let a = SafeIntColumn::try_new(
            vec![
                Scalar::from(1u32),
                Scalar::from(2u32),
                Scalar::from(3u32),
                Scalar::from(17u32),
                -Scalar::from(1u32),
                -Scalar::from(5u32),
                Scalar::from(0u32),
            ],
            6,
        )
        .unwrap();
        let reduced = SafeIntColumn::try_new(
            vec![
                Scalar::from(1u32),
                Scalar::from(2u32),
                Scalar::from(3u32),
                Scalar::from(17u32),
                -Scalar::from(1u32),
                -Scalar::from(5u32),
                Scalar::from(1u32),
            ],
            5,
        )
        .unwrap();

        let c_a = a.commit();

        let mut transcript = Transcript::new(b"logmaxreductiontest");
        let proof = LogMaxReductionProof::<5>::prove(&mut transcript, (a,), reduced, (c_a,));

        assert!(proof.verify(&mut transcript, (c_a,)).is_err());
    }

    #[test]
    #[should_panic]
    fn test_log_max_reduction_out_of_bounds() {
        let a = SafeIntColumn::try_new(
            vec![
                Scalar::from(1u32),
                Scalar::from(2u32),
                Scalar::from(3u32),
                Scalar::from(17u32),
                -Scalar::from(1u32),
                -Scalar::from(5u32),
                Scalar::from(0u32),
            ],
            6,
        )
        .unwrap();
        let reduced = SafeIntColumn::try_new(
            vec![
                Scalar::from(1u32),
                Scalar::from(2u32),
                Scalar::from(3u32),
                Scalar::from(17u32),
                -Scalar::from(1u32),
                -Scalar::from(5u32),
                Scalar::from(0u32),
            ],
            4,
        )
        .unwrap();

        let c_a = a.commit();

        let mut transcript = Transcript::new(b"logmaxreductiontest");
        let proof = LogMaxReductionProof::<4>::prove(&mut transcript, (a,), reduced, (c_a,));

        assert!(proof.verify(&mut transcript, (c_a,)).is_err());
    }
}
