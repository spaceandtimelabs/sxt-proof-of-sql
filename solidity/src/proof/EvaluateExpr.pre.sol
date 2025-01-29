// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol"; // solhint-disable-line no-global-import

library EvaluateExpr {
    function yulWrapperPlaceholder() public pure {
        assembly {
            // IMPORT-YUL VerificationBuilder.sol
            function consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL VerificationBuilder.sol
            function produce_identity_subpolynomial(builder_ptr, evaluation, degree) {
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
            function verifier_evaluate_proof_expr(_expr_ptr, builder_ptr, accessor_ptr, input_one_eval) ->
                expr_ptr,
                evaluation
            {
                expr_ptr := _expr_ptr
                switch calldataload_u32(expr_ptr)
                // COLUMN_EXPR_NUM = 0
                case 0 { expr_ptr, evaluation := column_expr_evaluate(increment_u32(expr_ptr), accessor_ptr) }
                // EQUALS_EXPR_NUM = 1
                case 1 {
                    expr_ptr, evaluation :=
                        equals_expr_evaluate(increment_u32(expr_ptr), builder_ptr, accessor_ptr, input_one_eval)
                }
                // LITERAL_EXPR_NUM = 2
                case 2 { expr_ptr, evaluation := literal_expr_evaluate(increment_u32(expr_ptr), input_one_eval) }
                default {
                    mstore(0, UNSUPPORTED_EXPR_TYPE)
                    revert(0, 4)
                }
            }
            function literal_expr_evaluate(_expr_ptr, input_one_eval) -> expr_ptr, evaluation {
                expr_ptr := _expr_ptr
                if sub(byte(0, calldataload_u32(expr_ptr)), BIGINT_TYPE_NUM) {
                    mstore(0, UNSUPPORTED_LITERAL_TYPE)
                    revert(0, 4)
                }
                expr_ptr := increment_u32(expr_ptr)
                evaluation := mulmod(add(signextend(7, calldataload_u64(expr_ptr)), MODULUS), input_one_eval, MODULUS)
                expr_ptr := increment_u64(expr_ptr)
            }
            function column_expr_evaluate(_expr_ptr, accessor_ptr) -> expr_ptr, evaluation {
                expr_ptr := _expr_ptr
                let column_num := calldataload_u64(expr_ptr)
                expr_ptr := increment_u64(expr_ptr)
                evaluation := calldataload(add(accessor_ptr, shl(WORD_SHIFT, column_num)))
            }
            function equals_expr_evaluate(_expr_ptr, builder_ptr, accessor_ptr, input_one_eval) -> expr_ptr, evaluation
            {
                expr_ptr := _expr_ptr
                let lhs_eval
                expr_ptr, lhs_eval := verifier_evaluate_proof_expr(expr_ptr, builder_ptr, accessor_ptr, input_one_eval)
                let rhs_eval
                expr_ptr, rhs_eval := verifier_evaluate_proof_expr(expr_ptr, builder_ptr, accessor_ptr, input_one_eval)
                let diff_eval := addmod(lhs_eval, mulmod(MODULUS_MINUS_ONE, rhs_eval, MODULUS), MODULUS)
                let diff_star_eval := consume_final_round_mle(builder_ptr)
                evaluation := consume_final_round_mle(builder_ptr)
                produce_identity_subpolynomial(builder_ptr, mulmod(evaluation, diff_eval, MODULUS), 2)
                produce_identity_subpolynomial(
                    builder_ptr,
                    addmod(
                        input_one_eval,
                        mulmod(
                            MODULUS_MINUS_ONE,
                            addmod(mulmod(diff_eval, diff_star_eval, MODULUS), evaluation, MODULUS),
                            MODULUS
                        ),
                        MODULUS
                    ),
                    2
                )
            }
        }
    }
}
