use crate::{
    base::{
        proof::{Commitment, PipProve, PipVerify, ProofError, ProofResult, Transcript},
        scalar::SafeIntColumn,
    },
    pip::range::LogMaxReductionProof,
};

/// Helper function for performing and proving log max reduction if necessary.
///
/// If the log_max of the output commitment is higher than the const generic parameter `B`,
/// reduction is performed.
/// The result is the new `c_output` with its log max reduced, and a [LogMaxReductionProof].
///
/// If no reduction is necessary, the unchanged `c_output` and [None] are returned instead.
pub fn reduce_and_prove_if_necessary<const B: u8>(
    transcript: &mut Transcript,
    output: SafeIntColumn,
    c_output: Commitment,
) -> (Commitment, Option<LogMaxReductionProof<B>>) {
    let output_log_max = c_output
        .log_max
        .expect("commitments of SafeIntColumns should have a log_max");

    if output_log_max > B {
        let output_unreduced: SafeIntColumn = SafeIntColumn::try_new(
            output.clone().into_iter().map(|s| s.value()).collect(),
            output_log_max,
        )
        .unwrap();

        let log_max_reduction_proof = Some(LogMaxReductionProof::<{ B }>::prove(
            transcript,
            (output_unreduced,),
            output,
            (c_output,),
        ));

        let c_output_reduced = c_output.with_log_max(B);

        (c_output_reduced, log_max_reduction_proof)
    } else {
        (c_output, None)
    }
}

/// Helper function for verifying an arithmetic proof with log max reduction if necessary.
///
/// This function takes two [Commitment] parameters.
/// `c_output` is the commitment to the output stored on the proof which should already be reduced.
/// `c_output_calculated` on the other hand, should be the result of applying the arithmetic
/// operation on the two input commitments, without reducing.
///
/// The function will determine if log max reduction is required.
/// If it is, and a [LogMaxReductionProof] isn't provided, it will return a
/// [ProofError::VerificationError].
/// If a [LogMaxReductionProof] is provided, it will be verified, whether or not it is required.
pub fn verify_with_reduction_if_necessary<const B: u8>(
    transcript: &mut Transcript,
    c_output: &Commitment,
    log_max_reduction_proof: &Option<LogMaxReductionProof<B>>,
    c_output_calculated: Commitment,
) -> ProofResult<()> {
    let calculated_log_max = c_output_calculated.log_max.ok_or(ProofError::FormatError)?;

    let output_log_max = c_output.log_max.ok_or(ProofError::FormatError)?;

    let maybe_log_max_reduction_proof = if calculated_log_max > B {
        // Proof should have a reduction, error if it doesn't
        Some(
            log_max_reduction_proof
                .as_ref()
                .ok_or(ProofError::VerificationError)?,
        )
    } else {
        // Proof doesn't need a reduction, but might have one anyway
        log_max_reduction_proof.as_ref()
    };

    if let Some(log_max_reduction_proof) = maybe_log_max_reduction_proof {
        // Proof has a reduction. Whether or not it's required, verify it

        // verify that the commitment log_max has been reduced
        if output_log_max != B {
            return Err(ProofError::VerificationError);
        }

        // verify the inner proof
        log_max_reduction_proof.verify(transcript, (c_output_calculated,))?;

        // verify the LogMaxReductionProof is actually verifying against the correct output commitment
        if c_output_calculated != log_max_reduction_proof.get_output_commitments() {
            return Err(ProofError::VerificationError);
        }
    }

    // Verify the provided output commitment
    if c_output_calculated != *c_output {
        return Err(ProofError::VerificationError);
    }

    Ok(())
}
