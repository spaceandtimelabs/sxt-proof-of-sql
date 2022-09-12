use curve25519_dalek::{ristretto::CompressedRistretto, scalar::Scalar, traits::Identity};
use pedersen::compute::compute_commitments;
use std::iter::repeat;

use crate::{
    base::{
        math::SignedBitDecompose,
        proof::{Column, Commit, Commitment, PipProve, PipVerify, ProofError, MessageLabel},
        scalar::IntoScalar,
    },
    pip::hadamard::HadamardProof,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PositiveProof {
    pub c_decomposed_columns: Vec<Commitment>,
    pub bit_proof_decomposed_columns: Vec<HadamardProof>,
}

impl<I> PipProve<(I,), Column<bool>> for PositiveProof
where
    I: IntoIterator + Commit<Commitment = Commitment> + Clone,
    I::Item: IntoScalar + SignedBitDecompose + Clone,
{
    fn prove(
        //The merlin transcript for the prover
        transcript: &mut crate::base::proof::Transcript,
        //The inputs to the PIP
        (input,): (I,),
        //The output of the PIP. Note: these are not computed by the PIP itself. The PIP simply produces a proof that these are correct.
        output: Column<bool>,
        //The commitments of the inputs to the PIP. This is redundant since it can be computed from input_columns, but they will already have been computed
        (_input_commitment,): (Commitment,),
    ) -> Self {
        // bit_columns is an atypical representation of the values.
        let bit_columns = decompose_input(&input, output);
        let group_commitments = calculate_commitments(&bit_columns);
        transcript.append_auto(MessageLabel::Positive, &group_commitments).unwrap();
        let commitments: Vec<_> = group_commitments
            .iter()
            .map(|c| Commitment::from_compressed(*c, input.clone().into_iter().count()))
            .collect();
        let proofs = commitments
            .iter()
            .zip(bit_columns)
            .map(|(c_c, bit_column)| {
                let c: Column<Scalar> = bit_column.into();
                HadamardProof::prove(transcript, (c.clone(), c.clone()), c, (*c_c, *c_c))
            })
            .collect();
        Self {
            c_decomposed_columns: commitments,
            bit_proof_decomposed_columns: proofs,
        }
    }
}

fn calculate_commitments(columns: &Vec<Vec<Scalar>>) -> Vec<CompressedRistretto> {
    let mut commitments: Vec<CompressedRistretto> = repeat(CompressedRistretto::identity())
        .take(columns.len())
        .collect();
    compute_commitments(
        &mut commitments,
        columns
            .iter()
            .map(|c| c.as_slice())
            .collect::<Vec<_>>()
            .as_slice(),
    );
    commitments
}

/// Parameters: `output[j]` is the sign of `input[j]`.
///
/// For any `N`, every value `-2^N<input[j]<=2^N` can be uniquely written in the form `input[j] = sign * 2^N - sum_{0}^{N-1} (b_i[j] * 2^i)` where sign is 1 when `input[j]` is positibe and 0 when `input[j]` is negative
/// This function picks the smallest `N` so that all values in the input fit in that range.
/// The returned vector of vectors, `r`, is such that
///
/// `r[i][j] = b_i[j]` as a `Scalar`
///
/// and
///
/// `r[N][j] = output[j]` as a `Scalar`. (Note: this is also the sign of `input[j]` as a Scalar)
fn decompose_input<I>(input: &I, output: Column<bool>) -> Vec<Vec<Scalar>>
where
    I: IntoIterator + Clone,
    I::Item: SignedBitDecompose,
{
    let scalar_output: Vec<Scalar> = output.iter().map(|&b| b.into_scalar()).collect();
    let mut columns: Vec<Vec<Scalar>> = Vec::new();
    for (i, (value, positive)) in input.clone().into_iter().zip(output.iter()).enumerate() {
        let bits = match positive {
            true => value.sub_one_bits(),
            false => value.neg_bits(),
        };
        while columns.len() < bits.len() {
            columns.push(scalar_output.clone())
        }
        for (j, b) in bits.iter().enumerate() {
            columns[j][i] = (b ^ positive).into_scalar();
        }
    }
    columns.push(scalar_output);
    columns
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{proof::Column, scalar::IntoScalar},
        pip::positive::proof::decompose_input,
    };

    #[test]
    fn test_decompose_input() {
        let columns = decompose_input(
            &Column::from(vec![3_i8, 6_i8, -3_i8, -4_i8, 0_i8]),
            Column::from(vec![true, true, false, false, false]),
        );
        // 3 = 8 - 5
        // 6 = 8 - 2
        // -3= 0 - 3
        // -4= 0 - 4
        // 0 = 0 - 0
        assert_eq!(
            columns[0],
            (vec![
                true.into_scalar(),
                false.into_scalar(),
                true.into_scalar(),
                false.into_scalar(),
                false.into_scalar()
            ])
        );
        assert_eq!(
            columns[1],
            (vec![
                false.into_scalar(),
                true.into_scalar(),
                true.into_scalar(),
                false.into_scalar(),
                false.into_scalar()
            ])
        );
        assert_eq!(
            columns[2],
            (vec![
                true.into_scalar(),
                false.into_scalar(),
                false.into_scalar(),
                true.into_scalar(),
                false.into_scalar()
            ])
        );
        assert_eq!(
            columns[3],
            (vec![
                true.into_scalar(),
                true.into_scalar(),
                false.into_scalar(),
                false.into_scalar(),
                false.into_scalar()
            ])
        );
    }
}

impl PipVerify<(Commitment,), Commitment> for PositiveProof {
    fn verify(
        &self,
        //The merlin transcript for the verifier
        transcript: &mut crate::base::proof::Transcript,
        //The commitments of the inputs to the PIP. Typically, these are known by the verifier.
        (input_commitments,): (Commitment,),
    ) -> Result<(), crate::base::proof::ProofError> {
        if self.c_decomposed_columns.len() > 252 {
            return Err(ProofError::VerificationError);
        }
        for c_c in self.c_decomposed_columns.iter() {
            if c_c.length != input_commitments.length {
                return Err(ProofError::VerificationError);
            }
        }
        transcript.append_auto(
            MessageLabel::Positive, 
            &self.c_decomposed_columns.iter().map(|c| c.as_compressed()).collect::<Vec<_>>(),
        )?;
        let mut it = self.c_decomposed_columns.iter().rev();
        let mut recompose = match it.next() {
            None => {
                return Err(ProofError::VerificationError);
            }
            Some(c) => c.try_as_decompressed()?,
        };
        for c in it {
            recompose += recompose;
            recompose -= c.try_as_decompressed()?;
        }
        if recompose.compress() != input_commitments.as_compressed() {
            return Err(ProofError::VerificationError);
        }
        for (c_c, p) in self
            .c_decomposed_columns
            .iter()
            .zip(&self.bit_proof_decomposed_columns)
        {
            p.verify(transcript, (*c_c, *c_c))?;
        }
        Ok(())
    }

    fn get_output_commitments(&self) -> Commitment {
        *self.c_decomposed_columns.last().unwrap()
    }
}
