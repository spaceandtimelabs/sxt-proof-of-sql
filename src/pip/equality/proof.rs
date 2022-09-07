use crate::base::{
    proof::{
        Column, Commit, Commitment, GeneralColumn, PipProve, PipVerify, ProofError, Transcript,
    },
    scalar::IntoScalar,
};
use crate::pip::hadamard::HadamardProof;
use curve25519_dalek::scalar::Scalar;
use serde::{Deserialize, Serialize};
use std::iter;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EqualityProof {
    pub c_c: Commitment,
    pub c_e: Commitment,
    pub proof_ez0: HadamardProof,
    pub proof_czd: HadamardProof,
}

impl PipProve<(GeneralColumn, GeneralColumn), GeneralColumn> for EqualityProof {
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
                EqualityProof::prove(transcript, (left, right), output, input_commitment)
            }
            (GeneralColumn::Int8Column(left), GeneralColumn::Int8Column(right)) => {
                EqualityProof::prove(transcript, (left, right), output, input_commitment)
            }
            (GeneralColumn::Int16Column(left), GeneralColumn::Int16Column(right)) => {
                EqualityProof::prove(transcript, (left, right), output, input_commitment)
            }
            (GeneralColumn::Int32Column(left), GeneralColumn::Int32Column(right)) => {
                EqualityProof::prove(transcript, (left, right), output, input_commitment)
            }
            (GeneralColumn::Int64Column(left), GeneralColumn::Int64Column(right)) => {
                EqualityProof::prove(transcript, (left, right), output, input_commitment)
            }
            _ => {
                panic!("type error");
            }
        }
    }
}

impl<I> PipProve<(I, I), Column<bool>> for EqualityProof
where
    I: IntoIterator + Commit<Commitment = Commitment>,
    I::Item: Clone + IntoScalar,
{
    fn prove(
        transcript: &mut Transcript,
        input: (I, I),
        output: Column<bool>,
        input_commitments: (Commitment, Commitment),
    ) -> Self {
        // core implementation
        let a = input.0;
        let b = input.1;
        let e = output;
        let c_a = input_commitments.0;
        let c_b = input_commitments.1;
        create_equality_proof(transcript, a, b, e, c_a, c_b)
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for EqualityProof {
    fn verify(
        &self,
        transcript: &mut Transcript,
        input_commitments: (Commitment, Commitment),
    ) -> Result<(), ProofError> {
        let c_a = input_commitments.0;
        let c_b = input_commitments.1;
        verify_proof(transcript, self, c_a, c_b)
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_e
    }
}

fn create_equality_proof<I>(
    transcript: &mut Transcript,
    a: I,
    b: I,
    e: Column<bool>,
    c_a: Commitment,
    c_b: Commitment,
) -> EqualityProof
where
    I: IntoIterator + Commit<Commitment = Commitment>,
    I::Item: Clone + IntoScalar,
{
    transcript.equality_domain_sep(c_a.length as u64);
    let length = c_a.length;
    let (z_vec, c_vec): (Vec<Scalar>, Vec<Scalar>) = a
        .into_iter()
        .zip(b.into_iter())
        .map(|(ai, bi)| {
            let zi: Scalar = ai.into_scalar() - bi.into_scalar();
            let ci = if zi == Scalar::zero() {
                Scalar::zero()
            } else {
                zi.invert()
            };
            (zi, ci)
        })
        .unzip();

    let e_scalar: Vec<Scalar> = e.iter().map(|ei| ei.into_scalar()).collect();
    let d_vec: Vec<Scalar> = e_scalar.iter().map(|ei| Scalar::one() - ei).collect();
    let zero_vec: Vec<Scalar> = iter::repeat(Scalar::zero()).take(length).collect();

    let c_c = Commitment::from(c_vec.as_slice());
    let c_e = Commitment::from(e_scalar.as_slice());
    let c_z = c_a - c_b;
    transcript.append_commitment(b"c_c", &c_c);
    transcript.append_commitment(b"c_e", &c_e);
    let proof_ez0 = HadamardProof::prove(
        transcript,
        (
            Column { data: e_scalar },
            Column {
                data: z_vec.clone(),
            },
        ),
        Column { data: zero_vec },
        (c_e, c_z),
    );
    let proof_czd = HadamardProof::prove(
        transcript,
        (Column { data: c_vec }, Column { data: z_vec }),
        Column { data: d_vec },
        (c_c, c_z),
    );

    EqualityProof {
        c_c,
        c_e,
        proof_ez0,
        proof_czd,
    }
}

fn verify_proof(
    transcript: &mut Transcript,
    proof: &EqualityProof,
    c_a: Commitment,
    c_b: Commitment,
) -> Result<(), ProofError> {
    assert_eq!(c_a.length, c_b.length);

    if c_a.length != proof.c_e.length
        || c_a.length != proof.c_c.length
        || c_a.length != proof.proof_ez0.commit_ab.length
        || c_a.length != proof.proof_czd.commit_ab.length
    {
        return Err(ProofError::VerificationError);
    }

    transcript.equality_domain_sep(c_a.length as u64);
    // Computing c_0 and c_1 here is terrible. It should be cached.
    let (zero, one): (Vec<Scalar>, Vec<Scalar>) = iter::repeat((Scalar::zero(), Scalar::one()))
        .take(c_a.length)
        .unzip();

    let c_0 = Commitment::from(&zero[..]);
    let c_1 = Commitment::from(&one[..]);

    let c_d = c_1 - proof.c_e;
    let c_z = c_a - c_b;
    transcript.append_commitment(b"c_c", &proof.c_c);
    transcript.append_commitment(b"c_e", &proof.c_e);
    proof.proof_ez0.verify(transcript, (proof.c_e, c_z))?;
    proof.proof_czd.verify(transcript, (proof.c_c, c_z))?;
    if proof.proof_ez0.commit_ab != c_0 || proof.proof_czd.commit_ab != c_d {
        Err(ProofError::VerificationError)
    } else {
        Ok(())
    }
}
