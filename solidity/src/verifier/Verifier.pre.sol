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
        uint256[1] memory __transcript,
        uint256[] memory __tableLengths,
        uint256[2][] memory __commitments
    ) public pure returns (uint256[] memory __evaluations, uint256[] memory __evaluationPoint) {
        uint256[] memory __evaluationsPtr = new uint256[](0);
        uint256[] memory __evaluationPointPtr = new uint256[](0);
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function append_calldata(transcript_ptr, offset, size) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenges(transcript_ptr, count) -> result_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_new() -> builder_ptr {
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
            function builder_set_rho_evaluations(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_first_round_mles(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_column_evaluations(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_final_round_mles(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            function builder_set_first_round_commitments(builder_ptr, values_ptr) {}
            function builder_set_final_round_commitments(builder_ptr, values_ptr) {}
            function builder_set_bit_distributions(builder_ptr, values_ptr) {}
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_aggregate_evaluation(builder_ptr, value) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_max_degree(builder_ptr, value) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_set_constraint_multipliers(builder_ptr, values_ptr) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_aggregate_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.sol
            function log2_up(value) -> exponent {
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

                proof_ptr, array_ptr := read_word_array(proof_ptr)
                builder_set_first_round_commitments(builder_ptr, array_ptr)

                append_calldata(transcript_ptr, proof_ptr_init, sub(proof_ptr, proof_ptr_init))
            }
            function read_final_round_message(proof_ptr_init, transcript_ptr, builder_ptr) -> proof_ptr, num_constraints
            {
                proof_ptr := proof_ptr_init

                num_constraints := shr(UINT64_PADDING_BITS, calldataload(proof_ptr))
                proof_ptr := add(proof_ptr, UINT64_SIZE)

                let array_ptr

                proof_ptr, array_ptr := read_word_array(proof_ptr)
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
            function compute_evaluations(builder_ptr, evaluation_point_ptr, table_lengths_ptr) {
                // TODO: Implement
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
            function verifier_evaluate_proof_plan(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr {
                evaluations_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluations_ptr, WORD_SIZE))
                mstore(evaluations_ptr, 0)
                // TODO: Implement
                builder_set_aggregate_evaluation(builder_ptr, 0)
            }
            function verify_pcs_evaluations(proof_ptr, commitments_ptr, builder_ptr) -> proof_ptr_out {
                // TODO: Implement
                proof_ptr_out := proof_ptr
            }

            function verify_query(result_ptr, plan_ptr, proof_ptr, transcript_ptr, table_lengths_ptr, commitments_ptr)
                -> evaluations_ptr, evaluation_point_ptr {
                append_calldata(transcript_ptr, plan_ptr, calldataload(sub(plan_ptr, WORD_SIZE)))
                append_calldata(transcript_ptr, result_ptr, calldataload(sub(result_ptr, WORD_SIZE)))

                mstore(mload(FREE_PTR), mload(transcript_ptr))
                mstore(add(mload(FREE_PTR), WORD_SIZE), 0)
                mstore(transcript_ptr, keccak256(mload(FREE_PTR), add(UINT64_SIZE, WORD_SIZE)))

                let builder_ptr := builder_new()

                let range_length, num_challenges
                proof_ptr, range_length, num_challenges :=
                    read_first_round_message(proof_ptr, transcript_ptr, builder_ptr)

                let num_vars := log2_up(range_length)

                builder_set_challenges(builder_ptr, draw_challenges(transcript_ptr, num_challenges))

                let num_constraints
                proof_ptr, num_constraints := read_final_round_message(proof_ptr, transcript_ptr, builder_ptr)

                builder_set_constraint_multipliers(builder_ptr, draw_challenges(transcript_ptr, num_constraints))
                pop(draw_challenges(transcript_ptr, num_vars))

                proof_ptr, evaluation_point_ptr :=
                    read_and_verify_sumcheck_proof(proof_ptr, transcript_ptr, builder_ptr, num_vars)

                compute_evaluations(builder_ptr, evaluation_point_ptr, table_lengths_ptr)

                proof_ptr := read_pcs_evaluations(proof_ptr, transcript_ptr, builder_ptr)

                plan_ptr, evaluations_ptr := verifier_evaluate_proof_plan(plan_ptr, builder_ptr)

                if builder_get_aggregate_evaluation(builder_ptr) { err(ERR_AGGREGATE_EVALUATION_MISMATCH) }

                proof_ptr := verify_pcs_evaluations(proof_ptr, commitments_ptr, builder_ptr)
            }
            __evaluationsPtr, __evaluationPointPtr :=
                verify_query(__result.offset, __plan.offset, __proof.offset, __transcript, __tableLengths, __commitments)
        }
        __evaluations = __evaluationsPtr;
        __evaluationPoint = __evaluationPointPtr;
    }
}
