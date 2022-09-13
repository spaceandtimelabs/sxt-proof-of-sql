use crate::base::{
    proof::{
        Column, Commit, Commitment, GeneralColumn, MessageLabel, PipProve, PipVerify, ProofError,
        Transcript,
    },
    scalar::IntoScalar,
};
use crate::pip::hadamard::HadamardProof;
use curve25519_dalek::scalar::Scalar;
use serde::{Deserialize, Serialize};
use std::iter;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InequalityProof {
    c_c: Commitment,
    c_d: Commitment,
    proof_ez0: HadamardProof,
    proof_czd: HadamardProof,
}

impl PipProve<(GeneralColumn, GeneralColumn), GeneralColumn> for InequalityProof {
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
                InequalityProof::prove(transcript, (left, right), output, input_commitment)
            }
            (GeneralColumn::SafeIntColumn(left), GeneralColumn::SafeIntColumn(right)) => {
                InequalityProof::prove(transcript, (left, right), output, input_commitment)
            }
            _ => {
                panic!("type error");
            }
        }
    }
}

impl<I> PipProve<(I, I), Column<bool>> for InequalityProof
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
        let d = output;
        let c_a = input_commitments.0;
        let c_b = input_commitments.1;
        create_inequality_proof(transcript, a, b, d, c_a, c_b)
    }
}

impl PipVerify<(Commitment, Commitment), Commitment> for InequalityProof {
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
        self.c_d
    }
}

fn create_inequality_proof<I>(
    transcript: &mut Transcript,
    a: I,
    b: I,
    d: Column<bool>,
    c_a: Commitment,
    c_b: Commitment,
) -> InequalityProof
where
    I: IntoIterator,
    I::Item: IntoScalar + Clone,
{
    let length = c_a.length;
    let (z_vec, c_vec): (Vec<Scalar>, Vec<Scalar>) = a
        .into_iter()
        .zip(b.into_iter())
        .map(|(ai, bi)| {
            let zi = ai.into_scalar() - bi.into_scalar();
            let ci = if zi == Scalar::zero() {
                Scalar::zero()
            } else {
                zi.invert()
            };
            (zi, ci)
        })
        .unzip();

    let d_scalar: Vec<Scalar> = d.iter().map(|di| di.into_scalar()).collect();
    let e_vec: Vec<Scalar> = d_scalar.iter().map(|di| Scalar::one() - di).collect();
    let zero_vec: Vec<Scalar> = iter::repeat(Scalar::zero()).take(length).collect();
    let one_vec: Vec<Scalar> = iter::repeat(Scalar::one()).take(length).collect();
    let c_1 = Commitment::from(&one_vec[..]);

    let c_c = Commitment::from(&c_vec[..]);
    let c_d = Commitment::from(d_scalar.as_slice());
    let c_e = c_1 - c_d;
    let c_z = c_a - c_b;

    transcript
        .append_auto(
            MessageLabel::Equality,
            &(c_a.length, c_c.as_compressed(), c_d.as_compressed()),
        )
        .unwrap();
    let proof_ez0 = HadamardProof::prove(
        transcript,
        (e_vec.into(), z_vec.clone().into()),
        zero_vec.into(),
        (c_e, c_z),
    );
    let proof_czd = HadamardProof::prove(
        transcript,
        (c_vec.into(), z_vec.into()),
        d_scalar.into(),
        (c_c, c_z),
    );

    InequalityProof {
        c_c,
        c_d,
        proof_ez0,
        proof_czd,
    }
}

fn verify_proof(
    transcript: &mut Transcript,
    proof: &InequalityProof,
    c_a: Commitment,
    c_b: Commitment,
) -> Result<(), ProofError> {
    // Computing c_0 and c_1 here is terrible. It should be cached.
    let (zero, one): (Vec<Scalar>, Vec<Scalar>) = iter::repeat((Scalar::zero(), Scalar::one()))
        .take(c_a.length)
        .unzip();
    let c_0 = Commitment::from(&zero[..]);
    let c_1 = Commitment::from(&one[..]);
    let c_e = c_1 - proof.c_d;
    let c_z = c_a - c_b;

    transcript.append_auto(
        MessageLabel::Equality,
        &(
            c_a.length,
            proof.c_c.as_compressed(),
            proof.c_d.as_compressed(),
        ),
    )?;
    proof.proof_ez0.verify(transcript, (c_e, c_z))?;
    proof.proof_czd.verify(transcript, (proof.c_c, c_z))?;
    if proof.proof_ez0.get_output_commitments() != c_0
        || proof.proof_czd.get_output_commitments() != proof.c_d
    {
        Err(ProofError::VerificationError)
    } else {
        Ok(())
    }
}
