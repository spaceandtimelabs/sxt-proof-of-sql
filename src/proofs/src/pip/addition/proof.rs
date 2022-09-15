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
pub struct AdditionProof {
    pub(super) c_sum: Commitment,
    pub(crate) log_max_reduction_proof:
        Option<LogMaxReductionProof<{ AdditionProof::LOG_MAX_MAX }>>,
}

impl AdditionProof {
    const LOG_MAX_MAX: u8 = 128;
}

impl PipProve<(GeneralColumn, GeneralColumn), GeneralColumn> for AdditionProof {
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
                // log_max correction may be necessary since the SafeIntColumn may be generated
                // from a datafusion array of an intermediate result instead of the output of an
                // intermediate proof.
                // In this scenario, log_max increments performed in previous proofs are only
                // accessible via the commitment log_max.
                //
                // This is safe so long as the correction is an increase, which we expect it to be
                let left_log_max = input_commitment
                    .0
                    .log_max
                    .expect("commitments of SafeIntColumns should have a log_max");
                let right_log_max = input_commitment
                    .1
                    .log_max
                    .expect("commitments of SafeIntColumns should have a log_max");

                let left = left
                    .with_log_max(
                        left_log_max
                    )
                    .expect("commitment log_max shouldn't be less than the log_max of the associated column");
                let right = right
                    .with_log_max(
                        right_log_max
                    )
                    .expect("commitment log_max shouldn't be less than the log_max of the associated column");

                AdditionProof::prove(transcript, (left, right), output, input_commitment)
            }
            _ => {
                panic!("type error");
            }
        }
    }
}

impl PipProve<(SafeIntColumn, SafeIntColumn), SafeIntColumn> for AdditionProof {
    fn prove(
        transcript: &mut Transcript,
        (input_a, input_b): (SafeIntColumn, SafeIntColumn),
        sum: SafeIntColumn,
        input_commitments: (Commitment, Commitment),
    ) -> Self {
        let (c_a, c_b) = input_commitments;

        assert_eq!(input_a.len(), input_b.len());
        assert_eq!(input_a.len(), sum.len());
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
        let c_sum = c_a + c_b;
        transcript
            .append_auto(
                MessageLabel::Addition,
                &(input_a.len(), c_sum.as_compressed()),
            )
            .unwrap();

        if c_sum
            .log_max
            .expect("commitments of SafeIntColumns should have a log_max")
            > AdditionProof::LOG_MAX_MAX
        {
            let sum_unreduced: SafeIntColumn = SafeIntColumn::try_new(
                sum.clone().into_iter().map(|s| s.value()).collect(),
                c_sum
                    .log_max
                    .expect("commitments of SafeIntColumn should have a log_max"),
            )
            .unwrap();

            let log_max_reduction_proof = Some(LogMaxReductionProof::<
                { AdditionProof::LOG_MAX_MAX },
            >::prove(
                transcript, (sum_unreduced,), sum, (c_sum,)
            ));

            let c_sum_reduced = c_sum.with_log_max(AdditionProof::LOG_MAX_MAX);

            AdditionProof {
                c_sum: c_sum_reduced,
                log_max_reduction_proof,
            }
        } else {
            AdditionProof {
                c_sum,
                log_max_reduction_proof: None,
            }
        }
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for AdditionProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        (c_a, c_b): (Commitment, Commitment),
    ) -> Result<(), ProofError> {
        // self.c_sum is provided by the prover and will have an already-reduced log_max
        let c_sum_calculated = c_a + c_b;
        transcript.append_auto(
            MessageLabel::Addition,
            &(c_a.length, c_sum_calculated.as_compressed()),
        )?;

        let calculated_log_max = c_sum_calculated.log_max.ok_or(ProofError::FormatError)?;

        let output_log_max = self.c_sum.log_max.ok_or(ProofError::FormatError)?;

        let maybe_log_max_reduction_proof = if calculated_log_max > AdditionProof::LOG_MAX_MAX {
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
            if output_log_max != AdditionProof::LOG_MAX_MAX {
                return Err(ProofError::VerificationError);
            }

            // verify the inner proof
            log_max_reduction_proof.verify(transcript, (c_sum_calculated,))?;

            // verify the LogMaxReductionProof is actually verifying against the correct output commitment
            if c_sum_calculated != log_max_reduction_proof.get_output_commitments() {
                return Err(ProofError::VerificationError);
            }
        }

        // Verify the provided output commitment
        if c_sum_calculated != self.c_sum {
            return Err(ProofError::VerificationError);
        }

        Ok(())
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_sum
    }
}
