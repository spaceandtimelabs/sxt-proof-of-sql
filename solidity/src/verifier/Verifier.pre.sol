// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

library Verifier {
    function __verify(
        bytes calldata __result,
        bytes calldata __plan,
        bytes calldata __proof,
        uint256[] memory __tableLengths,
        uint256[] memory __commitments
    ) public view {
        assembly {
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function read_uint64_array(proof_ptr_init) -> proof_ptr, array_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function read_word_array(proof_ptr_init) -> proof_ptr, array_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function read_wordx2_array(proof_ptr_init) -> proof_ptr, array_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function calldata_ec_add_assign(args_ptr, c_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function calldata_ec_mul_add_assign(args_ptr, c_ptr, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function constant_ec_mul_add_assign(args_ptr, c_x, c_y, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_add(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_add_assign(args_ptr, c_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_mul(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_mul_assign(args_ptr, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_pairing_x2(args_ptr) -> success {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_truncated_lagrange_basis_inner_product(length, x_ptr, y_ptr, num_vars) -> result {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_truncated_lagrange_basis_sum(length, x_ptr, num_vars) -> result {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.sol
            function log2_up(value) -> exponent {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function append_array(transcript_ptr, array_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function append_calldata(transcript_ptr, offset, size) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenge(transcript_ptr) -> result {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenges(transcript_ptr, count) -> result_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_challenge(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_aggregate_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_chi_evaluations(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_column_evaluations(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_final_round_commitments(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_final_round_mles(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_first_round_commitments(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_first_round_mles(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluations(builder_ptr) -> values_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_new() -> builder_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_zerosum_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_aggregate_evaluation(builder_ptr, value) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_bit_distributions(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_challenges(builder_ptr, challenges_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_chi_evaluations(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_column_evaluations(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_constraint_multipliers(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_final_round_commitments(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_final_round_mles(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_first_round_commitments(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_first_round_mles(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_max_degree(builder_ptr, value) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_rho_evaluations(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_row_multipliers_evaluation(builder_ptr, value) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_table_chi_evaluations(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGHelpers.pre.sol
            function bivariate_evaluation(v_ptr, q, d, ell) -> b {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGHelpers.pre.sol
            function check_v_consistency(v_ptr, r, x, y) {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGHelpers.pre.sol
            function compute_gl_msm(com_ptr, length, w_ptr, commitment_ptr, r, q, d, b, scratch) {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGHelpers.pre.sol
            function run_transcript(com_ptr, v_ptr, w_ptr, transcript_ptr, ell) -> r, q, d {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGHelpers.pre.sol
            function univariate_group_evaluation(g_ptr, e, length, scratch) {
                revert(0, 0)
            }
            // IMPORT-YUL ../hyperkzg/HyperKZGVerifier.pre.sol
            function verify_hyperkzg(proof_ptr, transcript_ptr, commitment_ptr, x, y) {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/ColumnExpr.pre.sol
            function column_expr_evaluate(expr_ptr, builder_ptr) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/EqualsExpr.pre.sol
            function equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/LiteralExpr.pre.sol
            function literal_expr_evaluate(expr_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/ProofExpr.pre.sol
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/FilterExec.pre.sol
            function compute_folds(plan_ptr, builder_ptr, input_chi_eval) ->
                plan_ptr_out,
                c_fold,
                d_fold,
                evaluations_ptr
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/FilterExec.pre.sol
            function filter_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../sumcheck/Sumcheck.pre.sol
            function process_round(proof_ptr, degree, challenge) -> proof_ptr_out, round_evaluation, actual_sum {
                revert(0, 0)
            }
            // IMPORT-YUL ../sumcheck/Sumcheck.pre.sol
            function verify_sumcheck_proof(transcript_ptr, proof_ptr, num_vars) ->
                proof_ptr_out,
                evaluation_point_ptr,
                expected_evaluation,
                degree
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_plans/ProofPlan.pre.sol
            function proof_plan_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr {
                revert(0, 0)
            }

            function read_first_round_message(proof_ptr_init, transcript_ptr, builder_ptr) ->
                proof_ptr,
                range_length,
                num_challenges
            {
                proof_ptr := proof_ptr_init

                range_length := shr(UINT64_PADDING_BITS, calldataload(proof_ptr))
                proof_ptr := add(proof_ptr, UINT64_SIZE)

                num_challenges := shr(UINT64_PADDING_BITS, calldataload(proof_ptr))
                proof_ptr := add(proof_ptr, UINT64_SIZE)

                let array_ptr

                proof_ptr, array_ptr := read_uint64_array(proof_ptr)
                builder_set_chi_evaluations(builder_ptr, array_ptr)

                proof_ptr, array_ptr := read_uint64_array(proof_ptr)
                builder_set_rho_evaluations(builder_ptr, array_ptr)

                proof_ptr, array_ptr := read_wordx2_array(proof_ptr)
                builder_set_first_round_commitments(builder_ptr, array_ptr)

                append_calldata(transcript_ptr, proof_ptr_init, sub(proof_ptr, proof_ptr_init))
            }
            function read_final_round_message(proof_ptr_init, transcript_ptr, builder_ptr) -> proof_ptr, num_constraints
            {
                proof_ptr := proof_ptr_init

                num_constraints := shr(UINT64_PADDING_BITS, calldataload(proof_ptr))
                proof_ptr := add(proof_ptr, UINT64_SIZE)

                let array_ptr

                proof_ptr, array_ptr := read_wordx2_array(proof_ptr)
                builder_set_final_round_commitments(builder_ptr, array_ptr)

                proof_ptr, array_ptr := read_wordx2_array(proof_ptr)
                builder_set_bit_distributions(builder_ptr, array_ptr)

                append_calldata(transcript_ptr, proof_ptr_init, sub(proof_ptr, proof_ptr_init))
            }
            function read_and_verify_sumcheck_proof(proof_ptr_init, transcript_ptr, builder_ptr, num_vars) ->
                proof_ptr,
                evaluation_point_ptr
            {
                let expected_evaluation, sumcheck_degree
                proof_ptr, evaluation_point_ptr, expected_evaluation, sumcheck_degree :=
                    verify_sumcheck_proof(transcript_ptr, proof_ptr_init, num_vars)
                builder_set_aggregate_evaluation(builder_ptr, mulmod(MODULUS_MINUS_ONE, expected_evaluation, MODULUS))
                builder_set_max_degree(builder_ptr, sumcheck_degree)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_evaluations(evaluation_point_ptr, array_ptr) {
                revert(0, 0)
            }
            function read_pcs_evaluations(proof_ptr_init, transcript_ptr, builder_ptr) -> proof_ptr {
                proof_ptr := proof_ptr_init

                let array_ptr

                proof_ptr, array_ptr := read_word_array(proof_ptr)
                builder_set_first_round_mles(builder_ptr, array_ptr)

                proof_ptr, array_ptr := read_word_array(proof_ptr)
                builder_set_column_evaluations(builder_ptr, array_ptr)

                proof_ptr, array_ptr := read_word_array(proof_ptr)
                builder_set_final_round_mles(builder_ptr, array_ptr)

                append_calldata(transcript_ptr, proof_ptr_init, sub(proof_ptr, proof_ptr_init))
            }
            // IMPORT-YUL PlanUtil.sol
            function skip_plan_names(plan_ptr) -> plan_ptr_out {
                revert(0, 0)
            }

            // IMPORT-YUL ../hyperkzg/HyperKZGBatch.pre.sol
            function batch_pcs(args_ptr, transcript_ptr, commitments_ptr, evaluations_ptr, batch_eval) -> batch_eval_out
            {
                revert(0, 0)
            }

            // TODO: possibly move this to another file and add unit tests
            function verify_pcs_evaluations(
                proof_ptr, commitments_ptr, transcript_ptr, builder_ptr, evaluation_point_ptr
            ) {
                let batch_commitment_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(batch_commitment_ptr, WORDX5_SIZE))
                mstore(batch_commitment_ptr, 0)
                mstore(add(batch_commitment_ptr, WORD_SIZE), 0)
                let batch_eval := 0
                batch_eval :=
                    batch_pcs(
                        batch_commitment_ptr,
                        transcript_ptr,
                        builder_get_first_round_commitments(builder_ptr),
                        builder_get_first_round_mles(builder_ptr),
                        batch_eval
                    )
                batch_eval :=
                    batch_pcs(
                        batch_commitment_ptr,
                        transcript_ptr,
                        commitments_ptr,
                        builder_get_column_evaluations(builder_ptr),
                        batch_eval
                    )
                batch_eval :=
                    batch_pcs(
                        batch_commitment_ptr,
                        transcript_ptr,
                        builder_get_final_round_commitments(builder_ptr),
                        builder_get_final_round_mles(builder_ptr),
                        batch_eval
                    )

                verify_hyperkzg(proof_ptr, transcript_ptr, batch_commitment_ptr, evaluation_point_ptr, batch_eval)
            }

            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_evaluation_vec(length, evaluation_point_ptr) -> evaluations_ptr {
                revert(0, 0)
            }

            // IMPORT-YUL ResultVerifier.pre.sol
            function verify_result_evaluations(result_ptr, evaluation_point_ptr, evaluations_ptr) {
                revert(0, 0)
            }

            function make_transcript(result_ptr, plan_ptr, table_lengths_ptr, commitments_ptr) -> transcript_ptr {
                transcript_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(transcript_ptr, WORD_SIZE))
                mstore(transcript_ptr, INITIAL_TRANSCRIPT_STATE)

                append_calldata(transcript_ptr, plan_ptr, calldataload(sub(plan_ptr, WORD_SIZE)))
                append_calldata(transcript_ptr, result_ptr, calldataload(sub(result_ptr, WORD_SIZE)))
                append_array(transcript_ptr, table_lengths_ptr)

                let commitment_len := mload(commitments_ptr)
                mstore(commitments_ptr, mulmod(commitment_len, 2, MODULUS))
                append_array(transcript_ptr, commitments_ptr)
                mstore(commitments_ptr, commitment_len)

                mstore(mload(FREE_PTR), mload(transcript_ptr))
                mstore(add(mload(FREE_PTR), WORD_SIZE), 0)
                mstore(transcript_ptr, keccak256(mload(FREE_PTR), add(UINT64_SIZE, WORD_SIZE)))
            }

            function verify_proof(result_ptr, plan_ptr, proof_ptr, table_lengths_ptr, commitments_ptr) ->
                evaluation_point_ptr,
                evaluations_ptr
            {
                let transcript_ptr := make_transcript(result_ptr, plan_ptr, table_lengths_ptr, commitments_ptr)
                let builder_ptr := builder_new()
                builder_set_table_chi_evaluations(builder_ptr, table_lengths_ptr)

                let range_length
                {
                    let num_challenges
                    proof_ptr, range_length, num_challenges :=
                        read_first_round_message(proof_ptr, transcript_ptr, builder_ptr)

                    builder_set_challenges(builder_ptr, draw_challenges(transcript_ptr, num_challenges))
                }
                {
                    let num_constraints
                    proof_ptr, num_constraints := read_final_round_message(proof_ptr, transcript_ptr, builder_ptr)

                    builder_set_constraint_multipliers(builder_ptr, draw_challenges(transcript_ptr, num_constraints))
                }
                let num_vars := log2_up(range_length)
                let row_multipliers_challenges := draw_challenges(transcript_ptr, num_vars)

                proof_ptr, evaluation_point_ptr :=
                    read_and_verify_sumcheck_proof(proof_ptr, transcript_ptr, builder_ptr, num_vars)

                proof_ptr := read_pcs_evaluations(proof_ptr, transcript_ptr, builder_ptr)

                verify_pcs_evaluations(proof_ptr, commitments_ptr, transcript_ptr, builder_ptr, evaluation_point_ptr)

                compute_evaluations(evaluation_point_ptr, builder_get_table_chi_evaluations(builder_ptr))
                compute_evaluations(evaluation_point_ptr, builder_get_chi_evaluations(builder_ptr))

                builder_set_row_multipliers_evaluation(
                    builder_ptr,
                    compute_truncated_lagrange_basis_inner_product(
                        range_length,
                        add(row_multipliers_challenges, WORD_SIZE),
                        add(evaluation_point_ptr, WORD_SIZE),
                        num_vars
                    )
                )

                plan_ptr := skip_plan_names(plan_ptr)
                plan_ptr, evaluations_ptr := proof_plan_evaluate(plan_ptr, builder_ptr)
                if builder_get_aggregate_evaluation(builder_ptr) { err(ERR_AGGREGATE_EVALUATION_MISMATCH) }
            }

            function verify_query(result_ptr, plan_ptr, proof_ptr, table_lengths_ptr, commitments_ptr) {
                let evaluation_point_ptr, evaluations_ptr :=
                    verify_proof(result_ptr, plan_ptr, proof_ptr, table_lengths_ptr, commitments_ptr)
                verify_result_evaluations(result_ptr, evaluation_point_ptr, evaluations_ptr)
            }

            mstore(__commitments, div(mload(__commitments), 2))
            verify_query(__result.offset, __plan.offset, __proof.offset, __tableLengths, __commitments)
        }
    }
}
