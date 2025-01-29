// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol"; // solhint-disable-line no-global-import

library EvaluatePlan {
    function yulWrapperPlaceholder() public pure {
        assembly {
            // ----- YUL IMPORTS -----
            // IMPORT-YUL VerificationBuilder.sol
            function consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function produce_zerosum_subpolynomial(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function produce_identity_subpolynomial(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function consume_challenge(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function consume_one_evaluation(builder_ptr) -> value {
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
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function calldataload_u32(i) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_u32(i) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function calldataload_u64(i) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/PointerArithmetic.sol
            function increment_u64(i) -> value {
                revert(0, 0)
            }
            // ----- END YUL IMPORTS -----

            function verifier_evaluate_proof_plan(_plan_ptr, builder_ptr, accessor_ptr, one_evals) ->
                plan_ptr,
                evaluations_ptr
            {
                plan_ptr := _plan_ptr
                switch calldataload_u32(plan_ptr)
                // FILTER_EXEC_NUM = 0
                case 0 {
                    plan_ptr, evaluations_ptr :=
                        filter_exec_evaluate(increment_u32(plan_ptr), builder_ptr, accessor_ptr, one_evals)
                }
                default {
                    mstore(0, UNSUPPORTED_PLAN_TYPE)
                    revert(0, 4)
                }
            }

            function filter_exec_evaluate(_plan_ptr, builder_ptr, accessor_ptr, one_evals) -> plan_ptr, evaluations_ptr
            {
                plan_ptr := _plan_ptr

                let alpha := consume_challenge(builder_ptr)
                let beta := consume_challenge(builder_ptr)

                let table_num := calldataload_u64(plan_ptr)
                let input_one_eval := mload(add(one_evals, shl(WORD_SHIFT, table_num)))
                plan_ptr := increment_u64(plan_ptr)

                let s
                plan_ptr, s := verifier_evaluate_proof_expr(plan_ptr, builder_ptr, accessor_ptr, input_one_eval)

                let column_count := calldataload_u64(plan_ptr)
                plan_ptr := increment_u64(plan_ptr)
                let c_fold, d_fold
                for { let i := column_count } i { i := sub(i, 1) } {
                    let c
                    plan_ptr, c := verifier_evaluate_proof_expr(plan_ptr, builder_ptr, accessor_ptr, input_one_eval)
                    c_fold := addmod(mulmod(c_fold, beta, MODULUS), c, MODULUS)
                }
                for { let i := column_count } i { i := sub(i, 1) } {
                    let d := consume_final_round_mle(builder_ptr)
                    d_fold := addmod(mulmod(d_fold, beta, MODULUS), d, MODULUS)
                }
                let c_star := consume_final_round_mle(builder_ptr)
                let d_star := consume_final_round_mle(builder_ptr)
                let output_one_eval := consume_one_evaluation(builder_ptr)

                let evaluation :=
                    addmod(mulmod(c_star, s, MODULUS), mulmod(MODULUS_MINUS_ONE, d_star, MODULUS), MODULUS)
                produce_zerosum_subpolynomial(builder_ptr, evaluation, 2)
                evaluation :=
                    addmod(
                        mulmod(add(1, mulmod(alpha, c_fold, MODULUS)), c_star, MODULUS),
                        mulmod(MODULUS_MINUS_ONE, input_one_eval, MODULUS),
                        MODULUS
                    )
                produce_identity_subpolynomial(builder_ptr, evaluation, 2)
                evaluation :=
                    addmod(
                        mulmod(add(1, mulmod(alpha, d_fold, MODULUS)), d_star, MODULUS),
                        mulmod(MODULUS_MINUS_ONE, output_one_eval, MODULUS),
                        MODULUS
                    )
                produce_identity_subpolynomial(builder_ptr, evaluation, 2)
            }
        }
    }
}
