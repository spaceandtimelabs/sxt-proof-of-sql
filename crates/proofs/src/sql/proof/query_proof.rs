use super::{
    compute_evaluation_vector, CountBuilder, ProofBuilder, ProofCounts, ProofExpr,
    ProvableQueryResult, QueryResult, SumcheckMleEvaluations, SumcheckRandomScalars, TransformExpr,
    VerificationBuilder,
};

use crate::base::slice_ops;
use crate::base::{
    database::{CommitmentAccessor, DataAccessor},
    polynomial::CompositePolynomialInfo,
    proof::{MessageLabel, ProofError, TranscriptProtocol},
};
use crate::proof_primitive::sumcheck::SumcheckProof;
use blitzar::proof::InnerProductProof;
use num_traits::Zero;

use crate::base::math::log2_up;
use crate::base::scalar::ArkScalar;
use bumpalo::Bump;
use byte_slice_cast::AsByteSlice;
use curve25519_dalek::ristretto::CompressedRistretto;
use merlin::Transcript;
use serde::{Deserialize, Serialize};
use std::cmp;

/// The proof for a query.
///
/// Note: Because the class is deserialized from untrusted data, it
/// cannot maintain any invariant on its data members; hence, they are
/// all public so as to allow for easy manipulation for testing.
#[derive(Clone, Serialize, Deserialize)]
pub struct QueryProof {
    pub commitments: Vec<CompressedRistretto>,
    pub sumcheck_proof: SumcheckProof,
    pub pre_result_mle_evaluations: Vec<ArkScalar>,
    pub evaluation_proof: InnerProductProof,
}

impl QueryProof {
    #[tracing::instrument(name = "proofs.sql.proof.query_proof.new", level = "info", skip_all)]
    pub fn new(expr: &impl ProofExpr, accessor: &impl DataAccessor) -> (Self, ProvableQueryResult) {
        let table_length = expr.get_length(accessor);
        let generator_offset = expr.get_offset(accessor);
        let num_sumcheck_variables = cmp::max(log2_up(table_length), 1);
        assert!(num_sumcheck_variables > 0);

        let alloc = Bump::new();

        // pass over provable AST to fill in the proof builder
        let mut builder = ProofBuilder::new(table_length, num_sumcheck_variables);
        expr.prover_evaluate(&mut builder, &alloc, accessor);

        // commit to any intermediate MLEs
        let commitments = builder.commit_intermediate_mles(generator_offset);

        // compute the query's result
        let provable_result = builder.make_provable_query_result();

        // construct a transcript for the proof
        let mut transcript = make_transcript(
            &commitments,
            &provable_result.indexes,
            &provable_result.data,
        );

        // construct the sumcheck polynomial
        let num_random_scalars = num_sumcheck_variables + builder.num_sumcheck_subpolynomials();
        let mut random_scalars = vec![Zero::zero(); num_random_scalars];
        transcript.challenge_ark_scalars(&mut random_scalars, MessageLabel::QuerySumcheckChallenge);
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
        transcript.append_ark_scalars(
            MessageLabel::QueryMleEvaluations,
            &pre_result_mle_evaluations,
        );

        // fold together the pre result MLEs -- this will form the input to an inner product proof
        // of their evaluations (fold in this context means create a random linear combination)
        let mut random_scalars = vec![Zero::zero(); pre_result_mle_evaluations.len()];
        transcript.challenge_ark_scalars(
            &mut random_scalars,
            MessageLabel::QueryMleEvaluationsChallenge,
        );
        let folded_mle = builder.fold_pre_result_mles(&random_scalars);

        // finally, form the inner product proof of the MLEs' evaluations
        let evaluation_proof = InnerProductProof::create(
            &mut transcript,
            &slice_ops::slice_cast(&folded_mle),
            &slice_ops::slice_cast(&evaluation_vec),
            generator_offset as u64,
        );

        let proof = Self {
            commitments,
            sumcheck_proof,
            pre_result_mle_evaluations,
            evaluation_proof,
        };
        (proof, provable_result)
    }

