use super::{
    make_sumcheck_state::make_sumcheck_prover_state, FinalRoundBuilder, FirstRoundBuilder,
    ProofPlan, QueryData, QueryResult, SumcheckMleEvaluations, SumcheckRandomScalars,
    VerificationBuilder,
};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::{Commitment, CommitmentEvaluationProof},
        database::{
            ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor, OwnedColumn, OwnedTable,
            Table, TableRef,
        },
        map::{IndexMap, IndexSet},
        math::log2_up,
        polynomial::compute_evaluation_vector,
        proof::{Keccak256Transcript, ProofError, Transcript},
        scalar::Scalar,
    },
    proof_primitive::sumcheck::SumcheckProof,
    utils::log,
};
use alloc::{string::String, vec, vec::Vec};
use bumpalo::Bump;
use core::cmp;
use num_traits::Zero;
use proof_of_sql_parser::sqlparser::TimezoneInfoExt;
use serde::{Deserialize, Serialize};

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
            let length = accessor.get_length(*table_ref);
            let offset = accessor.get_offset(*table_ref);
            (offset, offset + length)
        })
        .reduce(|(min_start, max_end), (start, end)| (min_start.min(start), max_end.max(end)))
        // Only applies to `EmptyExec` where there are no tables
        .unwrap_or((0, 1))
}

/// The proof for a query.
///
/// Note: Because the class is deserialized from untrusted data, it
/// cannot maintain any invariant on its data members; hence, they are
/// all public so as to allow for easy manipulation for testing.
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct QueryProof<CP: CommitmentEvaluationProof> {
    /// Bit distributions
    pub bit_distributions: Vec<BitDistribution>,
    /// One evaluation lengths
    pub one_evaluation_lengths: Vec<usize>,
    /// First Round Commitments
    pub first_round_commitments: Vec<CP::Commitment>,
    /// Final Round Commitments
    pub final_round_commitments: Vec<CP::Commitment>,
    /// Sumcheck Proof
    pub sumcheck_proof: SumcheckProof<CP::Scalar>,
    /// MLEs used in first round sumcheck except for the result columns
    pub first_round_pcs_proof_evaluations: Vec<CP::Scalar>,
    /// MLEs used in final round sumcheck except for the result columns
    pub final_round_pcs_proof_evaluations: Vec<CP::Scalar>,
    /// Inner product proof of the MLEs' evaluations
    pub evaluation_proof: CP,
    /// Length of the range of generators we use
    pub range_length: usize,
    pub subpolynomial_constraint_count: usize,
    pub post_result_challenge_count: usize,
}

