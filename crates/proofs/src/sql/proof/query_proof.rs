use super::{
    CountBuilder, ProofBuilder, ProofCounts, ProofExpr, ProvableQueryResult, QueryResult,
    SumcheckMleEvaluations, SumcheckRandomScalars, VerificationBuilder,
};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::{Commitment, CommitmentEvaluationProof, VecCommitmentExt},
        database::{CommitmentAccessor, DataAccessor},
        math::log2_up,
        polynomial::{compute_evaluation_vector, CompositePolynomialInfo},
        proof::{MessageLabel, ProofError, TranscriptProtocol},
    },
    proof_primitive::sumcheck::SumcheckProof,
    sql::proof::{QueryData, ResultBuilder},
};
use bumpalo::Bump;
use merlin::Transcript;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use std::cmp;

/// The proof for a query.
///
/// Note: Because the class is deserialized from untrusted data, it
/// cannot maintain any invariant on its data members; hence, they are
/// all public so as to allow for easy manipulation for testing.
#[derive(Clone, Serialize, Deserialize)]
pub struct QueryProof<CP: CommitmentEvaluationProof> {
    /// TODO: add docs
    pub bit_distributions: Vec<BitDistribution>,
    /// TODO: add docs
    pub commitments: Vec<CP::Commitment>,
    /// TODO: add docs
    pub sumcheck_proof: SumcheckProof<CP::Scalar>,
    /// TODO: add docs
    pub pre_result_mle_evaluations: Vec<CP::Scalar>,
    /// TODO: add docs
    pub evaluation_proof: CP,
}

impl<CP: CommitmentEvaluationProof> QueryProof<CP> {
    /// TODO: add docs
    #[tracing::instrument(name = "proofs.sql.proof.query_proof.new", level = "info", skip_all)]
    pub fn new(
        expr: &(impl ProofExpr<CP::Commitment> + Serialize),
        accessor: &impl DataAccessor<CP::Scalar>,
        setup: &CP::ProverPublicSetup,
    ) -> (Self, ProvableQueryResult) {
        let table_length = expr.get_length(accessor);
        let num_sumcheck_variables = cmp::max(log2_up(table_length), 1);
        let generator_offset = expr.get_offset(accessor);
        assert!(num_sumcheck_variables > 0);

        let alloc = Bump::new();
        let mut result_builder = ResultBuilder::new(table_length);
        expr.result_evaluate(&mut result_builder, &alloc, accessor);
        let provable_result = result_builder.make_provable_query_result();

        // construct a transcript for the proof
        let mut transcript: Transcript =
            make_transcript(expr, &provable_result, table_length, generator_offset);

        // These are the challenges that will be consumed by the proof
        // Specifically, these are the challenges that the verifier sends to
        // the prover after the prover sends the result, but before the prover
        // send commitments to the intermediate witness columns.
        // Note: the last challenge in the vec is the first one that is consumed.
        let mut post_result_challenges =
            vec![Zero::zero(); result_builder.num_post_result_challenges()];
        transcript.challenge_ark(
            &mut post_result_challenges,
            MessageLabel::PostResultChallenges,
        );

        let mut builder =
            ProofBuilder::new(table_length, num_sumcheck_variables, post_result_challenges);
        expr.prover_evaluate(&mut builder, &alloc, accessor);

        let proof = QueryProof::new_from_builder(builder, generator_offset, transcript, setup);
        (proof, provable_result)
    }

