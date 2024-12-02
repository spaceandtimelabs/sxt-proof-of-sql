use super::{
    CountBuilder, FinalRoundBuilder, ProofCounts, ProofPlan, ProvableQueryResult, QueryResult,
    SumcheckMleEvaluations, SumcheckRandomScalars, VerificationBuilder,
};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::CommitmentEvaluationProof,
        database::{
            ColumnRef, CommitmentAccessor, DataAccessor, MetadataAccessor, Table, TableRef,
        },
        map::{IndexMap, IndexSet},
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
pub struct QueryProof<CP: CommitmentEvaluationProof> {
    /// Bit distributions
    pub bit_distributions: Vec<BitDistribution>,
    /// One evaluation lengths
    pub one_evaluation_lengths: Vec<usize>,
    /// Commitments
    pub commitments: Vec<CP::Commitment>,
    /// Sumcheck Proof
    pub sumcheck_proof: SumcheckProof<CP::Scalar>,
    /// MLEs used in sumcheck except for the result columns
    pub pcs_proof_evaluations: Vec<CP::Scalar>,
    /// Inner product proof of the MLEs' evaluations
    pub evaluation_proof: CP,
    /// Length of the range of generators we use
    pub range_length: usize,
}

impl<CP: CommitmentEvaluationProof> QueryProof<CP> {
    /// Create a new `QueryProof`.
    #[tracing::instrument(name = "QueryProof::new", level = "debug", skip_all)]
    pub fn new(
        expr: &(impl ProofPlan + Serialize),
        accessor: &impl DataAccessor<CP::Scalar>,
        setup: &CP::ProverPublicSetup<'_>,
    ) -> (Self, ProvableQueryResult) {
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
                    .copied()
                    .collect();
                (table_ref, accessor.get_table(table_ref, &col_refs))
            })
            .collect();

        // Evaluate query result
        let (query_result, one_evaluation_lengths) = expr.result_evaluate(&alloc, &table_map);
        let provable_result = query_result.into();

        // Prover First Round
        let mut first_round_builder = FirstRoundBuilder::new();
        expr.first_round_evaluate(&mut first_round_builder);
        let range_length = one_evaluation_lengths
            .iter()
            .copied()
            .chain(core::iter::once(initial_range_length))
            .max()
            .expect("Will always have at least one element"); // safe to unwrap because we have at least one element

        let num_sumcheck_variables = cmp::max(log2_up(range_length), 1);
        assert!(num_sumcheck_variables > 0);

        // construct a transcript for the proof
        let mut transcript: Keccak256Transcript = make_transcript(
            expr,
            &provable_result,
            range_length,
            min_row_num,
            &one_evaluation_lengths,
        );

        // These are the challenges that will be consumed by the proof
        // Specifically, these are the challenges that the verifier sends to
        // the prover after the prover sends the result, but before the prover
        // send commitments to the intermediate witness columns.
        // Note: the last challenge in the vec is the first one that is consumed.
        let post_result_challenges =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(first_round_builder.num_post_result_challenges())
                .collect();

        let mut builder = FinalRoundBuilder::new(num_sumcheck_variables, post_result_challenges);

        for col_ref in total_col_refs {
            builder.produce_anchored_mle(accessor.get_column(col_ref));
        }

        expr.final_round_evaluate(&mut builder, &alloc, &table_map);

        let num_sumcheck_variables = builder.num_sumcheck_variables();

        // commit to any intermediate MLEs
        let commitments = builder.commit_intermediate_mles(min_row_num, setup);

        // add the commitments, bit distributions and one evaluation lengths to the proof
        extend_transcript(&mut transcript, &commitments, builder.bit_distributions());

        // construct the sumcheck polynomial
        let num_random_scalars = num_sumcheck_variables + builder.num_sumcheck_subpolynomials();
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(num_random_scalars)
                .collect();
        let poly = builder.make_sumcheck_polynomial(&SumcheckRandomScalars::new(
            &random_scalars,
            range_length,
            num_sumcheck_variables,
        ));

        // create the sumcheck proof -- this is the main part of proving a query
        let mut evaluation_point = vec![Zero::zero(); poly.num_variables];
        let sumcheck_proof = SumcheckProof::create(&mut transcript, &mut evaluation_point, &poly);

        // evaluate the MLEs used in sumcheck except for the result columns
        let mut evaluation_vec = vec![Zero::zero(); range_length];
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

        assert_eq!(random_scalars.len(), builder.pcs_proof_mles().len());
        let mut folded_mle = vec![Zero::zero(); range_length];
        for (multiplier, evaluator) in random_scalars.iter().zip(builder.pcs_proof_mles().iter()) {
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
            bit_distributions: builder.bit_distributions().to_vec(),
            one_evaluation_lengths,
            commitments,
            sumcheck_proof,
            pcs_proof_evaluations,
            evaluation_proof,
            range_length,
        };
        (proof, provable_result)
    }

    #[tracing::instrument(name = "QueryProof::verify", level = "debug", skip_all, err)]
    /// Verify a `QueryProof`. Note: This does NOT transform the result!
    pub fn verify(
        &self,
        expr: &(impl ProofPlan + Serialize),
        accessor: &impl CommitmentAccessor<CP::Commitment>,
        result: &ProvableQueryResult,
        setup: &CP::VerifierPublicSetup<'_>,
    ) -> QueryResult<CP::Scalar> {
        let owned_table_result = result.to_owned_table(&expr.get_column_result_fields())?;
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
            }
        }

        let column_references = expr.get_column_references();
        // count terms

        let mut builder = CountBuilder::new(&self.bit_distributions);
        builder.count_anchored_mles(column_references.len());
        expr.count(&mut builder)?;
        let counts = builder.counts()?;

        // verify sizes
        if !self.validate_sizes(&counts) {
            Err(ProofError::VerificationError {
                error: "invalid proof size",
            })?;
        }

        // construct a transcript for the proof
        let mut transcript: Keccak256Transcript = make_transcript(
            expr,
            result,
            self.range_length,
            min_row_num,
            &self.one_evaluation_lengths,
        );

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
            SumcheckRandomScalars::new(&random_scalars, self.range_length, num_sumcheck_variables);

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
            &self.pcs_proof_evaluations,
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
            &evaluation_random_scalars,
            post_result_challenges,
            self.one_evaluation_lengths.clone(),
        );

        let pcs_proof_commitments: Vec<_> = column_references
            .iter()
            .map(|col| accessor.get_commitment(*col))
            .chain(self.commitments.iter().cloned())
            .collect();
        let evaluation_accessor: IndexMap<_, _> = column_references
            .into_iter()
            .map(|col| (col, builder.consume_anchored_mle()))
            .collect();

        let verifier_evaluations = expr.verifier_evaluate(
            &mut builder,
            &evaluation_accessor,
            Some(&owned_table_result),
            &one_eval_map,
        )?;
        // compute the evaluation of the result MLEs
        let result_evaluations = owned_table_result.mle_evaluations(&subclaim.evaluation_point);
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

        // finally, check the MLE evaluations with the inner product proof
        let product = builder.folded_pcs_proof_evaluation();
        self.evaluation_proof
            .verify_batched_proof(
                &mut transcript,
                &pcs_proof_commitments,
                builder.inner_product_multipliers(),
                &product,
                &subclaim.evaluation_point,
                min_row_num as u64,
                self.range_length,
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
/// * `range_length` - The length of the range of the generator used in the proof, as a `usize`.
///
/// * `min_row_num` - The smallest offset of the generator used in the proof, as a `usize`.
///
/// * `one_evaluation_lengths` - A slice of `usize` values that represent unexpected intermediate table lengths
///
/// # Returns
/// This function returns a `merlin::Transcript`. The transcript is a record
/// of all the operations and data involved in creating a proof.
/// ```
fn make_transcript<T: Transcript>(
    expr: &(impl ProofPlan + Serialize),
    result: &ProvableQueryResult,
    range_length: usize,
    min_row_num: usize,
    one_evaluation_lengths: &[usize],
) -> T {
    let mut transcript = T::new();
    transcript.extend_serialize_as_le(result);
    transcript.extend_serialize_as_le(expr);
    transcript.extend_serialize_as_le(&range_length);
    transcript.extend_serialize_as_le(&min_row_num);
    transcript.extend_serialize_as_le(one_evaluation_lengths);
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
