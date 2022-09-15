use crate::{
    base::{
        proof::{
            Column, Commit, Commitment, MessageLabel, PipProve, PipVerify, ProofError, Transcript,
        },
        scalar::{SafeInt, SafeIntColumn},
    },
    pip::positive::PositiveProof,
};

use num_traits::One;
use serde::{Deserialize, Serialize};
use std::iter::repeat;

/// Proof of the claim that each value `a` in a column is or isn't within a particular range.
/// In this case, the range is represented by its log (base 2), `B`.
///
/// - `true` rows claim that `-2^B <= a <= 2^B`.
/// - `false` rows claim that `a < -2^B || a > 2^B`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BinaryRangeProof<const B: u8> {
    lower_bound_proof: PositiveProof,
    upper_bound_proof: PositiveProof,
    c_range: Commitment,
}

impl<const B: u8> PipProve<(SafeIntColumn,), Column<bool>> for BinaryRangeProof<B> {
    fn prove(
        //The merlin transcript for the prover
        transcript: &mut Transcript,
        //The inputs to the PIP
        (input,): (SafeIntColumn,),
        //The output of the PIP. Note: these are not computed by the PIP itself. The PIP simply produces a proof that these are correct.
        output: Column<bool>,
        //The commitments of the inputs to the PIP. This is redundant since it can be computed from input_columns, but they will already have been computed
        (_input_commitment,): (Commitment,),
    ) -> Self {
        // To avoid panicking during arithmetic, these two conditions must be true.
        // They assure that the input to both PositiveProofs will have a log_max of at most 250.
        // This is required since PositiveProof does subtraction, which increases the log_max once.
        assert!(input.log_max().max(B) <= 249);
        assert!(input.log_max().min(B) <= 248);

        let bound = SafeInt::from(2)
            .try_pow(B)
            .expect("the log_max shouldn't exceed 251 since B <= 250");
        transcript
            .append_auto(MessageLabel::BinaryRange, &B)
            .unwrap();

        let lower_bound_offset: SafeIntColumn = input
            .clone()
            .into_iter()
            .map(|si| {
                // SafeInt addition is not associative in terms of log_max.
                // Adding the smaller terms first gives us the smallest log_max in their sum.
                let (max_lower, max_higher) = if si.log_max() <= bound.log_max() {
                    (si, bound)
                } else {
                    (bound, si)
                };
                (max_lower + SafeInt::one()) + max_higher
            })
            .collect();
        let lower_bound_commitment = lower_bound_offset.commit();
        let lower_bound_result: Vec<bool> =
            input.clone().into_iter().map(|s| s >= -bound).collect();

        let lower_bound_proof = PositiveProof::prove(
            transcript,
            (lower_bound_offset,),
            lower_bound_result.into(),
            (lower_bound_commitment,),
        );

        let upper_bound_offset: SafeIntColumn = input
            .clone()
            .into_iter()
            .map(|si| {
                let (max_lower, max_higher) = if si.log_max() <= bound.log_max() {
                    (-si, bound)
                } else {
                    (bound, -si)
                };
                (max_lower + SafeInt::one()) + max_higher
            })
            .collect();
        let upper_bound_commitment = upper_bound_offset.commit();
        let upper_bound_result: Vec<bool> = input.into_iter().map(|s| s <= bound).collect();

        let upper_bound_proof = PositiveProof::prove(
            transcript,
            (upper_bound_offset,),
            upper_bound_result.into(),
            (upper_bound_commitment,),
        );

        let c_range = output.commit();

        BinaryRangeProof {
            lower_bound_proof,
            upper_bound_proof,
            c_range,
        }
    }
}

impl<const B: u8> PipVerify<(Commitment,), Commitment> for BinaryRangeProof<B> {
    fn verify(
        &self,
        //The merlin transcript for the verifier
        transcript: &mut Transcript,
        //The commitments of the inputs to the PIP. Typically, these are known by the verifier.
        (c_in,): (Commitment,),
    ) -> Result<(), ProofError> {
        let c_in_log_max = c_in.log_max.ok_or(ProofError::FormatError)?;
        assert!(c_in_log_max.max(B) <= 249);
        assert!(c_in_log_max.min(B) <= 248);

        let bound: SafeIntColumn = repeat(
            SafeInt::from(2)
                .try_pow(B)
                .expect("the log_max shouldn't exceed 251 since B <= 250"),
        )
        .take(c_in.length)
        .collect();

        let one: SafeIntColumn = repeat(SafeInt::one()).take(c_in.length).collect();
        transcript
            .append_auto(MessageLabel::BinaryRange, &B)
            .unwrap();
        let c_bound: Commitment = bound.commit();
        let c_one: Commitment = one.commit();

        let lower_bound_input_commitment = {
            let (c_max_lower, c_max_higher) = if c_in_log_max <= B {
                (c_in, c_bound)
            } else {
                (c_bound, c_in)
            };

            (c_max_lower + c_one) + c_max_higher
        };

        let upper_bound_input_commitment = {
            let (c_max_lower, c_max_higher) = if c_in_log_max <= B {
                (-c_in, c_bound)
            } else {
                (c_bound, -c_in)
            };

            (c_max_lower + c_one) + c_max_higher
        };

        self.lower_bound_proof
            .verify(transcript, (lower_bound_input_commitment,))?;
        self.upper_bound_proof
            .verify(transcript, (upper_bound_input_commitment,))?;

        Ok(())
    }

