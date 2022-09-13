use crate::{
    base::{
        proof::{
            Commitment, GeneralColumn, MessageLabel, PipProve, PipVerify, ProofError, Transcript,
        },
        scalar::SafeIntColumn,
    },
    pip::range::LogMaxReductionProof,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubtractionProof {
    pub c_diff: Commitment,
    pub log_max_reduction_proof: Option<LogMaxReductionProof<{ SubtractionProof::LOG_MAX_MAX }>>,
}

impl SubtractionProof {
    const LOG_MAX_MAX: u8 = 128;
}

impl PipProve<(GeneralColumn, GeneralColumn), GeneralColumn> for SubtractionProof {
    fn prove(
        //The merlin transcript for the prover
        transcript: &mut Transcript,
        //The inputs to the PIP
        (left, right): (GeneralColumn, GeneralColumn),
        //The output of the PIP. Note: these are not computed by the PIP itself. The PIP simply produces a proof that these are correct.
        output: GeneralColumn,
        //The commitments of the inputs to the PIP. This is redundant since it can be computed from input_columns, but they will already have been computed
        input_commitment: (Commitment, Commitment),
    ) -> Self {
        // general implementation
        // This will match against the type variants of the input and output columns,
        // and error if the combination of column types aren't valid for this proof.
        // The actual proof construction is handled in the core implementation.
        use GeneralColumn::*;
        match (left, right, output) {
            (SafeIntColumn(left), SafeIntColumn(right), SafeIntColumn(output)) => {
                SubtractionProof::prove(transcript, (left, right), output, input_commitment)
            }
            _ => {
                panic!("type error");
            }
        }
    }
}

impl PipProve<(SafeIntColumn, SafeIntColumn), SafeIntColumn> for SubtractionProof {
    fn prove(
        transcript: &mut Transcript,
        (input_a, input_b): (SafeIntColumn, SafeIntColumn),
        diff: SafeIntColumn,
        (c_a, c_b): (Commitment, Commitment),
    ) -> Self {
        // core implementation
        assert_eq!(input_a.len(), input_b.len());
        assert_eq!(input_a.len(), diff.len());
        assert_eq!(input_a.len(), c_a.length);
        assert_eq!(input_b.len(), c_b.length);

        assert_eq!(
            input_a.log_max(),
            c_a.log_max
                .expect("commitments of SafeIntColumns should have a log_max")
        );
        assert_eq!(
            input_b.log_max(),
            c_b.log_max
                .expect("commitments of SafeIntColumns should have a log_max")
        );
        let c_diff = c_a - c_b;
        transcript
            .append_auto(
                MessageLabel::Subtraction,
                &(input_a.len(), c_diff.as_compressed()),
            )
            .unwrap();

        if c_diff
            .log_max
            .expect("commitments of SafeIntColumns should have a log_max")
            > SubtractionProof::LOG_MAX_MAX
        {
            let diff_unreduced: SafeIntColumn = SafeIntColumn::try_new(
                diff.clone().into_iter().map(|s| s.value()).collect(),
                c_diff
                    .log_max
                    .expect("commitments of SafeIntColumn should have a log_max"),
            )
            .unwrap();

            let log_max_reduction_proof = Some(LogMaxReductionProof::<
                { SubtractionProof::LOG_MAX_MAX },
            >::prove(
                transcript, (diff_unreduced,), diff, (c_diff,)
            ));

            let c_diff_reduced = c_diff.with_log_max(SubtractionProof::LOG_MAX_MAX);

            SubtractionProof {
                c_diff: c_diff_reduced,
                log_max_reduction_proof,
            }
        } else {
            SubtractionProof {
                c_diff,
                log_max_reduction_proof: None,
            }
        }
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for SubtractionProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        (c_a, c_b): (Commitment, Commitment),
    ) -> Result<(), ProofError> {
        let c_diff_calculated = c_a - c_b;
        transcript.append_auto(
            MessageLabel::Subtraction,
            &(c_a.length, c_diff_calculated.as_compressed()),
        )?;

        let calculated_log_max = c_diff_calculated.log_max.ok_or(ProofError::FormatError)?;

        let output_log_max = self.c_diff.log_max.ok_or(ProofError::FormatError)?;

        let maybe_log_max_reduction_proof = if calculated_log_max > SubtractionProof::LOG_MAX_MAX {
            // Proof should have a reduction, error if it doesn't
            Some(
                self.log_max_reduction_proof
                    .as_ref()
                    .ok_or(ProofError::VerificationError)?,
            )
        } else {
            // Proof doesn't need a reduction, but might have one anyway
            self.log_max_reduction_proof.as_ref()
        };

        if let Some(log_max_reduction_proof) = maybe_log_max_reduction_proof {
            // Proof has a reduction. Whether or not it's required, verify it

            // verify that the commitment log_max has been reduced
            if output_log_max != SubtractionProof::LOG_MAX_MAX {
                return Err(ProofError::VerificationError);
            }

            // verify the inner proof
            log_max_reduction_proof.verify(transcript, (c_diff_calculated,))?;

            // verify the LogMaxReductionProof is actually verifying against the correct output commitment
            if c_diff_calculated != log_max_reduction_proof.get_output_commitments() {
                return Err(ProofError::VerificationError);
            }
        }

        // Verify the provided output commitment
        if c_diff_calculated != self.c_diff {
            return Err(ProofError::VerificationError);
        }

        Ok(())
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_diff
    }
}
