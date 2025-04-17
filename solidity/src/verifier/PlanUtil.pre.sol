// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title Plan Utility Library
/// @notice A library for handling utility functions related to plans.
library PlanUtil {
    /// @notice The Proof Plan is prefixed with metadata about the plan, primarily the names of the tables and columns.
    /// @notice This method skips over the names in a plan and returns the updated pointer.
    /// @dev The format of the plan is as follows:
    /// @dev * number of tables (uint64)
    /// @dev * table names
    /// @dev     * length of table name (uint64)
    /// @dev     * table name (variable length)
    /// @dev * number of columns (uint64)
    /// @dev * column names
    /// @dev     * index of the table the column belongs to (uint64)
    /// @dev     * length of column name (uint64)
    /// @dev     * column name (variable length)
    /// @dev     * column type (uint32)
    /// @dev * number of output columns (uint64)
    /// @dev * output column names
    /// @dev     * length of output column name (uint64)
    /// @dev     * output column name (variable length)
    /// @param __plan The calldata pointer to the plan.
    /// @return __planOut The updated pointer after skipping names.
    function __skipPlanNames(bytes calldata __plan) external pure returns (bytes calldata __planOut) {
        assembly {
            function skip_plan_names(plan_ptr) -> plan_ptr_out {
                // skip over the table names
                let num_tables := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_tables { num_tables := sub(num_tables, 1) } {
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                }
                // skip over the column names
                let num_columns := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_columns { num_columns := sub(num_columns, 1) } {
                    plan_ptr := add(plan_ptr, UINT64_SIZE)
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                    plan_ptr := add(plan_ptr, UINT32_SIZE)
                }
                // skip over the output column names
                let num_outputs := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                plan_ptr := add(plan_ptr, UINT64_SIZE)
                for {} num_outputs { num_outputs := sub(num_outputs, 1) } {
                    let name_len := shr(UINT64_PADDING_BITS, calldataload(plan_ptr))
                    plan_ptr := add(plan_ptr, add(UINT64_SIZE, name_len))
                }

                plan_ptr_out := plan_ptr
            }

            let __planOutOffset := skip_plan_names(__plan.offset)
            __planOut.offset := __planOutOffset
            // slither-disable-next-line write-after-write
            __planOut.length := sub(__plan.length, sub(__planOutOffset, __plan.offset))
        }
    }

    /// @notice Evaluates the full proof plan and returns evaluations.
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// full_proof_plan_evaluate(plan_ptr, builder_ptr) -> evaluations_ptr
    /// ```
    /// ##### Parameters
    /// * `plan_ptr` - pointer to the plan
    /// * `builder_ptr` - pointer to the builder
    /// ##### Return Values
    /// * `evaluations_ptr` - pointer to the evaluations
    /// @param __planPtr The pointer to the plan.
    /// @param __builderPtr The pointer to the builder.
    /// @return __evaluationsPtr The pointer to the evaluations.
    function __fullProofPlanEvaluate(uint256 __planPtr, uint256 __builderPtr)
        internal
        pure
        returns (uint256 __evaluationsPtr)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Queue.pre.sol
            function dequeue(queue_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_challenge(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_chi_evaluation(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_zerosum_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
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
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
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
            // IMPORT-YUL ../proof_exprs/EqualsExpr.pre.sol
            function equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof_exprs/ProofExpr.pre.sol
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_table_chi_evaluation(builder_ptr, table_num) -> value {
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
            // IMPORT-YUL ../proof_plans/ProofPlan.pre.sol
            function proof_plan_evaluate(plan_ptr, builder_ptr) -> plan_ptr_out, evaluations_ptr {
                revert(0, 0)
            }
            // IMPORT-YUL PlanUtil.pre.sol
            function skip_plan_names(plan_ptr) -> plan_ptr_out {
                revert(0, 0)
            }

            function full_proof_plan_evaluate(plan_ptr, builder_ptr) -> evaluations_ptr {
                let plan_ptr_end := add(plan_ptr, calldataload(sub(plan_ptr, WORD_SIZE)))
                plan_ptr := skip_plan_names(plan_ptr)
                let plan_ptr_out
                plan_ptr_out, evaluations_ptr := proof_plan_evaluate(plan_ptr, builder_ptr)
                if sub(plan_ptr_end, plan_ptr_out) { err(0) }
            }
            __evaluationsPtr := full_proof_plan_evaluate(__planPtr, __builderPtr)
        }
    }
}
