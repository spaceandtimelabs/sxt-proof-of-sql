use super::{
    CountBuilder, FinalRoundBuilder, ProofCounts, ProofPlan, ProvableQueryResult, QueryResult,
    SumcheckMleEvaluations, SumcheckRandomScalars, VerificationBuilder,
};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::{Commitment, CommitmentEvaluationProof},
        database::{Column, CommitmentAccessor, DataAccessor},
        math::log2_up,
        polynomial::{compute_evaluation_vector, CompositePolynomialInfo},
        proof::{Keccak256Transcript, ProofError, Transcript},
    },
    proof_primitive::sumcheck::SumcheckProof,
    sql::proof::{FirstRoundBuilder, QueryData},
};
use alloc::{vec, vec::Vec};
use bumpalo::Bump;
use core::cmp;
use num_traits::Zero;
use serde::{Deserialize, Serialize};

/// The proof for a query.
///
/// Note: Because the class is deserialized from untrusted data, it
/// cannot maintain any invariant on its data members; hence, they are
/// all public so as to allow for easy manipulation for testing.
#[derive(Clone, Serialize, Deserialize)]
pub struct QueryProof<CP: CommitmentEvaluationProof> {
    /// Bit distributions
    pub bit_distributions: Vec<BitDistribution>,
    /// Commitments
    pub commitments: Vec<CP::Commitment>,
    /// Sumcheck Proof
    pub sumcheck_proof: SumcheckProof<CP::Scalar>,
    /// MLEs used in sumcheck except for the result columns
    pub pcs_proof_evaluations: Vec<CP::Scalar>,
    /// Inner product proof of the MLEs' evaluations
    pub evaluation_proof: CP,
}

impl<CP: CommitmentEvaluationProof> QueryProof<CP> {
    /// Create a new `QueryProof`.
    #[tracing::instrument(name = "QueryProof::new", level = "debug", skip_all)]
    pub fn new(
        expr: &(impl ProofPlan<CP::Commitment> + Serialize),
        accessor: &impl DataAccessor<CP::Scalar>,
        setup: &CP::ProverPublicSetup<'_>,
    ) -> (Self, ProvableQueryResult) {
        let alloc = Bump::new();
        // TODO: Modify this to handle multiple tables
        let table_length = expr.get_input_lengths(&alloc, accessor)[0];
        let num_sumcheck_variables = cmp::max(log2_up(table_length), 1);
        let generator_offset = expr.get_offset(accessor);
        assert!(num_sumcheck_variables > 0);

        // Evaluate query result
        let result_cols = expr.result_evaluate(&[table_length], &alloc, accessor);
        let output_length = result_cols.first().map_or(0, Column::len);
        let provable_result = ProvableQueryResult::new(output_length as u64, &result_cols);

        // Prover First Round
        let mut first_round_builder = FirstRoundBuilder::new();
        expr.first_round_evaluate(&mut first_round_builder);

        // construct a transcript for the proof
        let mut transcript: Keccak256Transcript =
            make_transcript(expr, &provable_result, table_length, generator_offset);

        // These are the challenges that will be consumed by the proof
        // Specifically, these are the challenges that the verifier sends to
        // the prover after the prover sends the result, but before the prover
        // send commitments to the intermediate witness columns.
        // Note: the last challenge in the vec is the first one that is consumed.
        let post_result_challenges =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(first_round_builder.num_post_result_challenges())
                .collect();

        let mut builder =
            FinalRoundBuilder::new(table_length, num_sumcheck_variables, post_result_challenges);
        expr.final_round_evaluate(&[table_length], &mut builder, &alloc, accessor);

        let num_sumcheck_variables = builder.num_sumcheck_variables();
        let table_length = builder.table_length();

        // commit to any intermediate MLEs
        let commitments = builder.commit_intermediate_mles(generator_offset, setup);

        // add the commitments and bit distributions to the proof
        extend_transcript(&mut transcript, &commitments, builder.bit_distributions());

        // construct the sumcheck polynomial
        let num_random_scalars = num_sumcheck_variables + builder.num_sumcheck_subpolynomials();
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(num_random_scalars)
                .collect();
        let poly = builder.make_sumcheck_polynomial(&SumcheckRandomScalars::new(
            &random_scalars,
            table_length,
            num_sumcheck_variables,
        ));

        // create the sumcheck proof -- this is the main part of proving a query
        let mut evaluation_point = vec![Zero::zero(); poly.num_variables];
        let sumcheck_proof = SumcheckProof::create(&mut transcript, &mut evaluation_point, &poly);

        // evaluate the MLEs used in sumcheck except for the result columns
        let mut evaluation_vec = vec![Zero::zero(); table_length];
        compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
        let pcs_proof_evaluations = builder.evaluate_pcs_proof_mles(&evaluation_vec);

        // commit to the MLE evaluations
        transcript.extend_canonical_serialize_as_le(&pcs_proof_evaluations);

        // fold together the pre result MLEs -- this will form the input to an inner product proof
        // of their evaluations (fold in this context means create a random linear combination)
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(pcs_proof_evaluations.len())
                .collect();
        let folded_mle = builder.fold_pcs_proof_mles(&random_scalars);

        // finally, form the inner product proof of the MLEs' evaluations
        let evaluation_proof = CP::new(
            &mut transcript,
            &folded_mle,
            &evaluation_point,
            generator_offset as u64,
            setup,
        );

        let proof = Self {
            bit_distributions: builder.bit_distributions().to_vec(),
            commitments,
            sumcheck_proof,
            pcs_proof_evaluations,
            evaluation_proof,
        };
        (proof, provable_result)
    }