impl<CP: CommitmentEvaluationProof> QueryProof<CP> {
    /// Create a new `QueryProof`.
    #[tracing::instrument(name = "QueryProof::new", level = "debug", skip_all)]
    pub fn new(
        expr: &(impl ProofPlan + Serialize),
        accessor: &impl DataAccessor<CP::Scalar>,
        setup: &CP::ProverPublicSetup<'_>,
    ) -> (Self, OwnedTable<CP::Scalar>) {
        log::log_memory_usage("Start");

        let (min_row_num, max_row_num) = get_index_range(accessor, &expr.get_table_references());
        let initial_range_length = max_row_num - min_row_num;
        let alloc = Bump::new();

        let total_col_refs = expr.get_column_references();
        let table_map: IndexMap<TableRef, Table<CP::Scalar>> = expr
            .get_table_references()
            .into_iter()
            .map(|table_ref| {
                let col_refs: IndexSet<ColumnRef> = total_col_refs
                    .iter()
                    .filter(|col_ref| col_ref.table_ref() == table_ref)
                    .cloned()
                    .collect();
                (table_ref, accessor.get_table(table_ref, &col_refs))
            })
            .collect();

        // Prover First Round: Evaluate the query && get the right number of post result challenges
        let mut first_round_builder = FirstRoundBuilder::new(initial_range_length);
        let query_result = expr.first_round_evaluate(&mut first_round_builder, &alloc, &table_map);
        let owned_table_result = OwnedTable::from(&query_result);
        let provable_result = query_result.into();
        let one_evaluation_lengths = first_round_builder.one_evaluation_lengths();

        let range_length = first_round_builder.range_length();

        let num_sumcheck_variables = cmp::max(log2_up(range_length), 1);
        assert!(num_sumcheck_variables > 0);
        let post_result_challenge_count = first_round_builder.num_post_result_challenges();

        // commit to any intermediate MLEs
        let first_round_commitments =
            first_round_builder.commit_intermediate_mles(min_row_num, setup);

        // construct a transcript for the proof
        let mut transcript: Keccak256Transcript = make_transcript(
            expr,
            &owned_table_result,
            range_length,
            min_row_num,
            one_evaluation_lengths,
            post_result_challenge_count,
            &first_round_commitments,
        );

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

        for col_ref in total_col_refs {
            final_round_builder.produce_anchored_mle(accessor.get_column(col_ref));
        }

        expr.final_round_evaluate(&mut final_round_builder, &alloc, &table_map);

        let num_sumcheck_variables = final_round_builder.num_sumcheck_variables();

        // commit to any intermediate MLEs
        let final_round_commitments =
            final_round_builder.commit_intermediate_mles(min_row_num, setup);

        // add the commitments, bit distributions and one evaluation lengths to the proof
        extend_transcript_with_commitments(
            &mut transcript,
            &final_round_commitments,
            final_round_builder.bit_distributions(),
        );

        // construct the sumcheck polynomial
        let subpolynomial_constraint_count = final_round_builder.num_sumcheck_subpolynomials();
        let num_random_scalars = num_sumcheck_variables + subpolynomial_constraint_count;
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(num_random_scalars)
                .collect();
        let state = make_sumcheck_prover_state(
            final_round_builder.sumcheck_subpolynomials(),
            num_sumcheck_variables,
            &SumcheckRandomScalars::new(&random_scalars, range_length, num_sumcheck_variables),
        );

        // create the sumcheck proof -- this is the main part of proving a query
        let mut evaluation_point = vec![Zero::zero(); state.num_vars];
        let sumcheck_proof = SumcheckProof::create(&mut transcript, &mut evaluation_point, state);

        // evaluate the MLEs used in sumcheck except for the result columns
        let mut evaluation_vec = vec![Zero::zero(); range_length];
        compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);
        let first_round_pcs_proof_evaluations =
            first_round_builder.evaluate_pcs_proof_mles(&evaluation_vec);
        let final_round_pcs_proof_evaluations =
            final_round_builder.evaluate_pcs_proof_mles(&evaluation_vec);

        // commit to the MLE evaluations
        transcript.extend_canonical_serialize_as_le(&first_round_pcs_proof_evaluations);
        transcript.extend_canonical_serialize_as_le(&final_round_pcs_proof_evaluations);

        // fold together the pre result MLEs -- this will form the input to an inner product proof
        // of their evaluations (fold in this context means create a random linear combination)
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(
                    first_round_pcs_proof_evaluations.len()
                        + final_round_pcs_proof_evaluations.len(),
                )
                .collect();

        assert_eq!(
            random_scalars.len(),
            first_round_builder.pcs_proof_mles().len() + final_round_builder.pcs_proof_mles().len()
        );
        let mut folded_mle = vec![Zero::zero(); range_length];
        for (multiplier, evaluator) in random_scalars.iter().zip(
            first_round_builder
                .pcs_proof_mles()
                .iter()
                .chain(final_round_builder.pcs_proof_mles().iter()),
        ) {
            evaluator.mul_add(&mut folded_mle, multiplier);
        }

        // finally, form the inner product proof of the MLEs' evaluations
        let evaluation_proof = CP::new(
            &mut transcript,
            &folded_mle,
            &evaluation_point,
            min_row_num as u64,
            setup,
        );

        let proof = Self {
            bit_distributions: final_round_builder.bit_distributions().to_vec(),
            one_evaluation_lengths: one_evaluation_lengths.to_vec(),
            first_round_commitments,
            final_round_commitments,
            sumcheck_proof,
            first_round_pcs_proof_evaluations,
            final_round_pcs_proof_evaluations,
            evaluation_proof,
            range_length,
            subpolynomial_constraint_count,
            post_result_challenge_count,
        };

        log::log_memory_usage("End");

