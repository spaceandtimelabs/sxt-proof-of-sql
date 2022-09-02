use crate::base::proof::{Commitment, PipProve, PipVerify, ProofError, Table, Transcript};

/// For pass through ExecutionPlans
#[derive(Clone, Debug)]
pub struct TrivialProof {
    pub c_out: Vec<Commitment>,
}

impl PipProve<(Table,), Table> for TrivialProof {
    fn prove(
        transcript: &mut Transcript,
        input: (Table,),
        output: Table,
        input_commitment: (Vec<Commitment>,),
    ) -> Self {
        let c_in = input_commitment.0;
        let num_columns = input.0.data.len();
        assert_eq!(num_columns, c_in.len());
        if num_columns > 0 {
            let num_rows = input.0.data[0].len();
            assert_eq!(num_rows, c_in[0].length);
            assert_eq!(num_rows, output.data[0].len());
        }
        transcript.trivial_domain_sep();
        transcript.append_commitments(b"c_out", &c_in);
        TrivialProof { c_out: c_in }
    }
}

impl PipVerify<(Vec<Commitment>,), Vec<Commitment>> for TrivialProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Vec<Commitment>,),
    ) -> Result<(), ProofError> {
        transcript.trivial_domain_sep();
        let c_out = input_commitments.0;
        transcript.append_commitments(b"c_out", &self.c_out);
        if self.c_out != c_out {
            println!("Proof: {:?}", self.c_out);
            println!("Input: {:?}", c_out);
            Err(ProofError::VerificationError)
        } else {
            Ok(())
        }
    }

    fn get_output_commitments(&self) -> Vec<Commitment> {
        self.c_out.clone()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::base::proof::{Column, Commit, GeneralColumn};

    #[test]
    fn test_trivial_proof() {
        // Setup
        let table = Table {
            data: vec![
                GeneralColumn::Int32Column(Column {
                    data: vec![1, 2, 3],
                }),
                GeneralColumn::Int64Column(Column {
                    data: vec![1, -2, -3],
                }),
            ],
            num_rows: 3,
        };

        let mut transcript = Transcript::new(b"trivialtest");
        let c_in = table.commit();
        let trivial_proof = TrivialProof::prove(
            &mut transcript,
            (table.clone(),),
            table.clone(),
            (c_in.clone(),),
        );

        //the proof confirms as correct
        let mut transcript = Transcript::new(b"trivialtest");
        assert!(trivial_proof.verify(&mut transcript, (c_in,)).is_ok());

        //the output commitment is correct as well
        assert_eq!(table.commit(), trivial_proof.get_output_commitments());

        //wrong input commitments
        let mut transcript = Transcript::new(b"trivialtest");
        let wrong_table = Table {
            data: vec![
                GeneralColumn::Int32Column(Column {
                    data: vec![2, 2, 3],
                }),
                GeneralColumn::Int64Column(Column {
                    data: vec![1, -2, -3],
                }),
            ],
            num_rows: 3,
        };
        let c_in_wrong = wrong_table.commit();
        assert!(trivial_proof
            .verify(&mut transcript, (c_in_wrong,))
            .is_err());
    }
}
