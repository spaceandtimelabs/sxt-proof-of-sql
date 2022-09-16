use crate::{
    base::{
        proof::{
            Commitment, GeneralColumn, MessageLabel, PipProve, PipVerify, ProofError, Transcript,
        },
        scalar::SafeIntColumn,
    },
    pip::range::{arithmetic, LogMaxReductionProof},
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

        let (c_sum, log_max_reduction_proof) =
            arithmetic::reduce_and_prove_if_necessary(transcript, sum, c_sum);

        AdditionProof {
            c_sum,
            log_max_reduction_proof,
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

        arithmetic::verify_with_reduction_if_necessary(
            transcript,
            &self.c_sum,
            &self.log_max_reduction_proof,
            c_sum_calculated,
        )
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_sum
    }
}
