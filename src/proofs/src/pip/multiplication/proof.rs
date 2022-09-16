use crate::{
    base::{
        proof::{
            Commitment, GeneralColumn, MessageLabel, PipProve, PipVerify, ProofError, Transcript,
        },
        scalar::SafeIntColumn,
    },
    pip::{
        range::{arithmetic, LogMaxReductionProof},
        scalar_multiply::ScalarMultiplyProof,
    },
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiplicationProof {
    pub(super) scalar_multiply_proof: ScalarMultiplyProof,
    pub(super) c_product: Commitment,
    pub(crate) log_max_reduction_proof:
        Option<LogMaxReductionProof<{ MultiplicationProof::LOG_MAX_MAX }>>,
}

impl MultiplicationProof {
    const LOG_MAX_MAX: u8 = 128;
}

impl PipProve<(GeneralColumn, GeneralColumn), GeneralColumn> for MultiplicationProof {
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

                MultiplicationProof::prove(transcript, (left, right), output, input_commitment)
            }
            _ => {
                panic!("type error");
            }
        }
    }
}

impl PipProve<(SafeIntColumn, SafeIntColumn), SafeIntColumn> for MultiplicationProof {
    fn prove(
        transcript: &mut Transcript,
        (input_a, input_b): (SafeIntColumn, SafeIntColumn),
        product: SafeIntColumn,
        input_commitments: (Commitment, Commitment),
    ) -> Self {
        let (c_a, c_b) = input_commitments;

        assert_eq!(input_a.len(), input_b.len());
        assert_eq!(input_a.len(), product.len());
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

        let scalar_multiply_proof = ScalarMultiplyProof::prove(
            transcript,
            (
                input_a.values().clone().into(),
                input_b.values().clone().into(),
            ),
            product.values().clone().into(),
            (c_a, c_b),
        );

        let c_product = scalar_multiply_proof
            .get_output_commitments()
            .with_log_max(input_a.log_max() + input_b.log_max());

        transcript
            .append_auto(
                MessageLabel::Multiplication,
                &(input_a.len(), c_product.as_compressed()),
            )
            .unwrap();

        let (c_product, log_max_reduction_proof) =
            arithmetic::reduce_and_prove_if_necessary(transcript, product, c_product);

        MultiplicationProof {
            scalar_multiply_proof,
            c_product,
            log_max_reduction_proof,
        }
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for MultiplicationProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        (c_a, c_b): (Commitment, Commitment),
    ) -> Result<(), ProofError> {
        self.scalar_multiply_proof.verify(transcript, (c_a, c_b))?;

        if self.c_product != self.scalar_multiply_proof.get_output_commitments() {
            return Err(ProofError::VerificationError);
        }

        // self.c_product is provided by the prover and will have an already-reduced log_max
        let c_product_calculated = self.c_product.with_log_max(
            c_a.log_max
                .expect("commitments of SafeIntColumns should have a log_max")
                + c_b
                    .log_max
                    .expect("commitments of SafeIntColumns should have a log max"),
        );
        transcript.append_auto(
            MessageLabel::Multiplication,
            &(c_a.length, c_product_calculated.as_compressed()),
        )?;

        arithmetic::verify_with_reduction_if_necessary(
            transcript,
            &self.c_product,
            &self.log_max_reduction_proof,
            c_product_calculated,
        )
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_product
    }
}