    pub(crate) fn new_from_builder(
        builder: ProofBuilder<CP::Scalar>,
        generator_offset: usize,
        mut transcript: Transcript,
        setup: &CP::ProverPublicSetup,
    ) -> Self {
        let num_sumcheck_variables = builder.num_sumcheck_variables();
        let table_length = builder.table_length();

        // commit to any intermediate MLEs
        let commitments = builder.commit_intermediate_mles(generator_offset, setup);

        // add the commitments and bit distributions to the proof
        extend_transcript(&mut transcript, &commitments, builder.bit_distributions());

        // construct the sumcheck polynomial
        let num_random_scalars = num_sumcheck_variables + builder.num_sumcheck_subpolynomials();
        let mut random_scalars = vec![Zero::zero(); num_random_scalars];
        transcript.challenge_ark(&mut random_scalars, MessageLabel::QuerySumcheckChallenge);
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
        let pre_result_mle_evaluations = builder.evaluate_pre_result_mles(&evaluation_vec);

        // commit to the MLE evaluations
        transcript.append_canonical_serialize(
            MessageLabel::QueryMleEvaluations,
            &pre_result_mle_evaluations,
        );

        // fold together the pre result MLEs -- this will form the input to an inner product proof
        // of their evaluations (fold in this context means create a random linear combination)
        let mut random_scalars = vec![Zero::zero(); pre_result_mle_evaluations.len()];
        transcript.challenge_ark(
            &mut random_scalars,
            MessageLabel::QueryMleEvaluationsChallenge,
        );
        let folded_mle = builder.fold_pre_result_mles(&random_scalars);

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
            pre_result_mle_evaluations,
            evaluation_proof,
        };
        proof
    }

    #[tracing::instrument(
        name = "proofs.sql.proof.query_proof.verify",
        level = "info",
        skip_all,
        err
    )]
    /// Verify a `QueryProof`. Note: This does NOT transform the result!
    pub fn verify(
        &self,
        expr: &(impl ProofExpr<CP::Commitment> + Serialize),
        accessor: &impl CommitmentAccessor<CP::Commitment>,
        result: &ProvableQueryResult,
        setup: &CP::VerifierPublicSetup,
    ) -> QueryResult<CP::Scalar> {
        let table_length = expr.get_length(accessor);
        let generator_offset = expr.get_offset(accessor);
        let num_sumcheck_variables = cmp::max(log2_up(table_length), 1);
        assert!(num_sumcheck_variables > 0);

        // validate bit decompositions
        for dist in self.bit_distributions.iter() {
            if !dist.is_valid() {
                Err(ProofError::VerificationError("invalid bit distributions"))?;
            }
        }

        // count terms
        let counts = {
            let mut builder = CountBuilder::new(&self.bit_distributions);
            expr.count(&mut builder, accessor)?;
            builder.counts()
        }?;

        // verify sizes
        if !self.validate_sizes(&counts, result) {
            Err(ProofError::VerificationError("invalid proof size"))?;
        }

        let commitments =
            self.commitments
                .to_decompressed()
                .ok_or(ProofError::VerificationError(
                    "commitment failed to decompress",
                ))?;

        // construct a transcript for the proof
        let mut transcript = make_transcript(expr, result, table_length, generator_offset);

        // These are the challenges that will be consumed by the proof
        // Specifically, these are the challenges that the verifier sends to
        // the prover after the prover sends the result, but before the prover
        // send commitments to the intermediate witness columns.
        // Note: the last challenge in the vec is the first one that is consumed.
        let mut post_result_challenges = vec![Zero::zero(); counts.post_result_challenges];
        transcript.challenge_ark(
            &mut post_result_challenges,
            MessageLabel::PostResultChallenges,
        );

        // add the commitments and bit disctibutions to the proof
        extend_transcript(&mut transcript, &self.commitments, &self.bit_distributions);

        // draw the random scalars for sumcheck
        let num_random_scalars = num_sumcheck_variables + counts.sumcheck_subpolynomials;
        let mut random_scalars = vec![Zero::zero(); num_random_scalars];
        transcript.challenge_ark(&mut random_scalars, MessageLabel::QuerySumcheckChallenge);
        let sumcheck_random_scalars =
            SumcheckRandomScalars::new(&random_scalars, table_length, num_sumcheck_variables);

        // verify sumcheck up to the evaluation check
        let poly_info = CompositePolynomialInfo {
            max_multiplicands: counts.sumcheck_max_multiplicands,
            num_variables: num_sumcheck_variables,
        };
        let subclaim = self.sumcheck_proof.verify_without_evaluation(
            &mut transcript,
            poly_info,
            &Zero::zero(),
        )?;
        // evaluate the MLEs used in sumcheck except for the result columns
        let mut evaluation_vec = vec![Zero::zero(); table_length];
        compute_evaluation_vector(&mut evaluation_vec, &subclaim.evaluation_point);

        // commit to mle evaluations
        transcript.append_canonical_serialize(
            MessageLabel::QueryMleEvaluations,
            &self.pre_result_mle_evaluations,
        );

        // draw the random scalars for the evaluation proof
        // (i.e. the folding/random linear combination of the pre_result_mles)
        let mut evaluation_random_scalars =
            vec![Zero::zero(); self.pre_result_mle_evaluations.len()];
        transcript.challenge_ark(
            &mut evaluation_random_scalars,
            MessageLabel::QueryMleEvaluationsChallenge,
        );

        let column_result_fields = expr.get_column_result_fields();

        // compute the evaluation of the result MLEs
        let result_evaluations = match result.evaluate(&evaluation_vec, &column_result_fields[..]) {
            Some(evaluations) => evaluations,
            _ => Err(ProofError::VerificationError(
                "failed to evaluate intermediate result MLEs",
            ))?,
        };

        // pass over the provable AST to fill in the verification builder
        let sumcheck_evaluations = SumcheckMleEvaluations::new(
            table_length,
            &subclaim.evaluation_point,
            &sumcheck_random_scalars,
            &self.pre_result_mle_evaluations,
            &result_evaluations,
            result.indexes(),
        );
        let mut builder = VerificationBuilder::new(
            generator_offset,
            sumcheck_evaluations,
            &self.bit_distributions,
            &commitments,
            sumcheck_random_scalars.subpolynomial_multipliers,
            &evaluation_random_scalars,
            post_result_challenges,
        );
        expr.verifier_evaluate(&mut builder, accessor)?;

        // perform the evaluation check of the sumcheck polynomial
        if builder.sumcheck_evaluation() != subclaim.expected_evaluation {
            Err(ProofError::VerificationError(
                "sumcheck evaluation check failed",
            ))?;
        }

        // finally, check the MLE evaluations with the inner product proof
        let product = builder.folded_pre_result_evaluation();
        let expected_commit = builder.compute_folded_pre_result_commitment();
        self.evaluation_proof
            .verify_proof(
                &mut transcript,
                &expected_commit,
                &product,
                &subclaim.evaluation_point,
                generator_offset as u64,
                table_length,
                setup,
            )
            .map_err(|_e| {
                ProofError::VerificationError("Inner product proof of MLE evaluations failed")
            })?;

        let mut verification_hash = [0u8; 32];
        transcript.challenge_bytes(
            MessageLabel::VerificationHash.as_bytes(),
            &mut verification_hash,
        );
        result
            .into_owned_table(&column_result_fields[..])
            .map(|table| QueryData {
                table,
                verification_hash,
            })
    }

    fn validate_sizes(&self, counts: &ProofCounts, result: &ProvableQueryResult) -> bool {
        result.num_columns() == counts.result_columns
            && self.commitments.num_commitments() == counts.intermediate_mles
            && self.pre_result_mle_evaluations.len()
                == counts.intermediate_mles + counts.anchored_mles
    }
}

