use crate::base::proof::{
    Column, Commit, Commitment, GeneralColumn, PipProve, PipVerify, ProofError, Transcript,
};

#[derive(Clone, Debug)]
pub struct CountProof {
    pub c_col: Commitment,   // Commitment of the counted
    pub c_count: Commitment, // Commitment of the singleton Int64 "count" column
}

impl PipProve<(GeneralColumn,), GeneralColumn> for CountProof {
    fn prove(
        transcript: &mut Transcript,
        input: (GeneralColumn,),
        output: GeneralColumn,
        input_commitment: (Commitment,),
    ) -> Self {
        let a = input.0;
        let length = a.len() as i64;
        let c_in = input_commitment.0;
        assert_eq!(output.len(), 1);
        let output_as_length = match output.clone() {
            GeneralColumn::Int8Column(c) => c.data[0] as i64,
            GeneralColumn::Int16Column(c) => c.data[0] as i64,
            GeneralColumn::Int32Column(c) => c.data[0] as i64,
            GeneralColumn::Int64Column(c) => c.data[0] as i64,
            GeneralColumn::UInt8Column(c) => c.data[0] as i64,
            GeneralColumn::UInt16Column(c) => c.data[0] as i64,
            GeneralColumn::UInt32Column(c) => c.data[0] as i64,
            GeneralColumn::UInt64Column(c) => c.data[0] as i64,
            _ => panic!("The result of Count has to be an integer"),
        };
        assert_eq!(length, c_in.length as i64);
        assert_eq!(length, output_as_length);
        transcript.count_domain_sep();
        let c_count = output.commit();
        transcript.append_commitment(b"c_count", &c_count);
        CountProof {
            c_col: c_in,
            c_count,
        }
    }
}

impl PipVerify<(Commitment,), Commitment> for CountProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Commitment,),
    ) -> Result<(), ProofError> {
        transcript.count_domain_sep();
        transcript.append_commitment(b"c_count", &self.c_count);
        let length = input_commitments.0.length;
        let count_column = Column {
            data: vec![length as i64],
        };
        let c_count_expected = count_column.commit();
        if self.c_col.length != length || self.c_count != c_count_expected {
            Err(ProofError::VerificationError)
        } else {
            Ok(())
        }
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_count
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_count_proof() {
        let input = GeneralColumn::Int32Column(Column {
            data: vec![1, -2, 3],
        });
        let output = GeneralColumn::Int64Column(Column { data: vec![3] });

        let mut transcript = Transcript::new(b"counttest");
        let c_in = input.commit();
        let count_proof =
            CountProof::prove(&mut transcript, (input.clone(),), output.clone(), (c_in,));

        //the proof confirms as correct
        let mut transcript = Transcript::new(b"counttest");
        assert!(count_proof.verify(&mut transcript, (c_in,)).is_ok());

        //the output commitment is correct as well
        assert_eq!(output.commit(), count_proof.get_output_commitments());

        //wrong input commitment length
        let mut transcript = Transcript::new(b"counttest");
        let mut wrong_c_in = c_in.clone();
        wrong_c_in.length = 2;
        assert!(count_proof.verify(&mut transcript, (wrong_c_in,)).is_err());
    }
}
