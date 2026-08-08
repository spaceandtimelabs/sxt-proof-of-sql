use super::{
    make_sumcheck_state::make_sumcheck_prover_state, FinalRoundBuilder, FirstRoundBuilder,
    ProofPlan, QueryData, QueryResult, SumcheckMleEvaluations, SumcheckRandomScalars,
    VerificationBuilderImpl,
};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::{Commitment, CommitmentEvaluationProof, CommittableColumn},
        database::{
            ColumnRef, CommitmentAccessor, DataAccessor, LiteralValue, MetadataAccessor,
            OwnedTable, Table, TableRef,
        },
        map::{IndexMap, IndexSet},
        math::log2_up,
        polynomial::{compute_evaluation_vector, MultilinearExtension},
        proof::{Keccak256Transcript, PlaceholderResult, ProofError, Transcript},
    },
    proof_primitive::sumcheck::SumcheckProof,
    utils::log,
};
use alloc::{boxed::Box, vec, vec::Vec};
use bumpalo::Bump;
use core::cmp;
use itertools::Itertools;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Ident;
use tracing::{span, Level};

const SETUP_HASH: [u8; 32] = [
    0xe8, 0x84, 0x0d, 0x8a, 0x41, 0xce, 0x9d, 0x4e, 0x14, 0xe7, 0xba, 0x0e, 0x1b, 0x02, 0x32, 0x24,
    0x75, 0x13, 0x61, 0x57, 0x73, 0x78, 0x29, 0x1f, 0xcd, 0x3f, 0x0f, 0x05, 0xf0, 0xf7, 0xe8, 0x75,
]; // TODO: make this different for each setup