        (proof, provable_result)
    }

    #[tracing::instrument(name = "QueryProof::verify", level = "debug", skip_all, err)]
    /// Verify a `QueryProof`. Note: This does NOT transform the result!
    pub fn verify(
        self,
        expr: &(impl ProofPlan + Serialize),
        accessor: &impl CommitmentAccessor<CP::Commitment>,
        result: OwnedTable<CP::Scalar>,
        setup: &CP::VerifierPublicSetup<'_>,
    ) -> QueryResult<CP::Scalar> {
        log::log_memory_usage("Start");

        let table_refs = expr.get_table_references();
        let (min_row_num, _) = get_index_range(accessor, &table_refs);
        let num_sumcheck_variables = cmp::max(log2_up(self.range_length), 1);
        assert!(num_sumcheck_variables > 0);

        // validate bit decompositions
        for dist in &self.bit_distributions {
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
        let mut transcript: Keccak256Transcript = make_transcript(
            expr,
            &result,
            self.range_length,
            min_row_num,
            &self.one_evaluation_lengths,
            self.post_result_challenge_count,
            &self.first_round_commitments,
        );

        // These are the challenges that will be consumed by the proof
        // Specifically, these are the challenges that the verifier sends to
        // the prover after the prover sends the result, but before the prover
        // send commitments to the intermediate witness columns.
        // Note: the last challenge in the vec is the first one that is consumed.
        let post_result_challenges =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(self.post_result_challenge_count)
                .collect();

        // add the commitments and bit distributions to the proof
        extend_transcript_with_commitments(
            &mut transcript,
            &self.final_round_commitments,
            &self.bit_distributions,
        );

        // draw the random scalars for sumcheck
        let num_random_scalars = num_sumcheck_variables + self.subpolynomial_constraint_count;
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(num_random_scalars)
                .collect();
        let sumcheck_random_scalars =
            SumcheckRandomScalars::new(&random_scalars, self.range_length, num_sumcheck_variables);

        // verify sumcheck up to the evaluation check
        let subclaim = self.sumcheck_proof.verify_without_evaluation(
            &mut transcript,
            // This needs to be at least 2 since `CompositePolynomialBuilder::make_composite_polynomial`
            // always adds a degree 2 term.
            num_sumcheck_variables,
            &Zero::zero(),
        )?;

        // commit to mle evaluations
        transcript.extend_canonical_serialize_as_le(&self.first_round_pcs_proof_evaluations);
        transcript.extend_canonical_serialize_as_le(&self.final_round_pcs_proof_evaluations);

        // draw the random scalars for the evaluation proof
        // (i.e. the folding/random linear combination of the pcs_proof_mles)
        let evaluation_random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(
                    self.first_round_pcs_proof_evaluations.len()
                        + self.final_round_pcs_proof_evaluations.len(),
                )
                .collect();

        // Always prepend input lengths to the one evaluation lengths
        let table_length_map = table_refs
            .iter()
            .map(|table_ref| (table_ref, accessor.get_length(*table_ref)))
            .collect::<IndexMap<_, _>>();

        let one_evaluation_lengths = table_length_map
            .values()
            .chain(self.one_evaluation_lengths.iter())
            .copied();

        // pass over the provable AST to fill in the verification builder
        let sumcheck_evaluations = SumcheckMleEvaluations::new(
            self.range_length,
            one_evaluation_lengths,
            &subclaim.evaluation_point,
            &sumcheck_random_scalars,
            &self.first_round_pcs_proof_evaluations,
            &self.final_round_pcs_proof_evaluations,
        );
        let one_eval_map: IndexMap<TableRef, CP::Scalar> = table_length_map
            .iter()
            .map(|(table_ref, length)| (**table_ref, sumcheck_evaluations.one_evaluations[length]))
            .collect();
        let mut builder = VerificationBuilder::new(
            min_row_num,
            sumcheck_evaluations,
            &self.bit_distributions,
            sumcheck_random_scalars.subpolynomial_multipliers,
            post_result_challenges,
            self.one_evaluation_lengths.clone(),
            subclaim.max_multiplicands,
        );

        let pcs_proof_commitments: Vec<_> = self
            .first_round_commitments
            .iter()
            .cloned()
            .chain(
                column_references
                    .iter()
                    .map(|col| accessor.get_commitment(col.clone())),
            )
            .chain(self.final_round_commitments.iter().cloned())
            .collect();
        let evaluation_accessor: IndexMap<_, _> = column_references
            .into_iter()
            .map(|col| Ok((col, builder.try_consume_final_round_mle_evaluation()?)))
            .collect::<Result<_, ProofError>>()?;

        let verifier_evaluations = expr.verifier_evaluate(
            &mut builder,
            &evaluation_accessor,
            Some(&result),
            &one_eval_map,
        )?;
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

        let pcs_proof_evaluations: Vec<_> = self
            .first_round_pcs_proof_evaluations
            .iter()
            .chain(self.final_round_pcs_proof_evaluations.iter())
            .copied()
            .collect();

        // finally, check the MLE evaluations with the inner product proof
        self.evaluation_proof
            .verify_batched_proof(
                &mut transcript,
                &pcs_proof_commitments,
                &evaluation_random_scalars,
                &pcs_proof_evaluations,
                &subclaim.evaluation_point,
                min_row_num as u64,
                self.range_length,
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

/// Constructs a transcript for the proof process.
///
/// This function initializes a transcript and extends it with various elements
/// such as the result table columns, the proof plan expression, the range length,
/// the minimum row number, and the one evaluation lengths.
///
/// # Arguments
///
/// * `expr` - The proof plan expression.
/// * `result` - The result table containing the query result.
/// * `range_length` - The length of the range of generators used.
/// * `min_row_num` - The minimum row number in the index range of the tables referenced by the query.
/// * `one_evaluation_lengths` - The lengths of the one evaluations.
/// * `post_result_challenge_count` - The number of post-result challenges.
/// * `first_round_commitments` - A slice of commitments produced before post-result challenges that are part of the proof.
///
/// # Returns
///
/// A transcript initialized with the provided data.
fn make_transcript<C: Commitment, T: Transcript>(
    expr: &(impl ProofPlan + Serialize),
    result: &OwnedTable<C::Scalar>,
    range_length: usize,
    min_row_num: usize,
    one_evaluation_lengths: &[usize],
    post_result_challenge_count: usize,
    first_round_commitments: &[C],
) -> T {
    let mut transcript = T::new();
    extend_transcript_with_owned_table(&mut transcript, result);
    transcript.extend_serialize_as_le(expr);
    transcript.extend_serialize_as_le(&range_length);
    transcript.extend_serialize_as_le(&min_row_num);
    transcript.extend_serialize_as_le(one_evaluation_lengths);
    transcript.extend_serialize_as_le(&post_result_challenge_count);
    for commitment in first_round_commitments {
        commitment.append_to_transcript(&mut transcript);
    }
    transcript
}

/// Extends the transcript with the columns of an owned table.
///
/// This function adds the columns of the owned table to the transcript.
///
/// # Arguments
///
/// * `transcript` - The transcript to extend.
/// * `result` - The owned table containing the query result.
fn extend_transcript_with_owned_table<S: Scalar, T: Transcript>(
    transcript: &mut T,
    result: &OwnedTable<S>,
) {
    for (name, column) in result.inner_table() {
        transcript.extend_as_le_from_refs([name.value.as_str()]);
        match column {
            OwnedColumn::Boolean(col) => transcript.extend_as_be(col.iter().map(|&b| u8::from(b))),
            OwnedColumn::TinyInt(col) => transcript.extend_as_be_from_refs(col),
            OwnedColumn::SmallInt(col) => transcript.extend_as_be_from_refs(col),
            OwnedColumn::Int(col) => transcript.extend_as_be_from_refs(col),
            OwnedColumn::BigInt(col) => transcript.extend_as_be_from_refs(col),
            OwnedColumn::VarChar(col) => {
                transcript.extend_as_le_from_refs(col.iter().map(String::as_str));
            }
            OwnedColumn::Int128(col) => transcript.extend_as_be_from_refs(col),
            OwnedColumn::Decimal75(precision, scale, col) => {
                transcript.extend_as_be([precision.value()]);
                transcript.extend_as_be([*scale]);
                transcript.extend_as_be(col.iter().map(|&s| Into::<[u64; 4]>::into(s)));
            }
            OwnedColumn::Scalar(col) => {
                transcript.extend_as_be(col.iter().map(|&s| Into::<[u64; 4]>::into(s)));
            }
            OwnedColumn::TimestampTZ(po_sqltime_unit, timezone_info, col) => {
                transcript.extend_as_be([u64::from(*po_sqltime_unit)]);
                transcript.extend_as_be([timezone_info.offset(Some("+00:00"))]);
                transcript.extend_as_be_from_refs(col);
            }
        }
    }
}

/// Extends the transcript with commitments and bit distributions.
///
/// This function adds the commitments and bit distributions to the transcript.
///
/// # Arguments
///
/// * `transcript` - The transcript to extend.
/// * `commitments` - The commitments to add to the transcript.
/// * `bit_distributions` - The bit distributions to add to the transcript.
fn extend_transcript_with_commitments<C: Commitment>(
    transcript: &mut impl Transcript,
    final_round_commitments: &[C],
    bit_distributions: &[BitDistribution],
) {
    for commitment in final_round_commitments {
        commitment.append_to_transcript(transcript);
    }
    transcript.extend_serialize_as_le(bit_distributions);
}
