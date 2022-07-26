use crate::{
    base::{
        proof::{Column, Commitment, PipProve, PipVerify, ProofError, Transcript},
        scalar::IntoScalar,
    },
    pip::hadamard::HadamardProof,
};

// NOTE: The or operator can be written as x || y == x + y - x * y, which is what is used to prove this.
/// Implementation of proof for Or logical operator.
pub struct OrProof {
    pub product_proof: HadamardProof,
    pub output_commitment: Commitment,
}
impl PipProve<(Column<bool>, Column<bool>), Column<bool>> for OrProof {
    fn prove(
        transcript: &mut Transcript,
        input: (Column<bool>, Column<bool>),
        _output: Column<bool>,
        input_commitments: (Commitment, Commitment),
    ) -> Self {
        transcript.or_domain_sep(input_commitments.0.length as u64);
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
        transcript.or_domain_sep(input_commitments.0.length as u64);
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