    #[tracing::instrument(name = "QueryProof::verify", level = "debug", skip_all, err)]
    /// Verify a `QueryProof`. Note: This does NOT transform the result!
    pub fn verify(
        &self,
        expr: &(impl ProofPlan<CP::Commitment> + Serialize),
        accessor: &impl CommitmentAccessor<CP::Commitment>,
        result: &ProvableQueryResult,
        setup: &CP::VerifierPublicSetup<'_>,
    ) -> QueryResult<CP::Scalar> {
        //TODO: Modify this when we have multiple tables
        assert!(expr.get_table_references().len() == 1);
        let input_length = accessor.get_length(*expr.get_table_references().first().unwrap());
        let output_length = result.table_length();
        let generator_offset = expr.get_offset(accessor);
        let num_sumcheck_variables = cmp::max(log2_up(input_length), 1);
        assert!(num_sumcheck_variables > 0);

        // validate bit decompositions
        for dist in &self.bit_distributions {
            if !dist.is_valid() {
                Err(ProofError::VerificationError {
                    error: "invalid bit distributions",
                })?;
            }
        }

        // count terms
        let counts = {
            let mut builder = CountBuilder::new(&self.bit_distributions);
            expr.count(&mut builder, accessor)?;
            builder.counts()
        }?;

        // verify sizes
        if !self.validate_sizes(&counts) {
            Err(ProofError::VerificationError {
                error: "invalid proof size",
            })?;
        }

        // construct a transcript for the proof
        let mut transcript: Keccak256Transcript =
            make_transcript(expr, result, input_length, generator_offset);

        // These are the challenges that will be consumed by the proof
        // Specifically, these are the challenges that the verifier sends to
        // the prover after the prover sends the result, but before the prover
        // send commitments to the intermediate witness columns.
        // Note: the last challenge in the vec is the first one that is consumed.
        let post_result_challenges =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(counts.post_result_challenges)
                .collect();

        // add the commitments and bit disctibutions to the proof
        extend_transcript(&mut transcript, &self.commitments, &self.bit_distributions);

        // draw the random scalars for sumcheck
        let num_random_scalars = num_sumcheck_variables + counts.sumcheck_subpolynomials;
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(num_random_scalars)
                .collect();
        let sumcheck_random_scalars =
            SumcheckRandomScalars::new(&random_scalars, input_length, num_sumcheck_variables);

        // verify sumcheck up to the evaluation check
        let poly_info = CompositePolynomialInfo {
            // This needs to be at least 2 since `CompositePolynomialBuilder::make_composite_polynomial`
            // always adds a degree 2 term.
            max_multiplicands: core::cmp::max(counts.sumcheck_max_multiplicands, 2),
            num_variables: num_sumcheck_variables,
        };
        let subclaim = self.sumcheck_proof.verify_without_evaluation(
            &mut transcript,
            poly_info,
            &Zero::zero(),
        )?;

        // commit to mle evaluations
        transcript.extend_canonical_serialize_as_le(&self.pcs_proof_evaluations);

        // draw the random scalars for the evaluation proof
        // (i.e. the folding/random linear combination of the pcs_proof_mles)
        let evaluation_random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(self.pcs_proof_evaluations.len())
                .collect();

        let column_result_fields = expr.get_column_result_fields();

        // pass over the provable AST to fill in the verification builder
        let sumcheck_evaluations = SumcheckMleEvaluations::new(
            input_length,
            output_length,
            &subclaim.evaluation_point,
            &sumcheck_random_scalars,
            &self.pcs_proof_evaluations,
        );
        let mut builder = VerificationBuilder::new(
            generator_offset,
            sumcheck_evaluations,
            &self.bit_distributions,
            &self.commitments,
            sumcheck_random_scalars.subpolynomial_multipliers,
            &evaluation_random_scalars,
            post_result_challenges,
        );
        let owned_table_result = result.to_owned_table(&column_result_fields[..])?;
        let verifier_evaluations =
            expr.verifier_evaluate(&mut builder, accessor, Some(&owned_table_result))?;
        // compute the evaluation of the result MLEs
        let result_evaluations = result.evaluate(
            &subclaim.evaluation_point,
            output_length,
            &column_result_fields[..],
        )?;
        // check the evaluation of the result MLEs
        if verifier_evaluations != result_evaluations {
            Err(ProofError::VerificationError {
                error: "result evaluation check failed",
            })?;
        }

        // perform the evaluation check of the sumcheck polynomial
        if builder.sumcheck_evaluation() != subclaim.expected_evaluation {
            Err(ProofError::VerificationError {
                error: "sumcheck evaluation check failed",
            })?;
        }

        // finally, check the MLE evaluations with the inner product proof
        let product = builder.folded_pcs_proof_evaluation();
        self.evaluation_proof
            .verify_batched_proof(
                &mut transcript,
                builder.pcs_proof_commitments(),
                builder.inner_product_multipliers(),
                &product,
                &subclaim.evaluation_point,
                generator_offset as u64,
                input_length,
                setup,
            )
            .map_err(|_e| ProofError::VerificationError {
                error: "Inner product proof of MLE evaluations failed",
            })?;

        let verification_hash = transcript.challenge_as_le();
        Ok(QueryData {
            table: owned_table_result,
            verification_hash,
        })
    }