#[tracing::instrument(
    name = "proofs.sql.proof.query_proof.make_transcript",
    level = "debug",
    skip_all
)]

/// Creates a transcript using the Merlin library.
///
/// This function is used to produce a transcript for a proof expression
/// and a provable query result, along with additional parameters like
/// table length and generator offset. The transcript is constructed
/// with all protocol public inputs appended to it.
///
/// # Arguments
///
/// * `expr` - A reference to an object that implements `ProofExpr` and `Serialize`.
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
pub fn make_transcript<C: Commitment>(
    expr: &(impl ProofExpr<C> + Serialize),
    result: &ProvableQueryResult,
    table_length: usize,
    generator_offset: usize,
) -> merlin::Transcript {
    let mut transcript = Transcript::new(MessageLabel::QueryProof.as_bytes());
    transcript.append_auto(MessageLabel::QueryResultData, result);
    transcript.append_auto(MessageLabel::ProofExpr, expr);
    transcript.append_auto(MessageLabel::TableLength, &table_length);
    transcript.append_auto(MessageLabel::GeneratorOffset, &generator_offset);
    transcript
}

fn extend_transcript<C: serde::Serialize>(
    transcript: &mut Transcript,
    commitments: &C,
    bit_distributions: &[BitDistribution],
) {
    transcript.append_auto(MessageLabel::QueryCommit, commitments);
    transcript.append_auto(MessageLabel::QueryBitDistributions, bit_distributions);
}
