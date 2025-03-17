// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../proof/VerificationBuilder.pre.sol";

/// @title FilterExec
/// @dev Library for handling filter execution plans
library FilterExec {
    /// @notice Evaluates a filter execution plan
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// filter_exec_evaluate(plan_ptr, builder_ptr, accessor_ptr, one_evals) -> plan_ptr_out, evaluations_ptr
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - calldata pointer to the filter execution plan
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `accessor_ptr` - calldata pointer to column evaluations
    /// * `one_evals` - memory pointer to one evaluations
    /// ##### Return Values
    /// * `plan_ptr_out` - pointer to the remaining plan after consuming the filter execution plan
    /// * `evaluations_ptr` - pointer to the evaluations
    /// @dev Evaluates a filter execution plan by checking the filter condition on each row
    /// @param __plan The filter execution plan data
    /// @param __builder The verification builder
    /// @return __planOut The remaining plan after processing
    /// @return __builderOut The verification builder result
    /// @return __evaluationsPtr The evaluations pointer
    function __filterExecEvaluate( // solhint-disable-line gas-calldata-parameters
    bytes calldata __plan, VerificationBuilder.Builder memory __builder)
        external
        pure
        returns (
            bytes calldata __planOut,
            VerificationBuilder.Builder memory __builderOut,
            uint256[] memory __evaluationsPtr
        )
    {
        uint256[] memory __evaluations;
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_consume_challenge(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_consume_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_produce_zerosum_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/ColumnExpr.pre.sol
            function column_expr_evaluate(expr_ptr, builder_ptr) -> expr_ptr_out, eval {
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
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/EqualsExpr.pre.sol
            function equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }

            function compute_folds(plan_ptr, builder_ptr, input_chi_eval) ->
                plan_ptr_out,
                c_fold,
                d_fold,
                evaluations_ptr
            {
                let beta := builder_consume_challenge(builder_ptr)

                let column_count := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                evaluations_ptr := mload(FREE_PTR)
                mstore(evaluations_ptr, column_count)
                evaluations_ptr := add(evaluations_ptr, WORD_SIZE)

                for { let i := column_count } i { i := sub(i, 1) } {
                    let c
                    plan_ptr, c := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)
                    c_fold := addmod(mulmod(c_fold, beta, MODULUS), c, MODULUS)
                }

                for { let i := column_count } i { i := sub(i, 1) } {
                    let d := builder_consume_final_round_mle(builder_ptr)
                    d_fold := addmod(mulmod(d_fold, beta, MODULUS), d, MODULUS)

                    mstore(evaluations_ptr, d)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                }
                evaluations_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluations_ptr, add(WORD_SIZE, mul(column_count, WORD_SIZE))))
                plan_ptr_out := plan_ptr
            }

            function filter_exec_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr {
                let alpha := builder_consume_challenge(builder_ptr)

                let input_chi_eval :=
                    builder_get_table_chi_evaluation(builder_ptr, shr(UINT64_PADDING_BITS, calldataload(plan_ptr)))
                plan_ptr := add(plan_ptr, UINT64_SIZE)

                let selection_eval
                plan_ptr, selection_eval := proof_expr_evaluate(plan_ptr, builder_ptr, input_chi_eval)

                let c_fold, d_fold
                plan_ptr, c_fold, d_fold, evaluations_ptr := compute_folds(plan_ptr, builder_ptr, input_chi_eval)
                let c_star := builder_consume_final_round_mle(builder_ptr)
                let d_star := builder_consume_final_round_mle(builder_ptr)
                let output_chi_eval := builder_consume_chi_evaluation(builder_ptr)

                builder_produce_zerosum_constraint(
                    builder_ptr,
                    addmod(mulmod(c_star, selection_eval, MODULUS), mulmod(MODULUS_MINUS_ONE, d_star, MODULUS), MODULUS),
                    2
                )
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        mulmod(add(1, mulmod(alpha, c_fold, MODULUS)), c_star, MODULUS),
                        mulmod(MODULUS_MINUS_ONE, input_chi_eval, MODULUS),
                        MODULUS
                    ),
                    2
                )
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        mulmod(add(1, mulmod(alpha, d_fold, MODULUS)), d_star, MODULUS),
                        mulmod(MODULUS_MINUS_ONE, output_chi_eval, MODULUS),
                        MODULUS
                    ),
                    2
                )
                plan_ptr_out := plan_ptr
            }

            let __planOutOffset
            __planOutOffset, __evaluations := filter_exec_evaluate(__plan.offset, __builder)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
        __evaluationsPtr = __evaluations;
        __builderOut = __builder;
    }
}
