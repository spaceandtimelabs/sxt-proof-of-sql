use crate::{
    base::{
        proof::{Column, Commitment, PipProve, PipVerify, ProofError, Transcript, MessageLabel},
        scalar::IntoScalar,
    },
    pip::hadamard::HadamardProof,
};

use serde::{Deserialize, Serialize};

/// Implementation of the Hadamard Product of two columns, after converting them to Scalars.
/// Note: this is not safe for use when multiplying integers.
/// This is equivalent to the And logical operation when `T` is a bool, because x && y == x * y
#[derive(Serialize, Deserialize)]
pub struct ScalarMultiplyProof {
    pub proof: HadamardProof,
}

impl<T> PipProve<(Column<T>, Column<T>), Column<T>> for ScalarMultiplyProof
where
    T: IntoScalar + Clone,
{
    fn prove(
        transcript: &mut Transcript,
        input: (Column<T>, Column<T>),
        output: Column<T>,
        input_commitments: (Commitment, Commitment),
    ) -> Self {
        transcript.append_auto(MessageLabel::ScalarMultiply, &()).unwrap();
        Self {
            proof: HadamardProof::prove(
                transcript,
                (input.0.into_scalar_column(), input.1.into_scalar_column()),
                output.into_scalar_column(),
                input_commitments,
            ),
        }
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for ScalarMultiplyProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Commitment, Commitment),
    ) -> Result<(), ProofError> {
        transcript.append_auto(MessageLabel::ScalarMultiply, &()).unwrap();
        self.proof.verify(transcript, input_commitments)
    }

    fn get_output_commitments(&self) -> Commitment {
        self.proof.get_output_commitments()
    }
}
