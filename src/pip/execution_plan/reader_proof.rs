use crate::base::proof::{
    Commit, Commitment, MessageLabel, PipProve, PipVerify, ProofError, Table, Transcript,
};

/// For reading a new data source
#[derive(Clone, Debug)]
pub struct ReaderProof {
    pub c_out: Vec<Commitment>,
}

impl PipProve<(), Table> for ReaderProof {
    fn prove(
        transcript: &mut Transcript,
        _input: (),
        output: Table,
        _input_commitment: (),
    ) -> Self {
        let c_out = output.commit();
        transcript
            .append_auto(
                MessageLabel::Reader,
                &c_out.iter().map(|c| c.as_compressed()).collect::<Vec<_>>(),
            )
            .unwrap();
        ReaderProof { c_out }
    }
}

impl PipVerify<(), Vec<Commitment>> for ReaderProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        _input_commitments: (),
    ) -> Result<(), ProofError> {
        transcript.append_auto(
            MessageLabel::Reader,
            &self
                .c_out
                .iter()
                .map(|c| c.as_compressed())
                .collect::<Vec<_>>(),
        )
    }

    fn get_output_commitments(&self) -> Vec<Commitment> {
        self.c_out.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::base::proof::{Commit, GeneralColumn};

    #[test]
    fn test_reader_proof() {
        // Setup
        let output = Table {
            data: vec![
                GeneralColumn::SafeIntColumn(vec![1, 2, 3].into()),
                GeneralColumn::SafeIntColumn(vec![1, -2, -3].into()),
            ],
            num_rows: 3,
        };

        let mut transcript = Transcript::new(b"readertest");
        let reader_proof = ReaderProof::prove(&mut transcript, (), output.clone(), ());

        //the proof confirms as correct
        let mut transcript = Transcript::new(b"readertest");
        assert!(reader_proof.verify(&mut transcript, ()).is_ok());

        //the output commitment is correct as well
        assert_eq!(output.commit(), reader_proof.get_output_commitments());
    }
}
