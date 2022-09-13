use crate::{
    base::{
        proof::{
            Column, Commitment, GeneralColumn, MessageLabel, PipProve, PipVerify, ProofError,
            Transcript,
        },
        scalar::IntoScalar,
    },
    pip::hadamard::HadamardProof,
};
use serde::{Deserialize, Serialize};

// NOTE: The or operator can be written as x || y == x + y - x * y, which is what is used to prove this.
/// Implementation of proof for Or logical operator.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrProof {
    product_proof: HadamardProof,
    output_commitment: Commitment,
}

impl PipProve<(GeneralColumn, GeneralColumn), GeneralColumn> for OrProof {
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
        let output = Column::<bool>::try_from(output).expect("type error");

        match (left, right) {
            (GeneralColumn::BooleanColumn(left), GeneralColumn::BooleanColumn(right)) => {
                OrProof::prove(transcript, (left, right), output, input_commitment)
            }
            _ => {
                panic!("type error");
            }
        }
    }
}

impl PipProve<(Column<bool>, Column<bool>), Column<bool>> for OrProof {
    fn prove(
        transcript: &mut Transcript,
        input: (Column<bool>, Column<bool>),
        _output: Column<bool>,
        input_commitments: (Commitment, Commitment),
    ) -> Self {
        // core implementation
        transcript
            .append_auto(MessageLabel::Or, &input_commitments.0.length)
            .unwrap();
        // inputs_and is x*y.
        let inputs_product = Column::from(
            input
                .0
                .iter()
                .zip(input.1.iter())
                .map(|(a, b)| (*a && *b).into_scalar())
                .collect::<Vec<_>>(),
        );
        // this is a proof that x*y is computed correctly
        let proof = HadamardProof::prove(
            transcript,
            (input.0.into_scalar_column(), input.1.into_scalar_column()),
            inputs_product,
            input_commitments,
        );
        // The output is x+y-x*y.
        let output_commitment =
            input_commitments.0 + input_commitments.1 - proof.get_output_commitments();
        Self {
            product_proof: proof,
            output_commitment,
        }
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for OrProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Commitment, Commitment),
    ) -> Result<(), ProofError> {
        transcript
            .append_auto(MessageLabel::Or, &input_commitments.0.length)
            .unwrap();
        // We need to check that the claimed output matches with both the inputs as well as the internal proof's output. i.e. this is a check that x + y == x*y + x||y
        if input_commitments.0 + input_commitments.1
            == self.product_proof.get_output_commitments() + self.get_output_commitments()
        {
            self.product_proof.verify(transcript, input_commitments)
        } else {
            Err(ProofError::VerificationError)
        }
    }

    fn get_output_commitments(&self) -> Commitment {
        self.output_commitment
    }
}