    fn get_output_commitments(&self) -> Commitment {
        self.c_range
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use curve25519_dalek::scalar::Scalar;

    #[test]
    fn test_bin_range() {
        let a: SafeIntColumn = vec![1, 2, -4, 5, 4, -1, -5, 0]
            .into_iter()
            .map(SafeInt::from)
            .collect();
        let range: Column<_> = vec![true, true, true, false, true, true, false, true].into();

        let c_a = a.commit();
        let c_range = range.commit();

        let mut transcript = Transcript::new(b"binrangetest");
        let proof = BinaryRangeProof::<2>::prove(&mut transcript, (a,), range, (c_a,));

        let mut transcript = Transcript::new(b"binrangetest");
        assert!(proof.verify(&mut transcript, (c_a,)).is_ok());

        // correct output commitment
        assert_eq!(proof.get_output_commitments(), c_range);

        let b: SafeIntColumn = vec![1, 2, -4, 5, 4, -1, -5, 1]
            .into_iter()
            .map(SafeInt::from)
            .collect();
        let c_b = b.commit();

        // wrong input commitments
        let mut transcript = Transcript::new(b"binrangetest");
        assert!(proof.verify(&mut transcript, (c_b,)).is_err());
    }

    #[test]
    fn test_bin_range_wrong() {
        let a: SafeIntColumn = vec![1, 2, 3, 17, -1, -5, 0]
            .into_iter()
            .map(SafeInt::from)
            .collect();
        let range: Column<_> = vec![true, true, true, true, true, true, true].into();

        let c_a = a.commit();

        let mut transcript = Transcript::new(b"binrangetest");
        let proof = BinaryRangeProof::<4>::prove(&mut transcript, (a,), range, (c_a,));

        assert!(proof.verify(&mut transcript, (c_a,)).is_err());
    }

    #[test]
    fn test_bin_range_larger_numbers() {
        let a = SafeIntColumn::try_new(
            vec![
                Scalar::from(300u32),
                -Scalar::from(98u32),
                Scalar::from(0u32),
                Scalar::from(513u32),
                Scalar::from(512u32),
                -Scalar::from(512u32),
            ],
            10,
        )
        .unwrap();
        let range: Column<_> = vec![true, true, true, false, true, true].into();

        let c_a = a.commit();
        let c_range = range.commit();

        let mut transcript = Transcript::new(b"binrangetest");
        let proof = BinaryRangeProof::<9>::prove(&mut transcript, (a,), range, (c_a,));

        let mut transcript = Transcript::new(b"binrangetest");
        assert!(proof.verify(&mut transcript, (c_a,)).is_ok());

        // correct output commitment
        assert_eq!(proof.get_output_commitments(), c_range);
    }

    #[test]
    fn test_bin_range_maximum_case() {
        let pow_248 = Scalar::from_bytes_mod_order([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ]);

        let pow_249 = Scalar::from_bytes_mod_order([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 2,
        ]);
        let a = SafeIntColumn::try_new(
            vec![
                pow_249,
                Scalar::zero(),
                -pow_249,
                pow_248,
                -pow_248,
                pow_248 + Scalar::one(),
            ],
            249,
        )
        .unwrap();

        let range: Column<_> = vec![false, true, false, true, true, false].into();

        let c_a = a.commit();
        let c_range = range.commit();

        let mut transcript = Transcript::new(b"binrangetest");
        let proof = BinaryRangeProof::<248>::prove(&mut transcript, (a,), range, (c_a,));

        let mut transcript = Transcript::new(b"binrangetest");
        assert!(proof.verify(&mut transcript, (c_a,)).is_ok());

        // correct output commitment
        assert_eq!(proof.get_output_commitments(), c_range);
    }

    #[test]
    #[should_panic]
    fn test_bin_range_out_of_bounds() {
        let pow_248 = Scalar::from_bytes_mod_order([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 1,
        ]);

        let pow_249 = Scalar::from_bytes_mod_order([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 2,
        ]);
        let a = SafeIntColumn::try_new(
            vec![
                pow_249,
                Scalar::zero(),
                -pow_249,
                pow_248,
                -pow_248,
                pow_248 + Scalar::one(),
            ],
            249,
        )
        .unwrap();

        let range: Column<_> = vec![true, true, true, true, true, true].into();

        let c_a = a.commit();

        let mut transcript = Transcript::new(b"binrangetest");
        let _should_panic = BinaryRangeProof::<249>::prove(&mut transcript, (a,), range, (c_a,));
    }
}
