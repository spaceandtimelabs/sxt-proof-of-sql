// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol"; // solhint-disable-line no-global-import
import {Transcript} from "../base/Transcript.sol";

library VerifyProof {
    // These errors must be here in order for them to show up in the ABI
    error TooFewChallenges();
    error TooFewFinalRoundMLEs();
    error TooFewOneEvaluations();
    error TooFewOneSubpolynomialMultipliers();
    error UnsupportedLiteralType();
    error UnsupportedExprType();
    error UnsupportedPlanType();
    error RoundEvaluationMismatch();
    error WrongSumcheckSize();
    error UnsupportedProofType();
    error FinalEvaluationMismatch();

    function verifyProof(
        uint256 planPtr_,
        uint256 proofPtr_,
        uint256 transcriptPtr_,
        uint256 numTables_,
        uint256 tableLengthsPtr_
    ) public pure {
        assembly {
            // IMPORT-YUL VerificationBuilder.sol
            function allocate_builder() -> builder_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function set_challenges(builder_ptr, challenge_ptr, challenge_length) {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function consume_challenge(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenge(transcript_ptr, count) -> result_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenges(transcript_ptr, count) -> result_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/MathUtil.sol
            function log2_up(value) -> exponent {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_u32(ptr) -> ptr_out {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_u64(ptr) -> ptr_out {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_u64s(ptr, count) -> ptr_out {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_words(ptr, count) -> ptr_out {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function calldataload_u32(i) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function calldataload_u64(i) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function append_calldata(transcript_ptr, offset, size) {
                revert(0, 0)
            }
            // IMPORT-YUL Sumcheck.pre.sol
            function verify_sumcheck_proof(transcript_ptr, proof_ptr, num_vars, degree) ->
                evaluation_point_ptr,
                expected_evaluation
            {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function set_subpolynomial_multipliers(
                builder_ptr, subpolynomial_multiplier_ptr, subpolynomial_multiplier_length
            ) {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function produce_zerosum_subpolynomial(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function set_sumcheck_evaluations(builder_ptr, entrywise_multiplier_evaluation, max_degree) {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function set_final_round_mles(builder_ptr, final_round_mle_ptr, final_round_mle_length) {
                revert(0, 0)
            }
            // IMPORT-YUL EvaluatePlan.pre.sol
            function verifier_evaluate_proof_plan(plan_ptr, builder_ptr, accessor_ptr, one_evals) ->
                out_expr_ptr,
                evaluations_ptr
            {
                revert(0, 0)
            }
            // IMPORT-YUL EvaluatePlan.pre.sol
            function filter_exec_evaluate(plan_ptr, builder_ptr, accessor_ptr, one_evals) ->
                out_expr_ptr,
                evaluations_ptr
            {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function produce_identity_subpolynomial(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function consume_one_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function set_one_evaluations(builder_ptr, one_evaluation_ptr, one_evaluation_length) {
                revert(0, 0)
            }
            // IMPORT-YUL EvaluateExpr.pre.sol
            function verifier_evaluate_proof_expr(expr_ptr, builder_ptr, accessor_ptr, input_one_eval) ->
                out_expr_ptr,
                evaluation
            {
                revert(0, 0)
            }
            // IMPORT-YUL EvaluateExpr.pre.sol
            function literal_expr_evaluate(expr_ptr, input_one_eval) -> out_expr_ptr, evaluation {
                revert(0, 0)
            }
            // IMPORT-YUL EvaluateExpr.pre.sol
            function column_expr_evaluate(expr_ptr, accessor_ptr) -> out_expr_ptr, evaluation {
                revert(0, 0)
            }
            // IMPORT-YUL EvaluateExpr.pre.sol
            function equals_expr_evaluate(expr_ptr, builder_ptr, accessor_ptr, input_one_eval) ->
                out_expr_ptr,
                evaluation
            {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.sol
            function compute_truncated_lagrange_basis_sum(length, point_ptr, num_vars) -> result {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.sol
            function compute_truncated_lagrange_basis_inner_product(length, a_ptr, b_ptr, num_vars) -> result {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_word(ptr) -> ptr_out {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilderHelper.pre.sol
            function allocate_and_set_one_evaluation_lengths(builder_ptr, _proof_ptr) -> proof_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilderHelper.pre.sol
            function compute_one_evaluations(builder_ptr, point_ptr, num_vars) {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function set_range_length(builder_ptr, range_length) {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function get_range_length(builder_ptr) -> range_length {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function set_entrywise_point_multipliers(builder_ptr, entrywise_point_multipliers) {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function get_entrywise_point_multipliers(builder_ptr) -> entrywise_point_multipliers {
                revert(0, 0)
            }

            function read_first_round_message(_proof_ptr, transcript_ptr, builder_ptr) -> proof_ptr {
                proof_ptr := _proof_ptr

                set_range_length(builder_ptr, calldataload_u64(proof_ptr))
                proof_ptr := increment_u64(proof_ptr)

                let num_challenges := calldataload_u64(proof_ptr)
                proof_ptr := increment_u64(proof_ptr)

                proof_ptr := allocate_and_set_one_evaluation_lengths(builder_ptr, proof_ptr)

                let num_rhos := calldataload_u64(proof_ptr)
                proof_ptr := increment_u64(proof_ptr)
                proof_ptr := increment_u64s(proof_ptr, num_rhos)

                let num_first_round_commitments := calldataload_u64(proof_ptr)
                proof_ptr := increment_u64(proof_ptr)
                proof_ptr := add(proof_ptr, mul(num_first_round_commitments, COMMITMENT_SIZE))

                // Append entire message to transcript
                append_calldata(transcript_ptr, _proof_ptr, sub(proof_ptr, _proof_ptr))

                // Draw challenges from transcript
                set_challenges(builder_ptr, draw_challenges(transcript_ptr, num_challenges), num_challenges)
            }

            function read_final_round_message(_proof_ptr, transcript_ptr, builder_ptr, num_vars) -> proof_ptr {
                proof_ptr := _proof_ptr

                let num_constraints := calldataload_u64(proof_ptr)
                proof_ptr := increment_u64(proof_ptr)

                let num_final_round_commitments := calldataload_u64(proof_ptr)
                proof_ptr := increment_u64(proof_ptr)
                proof_ptr := add(proof_ptr, mul(num_final_round_commitments, COMMITMENT_SIZE))

                proof_ptr := increment_u64(proof_ptr)

                // Append entire message to transcript
                append_calldata(transcript_ptr, _proof_ptr, sub(proof_ptr, _proof_ptr))

                // Draw challenges from transcript
                set_entrywise_point_multipliers(builder_ptr, draw_challenges(transcript_ptr, num_vars))
                set_subpolynomial_multipliers(
                    builder_ptr, draw_challenges(transcript_ptr, num_constraints), num_constraints
                )
            }

            function read_sumcheck_proof(_proof_ptr, transcript_ptr, num_vars) ->
                proof_ptr,
                sumcheck_degree,
                evaluation_point_ptr,
                expected_evaluation
            {
                proof_ptr := _proof_ptr

                let sumcheck_length := calldataload_u64(proof_ptr)
                sumcheck_degree := div(sumcheck_length, num_vars)
                if or(iszero(sumcheck_degree), mod(sumcheck_length, num_vars)) {
                    mstore(0, WRONG_SUMCHECK_SIZE)
                    revert(0, 4)
                }
                sumcheck_degree := sub(sumcheck_degree, 1)

                proof_ptr := increment_u64(proof_ptr)
                evaluation_point_ptr, expected_evaluation :=
                    verify_sumcheck_proof(transcript_ptr, proof_ptr, num_vars, sumcheck_degree)
                proof_ptr := increment_words(proof_ptr, sumcheck_length)
            }

            function verify_proof(plan_ptr, proof_ptr, transcript_ptr, num_tables, table_lengths_ptr) {
                let builder_ptr := allocate_builder()
                // load first round message into builder, append to transcript, and draw challenges
                proof_ptr := read_first_round_message(proof_ptr, transcript_ptr, builder_ptr)

                let num_vars := log2_up(get_range_length(builder_ptr))

                // load final round message into builder, append to transcript, and draw challenges
                proof_ptr := read_final_round_message(proof_ptr, transcript_ptr, builder_ptr, num_vars)

                // load and verify sumcheck proof
                let sumcheck_degree, evaluation_point_ptr, expected_evaluation
                proof_ptr, sumcheck_degree, evaluation_point_ptr, expected_evaluation :=
                    read_sumcheck_proof(proof_ptr, transcript_ptr, num_vars)

                let table_ref_one_evals_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, increment_words(table_ref_one_evals_ptr, num_tables))
                for {
                    let i := num_tables
                    let ptr := table_ref_one_evals_ptr
                } i {
                    i := sub(i, 1)
                    table_lengths_ptr := increment_word(table_lengths_ptr)
                    ptr := increment_word(ptr)
                } {
                    mstore(
                        ptr,
                        compute_truncated_lagrange_basis_sum(mload(table_lengths_ptr), evaluation_point_ptr, num_vars)
                    )
                }
                set_sumcheck_evaluations(
                    builder_ptr,
                    compute_truncated_lagrange_basis_inner_product(
                        get_range_length(builder_ptr),
                        get_entrywise_point_multipliers(builder_ptr),
                        evaluation_point_ptr,
                        num_vars
                    ),
                    sumcheck_degree
                )
                compute_one_evaluations(builder_ptr, evaluation_point_ptr, num_vars)

                // -------- fourth prover message --------
                let num_first_round_mles := calldataload_u64(proof_ptr)
                proof_ptr := increment_u64(proof_ptr)
                if num_first_round_mles {
                    mstore(0, UNSUPPORTED_PROOF_TYPE)
                    revert(0, 4)
                }
                let num_column_refs := calldataload_u64(proof_ptr)
                proof_ptr := increment_u64(proof_ptr)
                let column_refs_ptr := proof_ptr
                proof_ptr := increment_words(proof_ptr, num_column_refs)
                let num_final_round_mles := calldataload_u64(proof_ptr)
                proof_ptr := increment_u64(proof_ptr)
                set_final_round_mles(builder_ptr, proof_ptr, num_final_round_mles)
                proof_ptr := increment_words(proof_ptr, num_final_round_mles)
                // -------- end fourth prover message --------

                let evaluations_ptr
                plan_ptr, evaluations_ptr :=
                    verifier_evaluate_proof_plan(plan_ptr, builder_ptr, column_refs_ptr, table_ref_one_evals_ptr)

                let final_evaluation := mload(add(builder_ptr, SUMCHECK_EVALUATION_OFFSET))

                if sub(final_evaluation, expected_evaluation) {
                    mstore(0, FINAL_EVALUATION_MISMATCH)
                    revert(0, 4)
                }
            }

            verify_proof(planPtr_, proofPtr_, transcriptPtr_, numTables_, tableLengthsPtr_)
        }
    }

    function _testVerifyProof(bytes calldata plan, bytes calldata result, bytes calldata proof) public pure {
        uint256 planPtr;
        uint256 proofPtr;
        uint256 transcriptPtr;
        uint256 tableLengthsPtr;
        uint256 numTables;

        assembly {
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function calldataload_u64(i) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_u64(i) -> value {
                revert(0, 0)
            }
            planPtr := plan.offset
            let num_tables := calldataload_u64(planPtr)

            numTables := num_tables
            planPtr := increment_u64(planPtr)
            for {} num_tables { num_tables := sub(num_tables, 1) } {
                let len := calldataload_u64(planPtr)
                planPtr := increment_u64(planPtr)
                planPtr := add(planPtr, len)
            }
            let num_columns := calldataload_u64(planPtr)

            planPtr := increment_u64(planPtr)
            for {} num_columns { num_columns := sub(num_columns, 1) } {
                planPtr := increment_u64(planPtr)
                let len := calldataload_u64(planPtr)
                planPtr := increment_u64(planPtr)
                planPtr := add(planPtr, len)
            }
        }

        uint256 state = uint256(
            keccak256(abi.encodePacked(abi.encodePacked(plan), abi.encodePacked(result), abi.encodePacked(uint64(0))))
        );

        uint256[1] memory transcript = Transcript.newTranscript(state);

        uint256[1] memory tableLengths = [uint256(6)];

        assembly {
            proofPtr := proof.offset
            transcriptPtr := transcript
            tableLengthsPtr := tableLengths
        }

        verifyProof(planPtr, proofPtr, transcriptPtr, numTables, tableLengthsPtr);
    }
}

contract VerifyProofTest {
    function testVerifyProof() public pure {
        VerifyProof._testVerifyProof(
            hex"0000000000000001000000000000000f6e616d6573706163652e7461626c6500"
            hex"0000000000000200000000000000000000000000000001620000000000000000"
            hex"0000000000000001610000000000000000000000000000000100000000000000"
            hex"0000000001000000020000000000000000000000050000000000000001000000" hex"000000000000000000",
            hex"0000000000000001000000000000000162000000000400000000000000020000" hex"0000000000000000000000000003",
            hex"0000000000000006000000000000000200000000000000010000000000000002"
            hex"0000000000000000000000000000000000000000000000050000000000000005"
            hex"c696af43e07ada070691ff7c4ed83d07b3c56e90e0cff15c876af7caa54e8887"
            hex"5d707648e5e2be6824083413eb94b67f5c5f4c2149db627f81c64c6c68380b99"
            hex"d913ddf73069020e0075dd4bd0f1b1f5da96357a6f408f904b96664f4abd6199"
            hex"5d8a3a3f96ccfced6d552e0210866cbb7b8172489c7435952212a9cd181a81ae"
            hex"c9eba85394b17e143da201bce6aca2ec6e04e40fbcd22a1ee65ea9cf0be2cd91"
            hex"0000000000000000000000000000000c0ac899b9aaac044dec164137ac4e03b8"
            hex"758ec488c8e0dce55d2a8465d80bbeb62576d7658cdba5a76cfbdce1cfeeaaa8"
            hex"a53aabe8d0a09c7e9c355d77ef54493f0024dd53a9a9f6345f3e279d0544a9fc"
            hex"0d6a77d6e037f72d4a8213b6289ff80c00000000000000000000000000000000"
            hex"0000000000000000000000000000000002cff199a07f5451d55f9ffe271b8ccd"
            hex"d8b984431847e5134b43d9951bf2f0b0011b6f25677756d61e4e4c16516e53e8"
            hex"29ff50d71806628147abb69e6b1b64480436361901c8d230ec8c612e5b181b4d"
            hex"244a77217377337b9bec432f732242720ca974a2e0655c02a15ac9aa0eab73f1"
            hex"bd34f619f38d4b4e61529509773b89a92da59fb5d0a9aa8fb149fe194766bfce"
            hex"c5454dc82e493cdd46244631b2a19af013931a7c0f4904149bf52841e4ca186d"
            hex"14fe48077d5e4d75aed1eb668c38d98005edf7aee6f0769731a33a8a93bd91a8"
            hex"99dd55253405d5d408e83746d7a020461b8ec9da920294c52c49d09fc7c1567e"
            hex"2b9efbf0bade215d76f5a1fed09546b500000000000000000000000000000002"
            hex"26ee0fd6eeffa74b82d05d02c1c409e8f57be060f64f9a6578893527d19c9497"
            hex"0c561f8c1ce0d025b86ad782321f922a50c157e7ae92911d4db57712ebb443ee"
            hex"00000000000000052dff208f8c3e95135288e702ffb06defebf3afc325aca94b"
            hex"6faf1babd58f7cbf1ca0cb1cf3c0af9f4c897ce3558c6c23acc58c536d7d3e3c"
            hex"6cd53f4605d03ce120aae24e4087553b65755af6f2503f7580ac754d7794a9d3"
            hex"a11392b0ab08b75b03bd84a196ec39e27cedab5729deece71aaa3d111274cbe2"
            hex"c193ba07c6bb68800b9abc44068ffd8763379f0df88a51b8c82fa4ef31da524e"
            hex"08da0c505025d3ea00000000000000026399e2e612f724f7ebea5d34e9ceb22f"
            hex"80ebe20de5f4c82b3a7fe822ecf56d93555fb1e72881ce6cbd7c2f5e39267ee3"
            hex"d697ce497039df9e6f858ff28e260f1c0000000000000003fd7674a760a564e9"
            hex"1fc4fc69ab739ae98aba05336b73ed05fd9f1affdfb0461d72279c0433c2c60c"
            hex"0b180a4589d62a3605505dca1c30656127309fcca8cb34135c43d1f2aed0a1b7"
            hex"8b89f34cef36d96465392b04d77cad904c2d532f808e942d0000000000000003"
            hex"00000000000000034bb27505a6a0afd688ff26b0a869fa408d4bc195df059f19"
            hex"1b9b0399b540a11fde1dcc525a1acc331fcf000e111bda2e0c60a2d189309ee1"
            hex"f514f3f8311e790b020951bf031743fd5c94905c8aa081c0ac181033044d337e"
            hex"c39eeb494a685929000000000000000309f2355a25b44e8b565c2be3ceaf6651"
            hex"48baab1ab75fced39805cc64c3362220580889bfc36b30af80c7bbf3a9f80852"
            hex"e30d1430dd01e35eb43c08d37196181662f26f743866f8c8e6283befe3487141"
            hex"7b82cddc876a59091bd31943386fdc2100000000000000036acab0aac1365d55"
            hex"37f0663562b434a6dbffbf1a7eef782815b2068d9425d80456296e0d6c26d89f"
            hex"0524c4edb9c19d1ede27a227397bdfaf68c8f4d9791ec32e06e0d15e3abb32b2"
            hex"0fe69d7e8d3ea73cbf2904cdd5efca932a8c9b0ec054bb1b"
        );
    }
}