    #[tracing::instrument(
        name = "proofs.sql.proof.query_proof.verify",
        level = "info",
        skip_all,
        err
    )]
    pub fn verify(
        &self,
        expr: &(impl ProofExpr + TransformExpr),
        accessor: &impl CommitmentAccessor,
        result: &ProvableQueryResult,
    ) -> Result<QueryResult, ProofError> {
        let table_length = expr.get_length(accessor);
        let generator_offset = expr.get_offset(accessor);
        let num_sumcheck_variables = cmp::max(log2_up(table_length), 1);
        assert!(num_sumcheck_variables > 0);

        // count terms
        let counts = {
            let mut builder = CountBuilder::new();
            expr.count(&mut builder, accessor)?;
            builder.counts()
        }?;

        // verify sizes
        if !self.validate_sizes(&counts, result) {
            return Err(ProofError::VerificationError("invalid proof size"));
        }

        // decompress commitments
        let mut commitments = Vec::with_capacity(self.commitments.len());
        for commitment in self.commitments.iter() {
            if let Some(commitment) = commitment.decompress() {
                commitments.push(commitment);
            } else {
                return Err(ProofError::VerificationError(
                    "commitment failed to decompress",
                ));
            }
        }

        // construct a transcript for the proof
        let mut transcript = make_transcript(&self.commitments, &result.indexes, &result.data);

        // draw the random scalars for sumcheck
        let num_random_scalars = num_sumcheck_variables + counts.sumcheck_subpolynomials;
        let mut random_scalars = vec![Zero::zero(); num_random_scalars];
        transcript.challenge_ark_scalars(&mut random_scalars, MessageLabel::QuerySumcheckChallenge);
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
        transcript.append_ark_scalars(
            MessageLabel::QueryMleEvaluations,
            &self.pre_result_mle_evaluations,
        );

        // draw the random scalars for the evaluation proof
        // (i.e. the folding/random linear combination of the pre_result_mles)
        let mut evaluation_random_scalars =
            vec![Zero::zero(); self.pre_result_mle_evaluations.len()];
        transcript.challenge_ark_scalars(
            &mut evaluation_random_scalars,
            MessageLabel::QueryMleEvaluationsChallenge,
        );

        let column_result_fields = expr.get_column_result_fields();

        // compute the evaluation of the result MLEs
        let result_evaluations = match result.evaluate(&evaluation_vec, &column_result_fields[..]) {
            Some(evaluations) => evaluations,
            _ => {
                return Err(ProofError::VerificationError(
                    "failed to evaluate intermediate result MLEs",
                ))
            }
        };

        // pass over the provable AST to fill in the verification builder
        let sumcheck_evaluations = SumcheckMleEvaluations::new(
            table_length,
            &subclaim.evaluation_point,
            &sumcheck_random_scalars,
            &self.pre_result_mle_evaluations,
            &result_evaluations,
        );
        let mut builder = VerificationBuilder::new(
            generator_offset,
            sumcheck_evaluations,
            &commitments,
            sumcheck_random_scalars.subpolynomial_multipliers,
            &evaluation_random_scalars,
        );
        expr.verifier_evaluate(&mut builder, accessor);

        // perform the evaluation check of the sumcheck polynomial
        if builder.sumcheck_evaluation() != subclaim.expected_evaluation {
            return Err(ProofError::VerificationError(
                "sumcheck evaluation check failed",
            ));
        }

        // finally, check the MLE evaluations with the inner product proof
        let product = builder.folded_pre_result_evaluation();
        let expected_commit = builder.folded_pre_result_commitment();
        self.evaluation_proof
            .verify(
                &mut transcript,
                &expected_commit,
                &product.into(),
                &slice_ops::slice_cast(&evaluation_vec),
                generator_offset as u64,
            )
            .map_err(|_e| {
                ProofError::VerificationError("Inner product proof of MLE evaluations failed")
            })?;

        Ok(result
            .into_query_result(&column_result_fields[..])
            .map(|batch| expr.transform_results(batch)))
    }

    fn validate_sizes(&self, counts: &ProofCounts, result: &ProvableQueryResult) -> bool {
        result.num_columns as usize == counts.result_columns
            && self.commitments.len() == counts.intermediate_mles
            && self.pre_result_mle_evaluations.len()
                == counts.intermediate_mles + counts.anchored_mles
    }
}

#[tracing::instrument(
    name = "proofs.sql.proof.query_proof.make_transcript",
    level = "debug",
    skip_all
)]
fn make_transcript(
    commitments: &[CompressedRistretto],
    result_indexes: &[u64],
    result_data: &[u8],
) -> merlin::Transcript {
    let mut transcript = Transcript::new(MessageLabel::QueryProof.as_bytes());
    transcript.append_points(MessageLabel::QueryCommit, commitments);
    transcript.append_message(
        MessageLabel::QueryResultIndexes.as_bytes(),
        result_indexes.as_byte_slice(),
    );
    transcript.append_message(MessageLabel::QueryResultData.as_bytes(), result_data);
    transcript
}