    fn validate_sizes(&self, counts: &ProofCounts) -> bool {
        self.commitments.len() == counts.intermediate_mles
            && self.pcs_proof_evaluations.len() == counts.intermediate_mles + counts.anchored_mles
    }
}

/// Creates a transcript using the Merlin library.
///
/// This function is used to produce a transcript for a proof expression
/// and a provable query result, along with additional parameters like
/// table length and generator offset. The transcript is constructed
/// with all protocol public inputs appended to it.
///
/// # Arguments
///
/// * `expr` - A reference to an object that implements `ProofPlan` and `Serialize`.
///   This is the proof expression which is part of the proof.
///
/// * `result` - A reference to a `ProvableQueryResult`, which is the result
///   of a query that needs to be proven.
///
/// * `table_length` - The length of the table used in the proof, as a `usize`.
///
/// * `generator_offset` - The offset of the generator used in the proof, as a `usize`.
///
/// # Returns
/// This function returns a `merlin::Transcript`. The transcript is a record
/// of all the operations and data involved in creating a proof.
/// ```
fn make_transcript<C: Commitment, T: Transcript>(
    expr: &(impl ProofPlan<C> + Serialize),
    result: &ProvableQueryResult,
    table_length: usize,
    generator_offset: usize,
) -> T {
    let mut transcript = T::new();
    transcript.extend_serialize_as_le(result);
    transcript.extend_serialize_as_le(expr);
    transcript.extend_serialize_as_le(&table_length);
    transcript.extend_serialize_as_le(&generator_offset);
    transcript
}

fn extend_transcript<C: serde::Serialize>(
    transcript: &mut impl Transcript,
    commitments: &C,
    bit_distributions: &[BitDistribution],
) {
    transcript.extend_serialize_as_le(commitments);
    transcript.extend_serialize_as_le(bit_distributions);
}
