use super::{
    make_sumcheck_state::make_sumcheck_prover_state, CountBuilder, FinalRoundBuilder, ProofCounts,
    ProofPlan, ProvableQueryResult, QueryResult, SumcheckMleEvaluations, SumcheckRandomScalars,
    VerificationBuilder,
};
use crate::{
    base::{
        bit::BitDistribution,
        commitment::CommitmentEvaluationProof,
        database::{Column, CommitmentAccessor, DataAccessor, MetadataAccessor, TableRef},
        map::IndexMap,
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
use sysinfo::{System, SystemExt};


/// Return the row number range of tables referenced in the Query
///
/// Basically we are looking for the smallest offset and the largest offset + length
/// so that we have an index range of the table rows that the query is referencing.
fn get_index_range(
    accessor: &dyn MetadataAccessor,
    table_refs: impl IntoIterator<Item = TableRef>,
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
        expr: &(impl ProofPlan + Serialize),
        accessor: &impl DataAccessor<CP::Scalar>,
        setup: &CP::ProverPublicSetup<'_>,
    ) -> (Self, ProvableQueryResult) {
        let (min_row_num, max_row_num) = get_index_range(accessor, expr.get_table_references());
        let range_length = max_row_num - min_row_num;
        let num_sumcheck_variables = cmp::max(log2_up(range_length), 1);
        assert!(num_sumcheck_variables > 0);

        let alloc = Bump::new();

        let mut system = System::new_all();
        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin QueryProof::new");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);           

        // Evaluate query result
        let result_cols = expr.result_evaluate(range_length, &alloc, accessor);
        let output_length = result_cols.first().map_or(0, Column::len);
        let provable_result = ProvableQueryResult::new(output_length as u64, &result_cols);

        // Prover First Round
        let mut first_round_builder = FirstRoundBuilder::new();
        expr.first_round_evaluate(&mut first_round_builder);

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin make_transcript");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);   

        // construct a transcript for the proof
        let mut transcript: Keccak256Transcript =
            make_transcript(expr, &provable_result, range_length, min_row_num);

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin post_result_challenges");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        // These are the challenges that will be consumed by the proof
        // Specifically, these are the challenges that the verifier sends to
        // the prover after the prover sends the result, but before the prover
        // send commitments to the intermediate witness columns.
        // Note: the last challenge in the vec is the first one that is consumed.
        let post_result_challenges =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(first_round_builder.num_post_result_challenges())
                .collect();

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("FinalRoundBuilder::new");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        let mut builder =
            FinalRoundBuilder::new(range_length, num_sumcheck_variables, post_result_challenges);

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin produce_anchored_mle");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        expr.get_column_references().into_iter().for_each(|col| {
            builder.produce_anchored_mle(accessor.get_column(col));
        });

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin final_round_evaluate");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        expr.final_round_evaluate(&mut builder, &alloc, accessor);

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin num_sumcheck_variables");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        let num_sumcheck_variables = builder.num_sumcheck_variables();

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin commit_intermediate_mles");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);   

        // commit to any intermediate MLEs
        let commitments = builder.commit_intermediate_mles(min_row_num, setup);

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin extend_transcript");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        // add the commitments and bit distributions to the proof
        extend_transcript(&mut transcript, &commitments, builder.bit_distributions());

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin num_sumcheck_subpolynomials");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        // construct the sumcheck polynomial
        let num_random_scalars = num_sumcheck_variables + builder.num_sumcheck_subpolynomials();

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin random_scalars");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(num_random_scalars)
                .collect();

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin sumcheck_state");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        let sumcheck_state = make_sumcheck_prover_state(
            builder.sumcheck_subpolynomials(),
            num_sumcheck_variables,
            &SumcheckRandomScalars::new(&random_scalars, range_length, num_sumcheck_variables),
        );

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin SumcheckProof::create");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);  

        // create the sumcheck proof -- this is the main part of proving a query
        let mut evaluation_point = vec![Zero::zero(); num_sumcheck_variables];
        let sumcheck_proof =
            SumcheckProof::create(&mut transcript, &mut evaluation_point, sumcheck_state);

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin compute_evaluation_vector");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        // evaluate the MLEs used in sumcheck except for the result columns
        let mut evaluation_vec = vec![Zero::zero(); range_length];
        compute_evaluation_vector(&mut evaluation_vec, &evaluation_point);

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin evaluate_pcs_proof_mles");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        let pcs_proof_evaluations = builder.evaluate_pcs_proof_mles(&evaluation_vec);

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin extend_canonical_serialize_as_le");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        // commit to the MLE evaluations
        transcript.extend_canonical_serialize_as_le(&pcs_proof_evaluations);

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin random_scalars");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        // fold together the pre result MLEs -- this will form the input to an inner product proof
        // of their evaluations (fold in this context means create a random linear combination)
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(pcs_proof_evaluations.len())
                .collect();

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin folded_mle");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used); 

        let folded_mle = builder.fold_pcs_proof_mles(&random_scalars);

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin CP::new");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);  

        // finally, form the inner product proof of the MLEs' evaluations
        let evaluation_proof = CP::new(
            &mut transcript,
            &folded_mle,
            &evaluation_point,
            min_row_num as u64,
            setup,
        );

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin proof");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);  

        let proof = Self {
            bit_distributions: builder.bit_distributions().to_vec(),
            commitments,
            sumcheck_proof,
            pcs_proof_evaluations,
            evaluation_proof,
        };

        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("End proof");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);

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
        ////////////////////
        // PRINT CPU INFO //
        ////////////////////
        let mut system = System::new_all();
        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("Begin QueryProof::verify");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);
        ////////////////////
        ////////////////////
        ////////////////////

        let (min_row_num, max_row_num) = get_index_range(accessor, expr.get_table_references());
        let range_length = max_row_num - min_row_num;
        let num_sumcheck_variables = cmp::max(log2_up(range_length), 1);
        assert!(num_sumcheck_variables > 0);

        let output_length = result.table_length();

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

        ////////////////////
        // PRINT CPU INFO //
        ////////////////////
        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("QueryProof::verify - before creating CountBuilder::new");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);
        ////////////////////
        ////////////////////
        ////////////////////

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

        ////////////////////
        // PRINT CPU INFO //
        ////////////////////
        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("QueryProof::verify - before make_transcript");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);
        ////////////////////
        ////////////////////
        ////////////////////

        // construct a transcript for the proof
        let mut transcript: Keccak256Transcript =
            make_transcript(expr, result, range_length, min_row_num);

        ////////////////////
        // PRINT CPU INFO //
        ////////////////////
        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("QueryProof::verify - before extend_transcript");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);
        ////////////////////
        ////////////////////
        ////////////////////

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

        ////////////////////
        // PRINT CPU INFO //
        ////////////////////
        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("QueryProof::verify - before SumcheckRandomScalars");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);
        ////////////////////
        ////////////////////
        ////////////////////

        // draw the random scalars for sumcheck
        let num_random_scalars = num_sumcheck_variables + counts.sumcheck_subpolynomials;
        let random_scalars: Vec<_> =
            core::iter::repeat_with(|| transcript.scalar_challenge_as_be())
                .take(num_random_scalars)
                .collect();
        let sumcheck_random_scalars =
            SumcheckRandomScalars::new(&random_scalars, range_length, num_sumcheck_variables);

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
            range_length,
            output_length,
            &subclaim.evaluation_point,
            &sumcheck_random_scalars,
            &self.pcs_proof_evaluations,
        );
        let mut builder = VerificationBuilder::new(
            min_row_num,
            sumcheck_evaluations,
            &self.bit_distributions,
            sumcheck_random_scalars.subpolynomial_multipliers,
            &evaluation_random_scalars,
            post_result_challenges,
        );
        let owned_table_result = result.to_owned_table(&column_result_fields[..])?;

        let pcs_proof_commitments: Vec<_> = column_references
            .iter()
            .map(|col| accessor.get_commitment(*col))
            .chain(self.commitments.iter().cloned())
            .collect();
        let evaluation_accessor: IndexMap<_, _> = column_references
            .into_iter()
            .map(|col| (col, builder.consume_anchored_mle()))
            .collect();

        ////////////////////
        // PRINT CPU INFO //
        ////////////////////
        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("QueryProof::verify - before verifier_evaluations");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);
        ////////////////////
        ////////////////////
        ////////////////////

        let verifier_evaluations = expr.verifier_evaluate(
            &mut builder,
            &evaluation_accessor,
            Some(&owned_table_result),
        )?;
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

        ////////////////////
        // PRINT CPU INFO //
        ////////////////////
        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("QueryProof::verify - before sumcheck_evaluation");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);
        ////////////////////
        ////////////////////
        ////////////////////

        // perform the evaluation check of the sumcheck polynomial
        if builder.sumcheck_evaluation() != subclaim.expected_evaluation {
            Err(ProofError::VerificationError {
                error: "sumcheck evaluation check failed",
            })?;
        }

        ////////////////////
        // PRINT CPU INFO //
        ////////////////////
        system.refresh_all();
        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        dbg!("QueryProof::verify - before verify_batched_proof");
        dbg!(total_memory);
        dbg!(used_memory);
        let memory_used = ((used_memory as f32) / (total_memory as f32)) * 100.0;
        dbg!(memory_used);
        ////////////////////
        ////////////////////
        ////////////////////
        
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
                range_length,
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
/// # Returns
/// This function returns a `merlin::Transcript`. The transcript is a record
/// of all the operations and data involved in creating a proof.
/// ```
fn make_transcript<T: Transcript>(
    expr: &(impl ProofPlan + Serialize),
    result: &ProvableQueryResult,
    range_length: usize,
    min_row_num: usize,
) -> T {
    let mut transcript = T::new();
    transcript.extend_serialize_as_le(result);
    transcript.extend_serialize_as_le(expr);
    transcript.extend_serialize_as_le(&range_length);
    transcript.extend_serialize_as_le(&min_row_num);
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