/// Return the row number range of tables referenced in the Query
///
/// Basically we are looking for the smallest offset and the largest offset + length
/// so that we have an index range of the table rows that the query is referencing.
fn get_index_range<'a>(
    accessor: &dyn MetadataAccessor,
    table_refs: impl IntoIterator<Item = &'a TableRef>,
) -> (usize, usize) {
    table_refs
        .into_iter()
        .map(|table_ref| {
            let length = accessor.get_length(table_ref);
            let offset = accessor.get_offset(table_ref);
            (offset, offset + length)
        })
        .reduce(|(min_start, max_end), (start, end)| (min_start.min(start), max_end.max(end)))
        // Only applies to `EmptyExec` where there are no tables
        .unwrap_or((0, 1))
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FirstRoundMessage<C> {
    /// Length of the range of generators we use
    pub range_length: usize,
    pub post_result_challenge_count: usize,
    /// Chi evaluation lengths
    pub chi_evaluation_lengths: Vec<usize>,
    /// Rho evaluation lengths
    pub rho_evaluation_lengths: Vec<usize>,
    /// First Round Commitments
    pub round_commitments: Vec<C>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FinalRoundMessage<C> {
    pub subpolynomial_constraint_count: usize,
    /// Final Round Commitments
    pub round_commitments: Vec<C>,
    /// Bit distributions
    pub bit_distributions: Vec<BitDistribution>,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct QueryProofPCSProofEvaluations<S> {
    /// MLEs used in first round sumcheck except for the result columns
    pub first_round: Vec<S>,
    /// evaluations of the columns referenced in the query
    pub column_ref: Vec<S>,
    /// MLEs used in final round sumcheck except for the result columns
    pub final_round: Vec<S>,
}

/// The proof for a query.
///
/// Note: Because the class is deserialized from untrusted data, it
/// cannot maintain any invariant on its data members; hence, they are
/// all public so as to allow for easy manipulation for testing.
#[derive(Clone, Serialize, Deserialize)]
pub struct QueryProof<CP: CommitmentEvaluationProof> {
    pub(super) first_round_message: FirstRoundMessage<CP::Commitment>,
    pub(super) final_round_message: FinalRoundMessage<CP::Commitment>,
    /// Sumcheck Proof
    pub(super) sumcheck_proof: SumcheckProof<CP::Scalar>,
    pub(super) pcs_proof_evaluations: QueryProofPCSProofEvaluations<CP::Scalar>,
    /// Inner product proof of the MLEs' evaluations
    pub(super) evaluation_proof: CP,
}

impl<CP: CommitmentEvaluationProof> QueryProof<CP> {
    /// Create a new `QueryProof`.
    #[tracing::instrument(name = "QueryProof::new", level = "debug", skip_all)]
    #[expect(clippy::too_many_lines)]
    pub fn new(
        expr: &(impl ProofPlan + Serialize),
        accessor: &impl DataAccessor<CP::Scalar>,
        setup: &CP::ProverPublicSetup<'_>,
        params: &[LiteralValue],
    ) -> PlaceholderResult<(Self, OwnedTable<CP::Scalar>)> {
        log::log_memory_usage("Start");

        let (min_row_num, max_row_num) = get_index_range(accessor, &expr.get_table_references());
        let initial_range_length = (max_row_num - min_row_num).max(1);
        let alloc = Bump::new();

        let total_col_refs = expr.get_column_references();
        let table_map: IndexMap<TableRef, Table<CP::Scalar>> = expr
            .get_table_references()
            .into_iter()
            .map(|table_ref| {
                let idents: IndexSet<Ident> = total_col_refs
                    .iter()
                    .filter(|col_ref| col_ref.table_ref() == table_ref)
                    .map(ColumnRef::column_id)
                    .collect();
                (table_ref.clone(), accessor.get_table(&table_ref, &idents))
            })
            .collect();

        // Prover First Round: Evaluate the query && get the right number of post result challenges
        let mut first_round_builder = FirstRoundBuilder::new(initial_range_length);
        let query_result =
            expr.first_round_evaluate(&mut first_round_builder, &alloc, &table_map, params)?;
        let owned_table_result = OwnedTable::from(&query_result);
        let provable_result = query_result.into();
        let chi_evaluation_lengths = first_round_builder.chi_evaluation_lengths();
        let rho_evaluation_lengths = first_round_builder.rho_evaluation_lengths();

        let range_length = first_round_builder.range_length();
        let num_sumcheck_variables = cmp::max(log2_up(range_length), 1);
        assert!(num_sumcheck_variables > 0);
        let post_result_challenge_count = first_round_builder.num_post_result_challenges();

        // commit to any intermediate MLEs
        let first_round_commitments =
            first_round_builder.commit_intermediate_mles(min_row_num, setup);

        // construct a transcript for the proof
        let mut transcript: Keccak256Transcript = Transcript::new();
        transcript.extend_as_le([SETUP_HASH]);
        transcript.challenge_as_le();
        transcript.extend_serialize_as_le(expr);
        transcript.challenge_as_le();
        transcript.extend_serialize_as_le(&owned_table_result);
        transcript.challenge_as_le();

        for table in expr.get_table_references() {
            let length = accessor.get_length(&table);
            transcript.extend_serialize_as_le(&[0, 0, 0, length]);
        }
        transcript.challenge_as_le();

        for commitment in CP::Commitment::compute_commitments(
            &expr
                .get_column_references()
                .into_iter()
                .map(|col| {
                    CommittableColumn::from(accessor.get_column(&col.table_ref(), &col.column_id()))
                })
                .collect_vec(),
            min_row_num,
            setup,
        ) {
            transcript.extend_serialize_as_le(&commitment);
        }
        transcript.challenge_as_le();

        transcript.extend_serialize_as_le(&min_row_num);
        transcript.challenge_as_le();

        let first_round_message = FirstRoundMessage {
            range_length,
            chi_evaluation_lengths: chi_evaluation_lengths.to_vec(),
            rho_evaluation_lengths: rho_evaluation_lengths.to_vec(),
            post_result_challenge_count,
            round_commitments: first_round_commitments,
        };
        transcript.extend_serialize_as_le(&first_round_message);

        // These are the challenges that will be consumed by the proof
        // Specifically, these are the challenges that the verifier sends to
        // the prover after the prover sends the result, but before the prover
        // send commitments to the intermediate witness columns.
        // Note: the last challenge in the vec is the first one that is consumed.
        let post_result_challenges =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(post_result_challenge_count)
                .collect();

        let mut final_round_builder =
            FinalRoundBuilder::new(num_sumcheck_variables, post_result_challenges);

        expr.final_round_evaluate(&mut final_round_builder, &alloc, &table_map, params)?;

        let num_sumcheck_variables = final_round_builder.num_sumcheck_variables();

        // commit to any intermediate MLEs
        let final_round_commitments =
            final_round_builder.commit_intermediate_mles(min_row_num, setup);

        let final_round_message = FinalRoundMessage {
            subpolynomial_constraint_count: final_round_builder.num_sumcheck_subpolynomials(),
            round_commitments: final_round_commitments,
            bit_distributions: final_round_builder.bit_distributions().to_vec(),
        };

        // add the commitments, bit distributions and chi evaluation lengths to the proof
        transcript.challenge_as_le();
        transcript.extend_serialize_as_le(&final_round_message);

        // construct the sumcheck polynomial
        let num_random_scalars =
            num_sumcheck_variables + final_round_message.subpolynomial_constraint_count;
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(num_random_scalars)
                .collect();
        let state = make_sumcheck_prover_state(
            final_round_builder.sumcheck_subpolynomials(),
            num_sumcheck_variables,
            &SumcheckRandomScalars::new(&random_scalars, range_length, num_sumcheck_variables),
        );
        transcript.challenge_as_le();

        // create the sumcheck proof -- this is the main part of proving a query
        let span = span!(Level::DEBUG, "Sumcheck with initialization").entered();
        let mut evaluation_point = vec![Zero::zero(); state.num_vars];
        let sumcheck_proof = SumcheckProof::create(&mut transcript, &mut evaluation_point, state);
        span.exit();

        // evaluate the MLEs used in sumcheck except for the result columns
        let span = span!(Level::DEBUG, "initialize evaluation_vec").entered();
        let mut evaluation_vec = vec![Zero::zero(); range_length];
        span.exit();
        compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
        let first_round_pcs_proof_evaluations =
            first_round_builder.evaluate_pcs_proof_mles(&evaluation_vec);
        let span = span!(Level::DEBUG, "initialize column_ref_pcs_proof_evaluations").entered();
        let column_ref_pcs_proof_evaluations: Vec<_> = total_col_refs
            .iter()
            .map(|col_ref| {
                accessor
                    .get_column(&col_ref.table_ref(), &col_ref.column_id())
                    .inner_product(&evaluation_vec)
            })
            .collect();
        span.exit();
        let final_round_pcs_proof_evaluations =
            final_round_builder.evaluate_pcs_proof_mles(&evaluation_vec);

        // commit to the MLE evaluations
        let pcs_proof_evaluations = QueryProofPCSProofEvaluations {
            first_round: first_round_pcs_proof_evaluations,
            column_ref: column_ref_pcs_proof_evaluations,
            final_round: final_round_pcs_proof_evaluations,
        };
        transcript.extend_serialize_as_le(&pcs_proof_evaluations);

        // fold together the pre result MLEs -- this will form the input to an inner product proof
        // of their evaluations (fold in this context means create a random linear combination)
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(
                    pcs_proof_evaluations.first_round.len()
                        + pcs_proof_evaluations.column_ref.len()
                        + pcs_proof_evaluations.final_round.len(),
                )
                .collect();

        let column_ref_mles: Vec<_> = total_col_refs
            .into_iter()
            .map(|c| {
                Box::new(accessor.get_column(&c.table_ref(), &c.column_id()))
                    as Box<dyn MultilinearExtension<_>>
            })
            .collect();

        let span = span!(Level::DEBUG, "QueryProof get folded_mle").entered();
        let mut folded_mle = vec![Zero::zero(); range_length];
        for (multiplier, evaluator) in random_scalars.iter().zip(
            first_round_builder
                .pcs_proof_mles()
                .iter()
                .chain(&column_ref_mles)
                .chain(final_round_builder.pcs_proof_mles().iter()),
        ) {
            evaluator.mul_add(&mut folded_mle, multiplier);
        }
        span.exit();

        // finally, form the inner product proof of the MLEs' evaluations
        let evaluation_proof = CP::new(
            &mut transcript,
            &folded_mle,
            &evaluation_point,
            min_row_num as u64,
            setup,
        );

        let proof = Self {
            first_round_message,
            final_round_message,
            sumcheck_proof,
            pcs_proof_evaluations,
            evaluation_proof,
        };

        log::log_memory_usage("End");

        Ok((proof, provable_result))
    }

    #[tracing::instrument(name = "QueryProof::verify", level = "debug", skip_all, err)]
    #[expect(clippy::too_many_lines)]
    /// Verify a `QueryProof`. Note: This does NOT transform the result!
    pub fn verify(
        self,
        expr: &(impl ProofPlan + Serialize),
        accessor: &impl CommitmentAccessor<CP::Commitment>,
        result: OwnedTable<CP::Scalar>,
        setup: &CP::VerifierPublicSetup<'_>,
        params: &[LiteralValue],
    ) -> QueryResult<CP::Scalar> {
        log::log_memory_usage("Start");

        let table_refs = expr.get_table_references();
        let (min_row_num, _) = get_index_range(accessor, &table_refs);
        let num_sumcheck_variables = cmp::max(log2_up(self.first_round_message.range_length), 1);
        assert!(num_sumcheck_variables > 0);

        // validate bit decompositions
        for dist in &self.final_round_message.bit_distributions {
            if !dist.is_valid() {
                Err(ProofError::VerificationError {
                    error: "invalid bit distributions",
                })?;
            } else if !dist.is_within_acceptable_range() {
                Err(ProofError::VerificationError {
                    error: "bit distribution outside of acceptable range",
                })?;
            }
        }

        let column_references = expr.get_column_references();

        // construct a transcript for the proof
        let mut transcript: Keccak256Transcript = Transcript::new();
        transcript.extend_as_le([SETUP_HASH]);
        transcript.challenge_as_le();
        transcript.extend_serialize_as_le(expr);
        transcript.challenge_as_le();
        transcript.extend_serialize_as_le(&result);
        transcript.challenge_as_le();

        for table in expr.get_table_references() {
            let length = accessor.get_length(&table);
            transcript.extend_serialize_as_le(&[0, 0, 0, length]);
        }
        transcript.challenge_as_le();

        for commitment in expr
            .get_column_references()
            .into_iter()
            .map(|col| accessor.get_commitment(&col.table_ref(), &col.column_id()))
        {
            transcript.extend_serialize_as_le(&commitment);
        }
        transcript.challenge_as_le();

        transcript.extend_serialize_as_le(&min_row_num);
        transcript.challenge_as_le();

        transcript.extend_serialize_as_le(&self.first_round_message);

        // These are the challenges that will be consumed by the proof
        // Specifically, these are the challenges that the verifier sends to
        // the prover after the prover sends the result, but before the prover
        // send commitments to the intermediate witness columns.
        let post_result_challenge_count = self.first_round_message.post_result_challenge_count;
        let mut post_result_challenges = Vec::with_capacity(post_result_challenge_count);
        for _ in 0..post_result_challenge_count {
            post_result_challenges.push(transcript.scalar_challenge_as_be());
        }

        // add the commitments and bit distributions to the proof
        transcript.challenge_as_le();
        transcript.extend_serialize_as_le(&self.final_round_message);

        // draw the random scalars for sumcheck
        let num_random_scalars =
            num_sumcheck_variables + self.final_round_message.subpolynomial_constraint_count;
        let mut random_scalars = Vec::with_capacity(num_random_scalars);
        for _ in 0..num_random_scalars {
            random_scalars.push(transcript.scalar_challenge_as_be());
        }
        let sumcheck_random_scalars = SumcheckRandomScalars::new(
            &random_scalars,
            self.first_round_message.range_length,
            num_sumcheck_variables,
        );
        transcript.challenge_as_le();

        // verify sumcheck up to the evaluation check
        let subclaim = self.sumcheck_proof.verify_without_evaluation(
            &mut transcript,
            num_sumcheck_variables,
            &Zero::zero(),
        )?;

        // commit to mle evaluations
        transcript.extend_serialize_as_le(&self.pcs_proof_evaluations);

        // draw the random scalars for the evaluation proof
        // (i.e. the folding/random linear combination of the pcs_proof_mles)
        let eval_random_count = self.pcs_proof_evaluations.first_round.len()
            + self.pcs_proof_evaluations.column_ref.len()
            + self.pcs_proof_evaluations.final_round.len();
        let mut evaluation_random_scalars = Vec::with_capacity(eval_random_count);
        for _ in 0..eval_random_count {
            evaluation_random_scalars.push(transcript.scalar_challenge_as_be());
        }

        // Always prepend input lengths to the chi evaluation lengths
        let table_length_map = table_refs
            .into_iter()
            .map(|table_ref| {
                let len = accessor.get_length(&table_ref);
                (table_ref, len)
            })
            .collect::<IndexMap<TableRef, usize>>();

        let chi_evaluation_lengths = table_length_map
            .values()
            .chain(self.first_round_message.chi_evaluation_lengths.iter())
            .copied();

        // pass over the provable AST to fill in the verification builder
        let sumcheck_evaluations = SumcheckMleEvaluations::new(
            self.first_round_message.range_length,
            chi_evaluation_lengths,
            self.first_round_message.rho_evaluation_lengths.iter().copied(),
            &subclaim.evaluation_point,
            &sumcheck_random_scalars,
            &self.pcs_proof_evaluations.first_round,
            &self.pcs_proof_evaluations.final_round,
        );
        let chi_eval_map: IndexMap<TableRef, (CP::Scalar, usize)> = table_length_map
            .into_iter()
            .map(|(table_ref, length)| {
                (
                    table_ref,
                    (sumcheck_evaluations.chi_evaluations[&length], length),
                )
            })
            .collect();
        let mut builder = VerificationBuilderImpl::new(
            sumcheck_evaluations,
            &self.final_round_message.bit_distributions,
            sumcheck_random_scalars.subpolynomial_multipliers,
            &post_result_challenges,
            &self.first_round_message.chi_evaluation_lengths,
            &self.first_round_message.rho_evaluation_lengths,
            subclaim.max_multiplicands,
        );

        let total_commitments = self.first_round_message.round_commitments.len()
            + column_references.len()
            + self.final_round_message.round_commitments.len();
        let mut pcs_proof_commitments = Vec::with_capacity(total_commitments);
        pcs_proof_commitments.extend(self.first_round_message.round_commitments.iter().cloned());
        pcs_proof_commitments.extend(
            column_references
                .iter()
                .map(|col| accessor.get_commitment(&col.table_ref(), &col.column_id())),
        );
        pcs_proof_commitments.extend(self.final_round_message.round_commitments.iter().cloned());

        let evaluation_accessor: IndexMap<_, _> = column_references
            .into_iter()
            .zip(self.pcs_proof_evaluations.column_ref.iter().copied())
            .chunk_by(|(r, _)| r.table_ref())
            .into_iter()
            .map(|(tr, g)| {
                let im: IndexMap<_, _> = g.map(|(cr, eval)| (cr.column_id(), eval)).collect();
                (tr, im)
            })
            .collect();

        let verifier_evaluations =
            expr.verifier_evaluate(&mut builder, &evaluation_accessor, &chi_eval_map, params)?;
        // compute the evaluation of the result MLEs
        let result_evaluations = result.mle_evaluations(&subclaim.evaluation_point);
        // check the evaluation of the result MLEs
        if verifier_evaluations.column_evals() != result_evaluations {
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

        let mut pcs_proof_evaluations = Vec::with_capacity(eval_random_count);
        pcs_proof_evaluations.extend_from_slice(&self.pcs_proof_evaluations.first_round);
        pcs_proof_evaluations.extend_from_slice(&self.pcs_proof_evaluations.column_ref);
        pcs_proof_evaluations.extend_from_slice(&self.pcs_proof_evaluations.final_round);

        // finally, check the MLE evaluations with the inner product proof
        self.evaluation_proof
            .verify_batched_proof(
                &mut transcript,
                &pcs_proof_commitments,
                &evaluation_random_scalars,
                &pcs_proof_evaluations,
                &subclaim.evaluation_point,
                min_row_num as u64,
                self.first_round_message.range_length,
                setup,
            )
            .map_err(|_e| ProofError::VerificationError {
                error: "Inner product proof of MLE evaluations failed",
            })?;

        let verification_hash = transcript.challenge_as_le();

        log::log_memory_usage("End");

        Ok(QueryData {
            table: result,
            verification_hash,
        })
    }
}
