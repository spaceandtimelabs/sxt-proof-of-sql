// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title Result Verifier Library
/// @notice A library for verifying result evaluations.
library ResultVerifier {
    /// @notice Verifies that the evaluations of a column match with the provided evaluations.
    /// @notice Let \\(C\\) be a column of data, and let \\(E\\) be the evaluation vector.
    /// @notice The result evaluation is \\(\sum C[i] \cdot E[i]\\).
    /// @notice This is a wrapper around the `verify_result_evaluations` Yul function.
    /// @dev The format of the result is as follows:
    /// @dev * number of columns (64 bit unsigned integer)
    /// @dev * the following for each column:
    /// @dev   * column name length (64 bit unsigned integer)
    /// @dev   * column name (variable length - number of bytes specified by the column name length)
    /// @dev   * optional "quote" - must be 0 in the current implementation (single bytes)
    /// @dev   * column variant (32 bit unsigned integer)
    /// @dev       * Only supports the BigInt variant in the current implementation (0)
    /// @dev   * column length (64 bit unsigned integer)
    /// @dev   * column data (variable length
    /// @dev       * number of rows specified by the column length
    /// @dev       * each row is the data type specified by the column variant
    /// @dev           * BigInt - 64 bit signed integer (8 bytes)
    /// @param __result The result data: the columns.
    /// @param __evaluationPoint The evaluation point.
    /// @param __evaluations The evaluations to check against.
    function __verifyResultEvaluations(
        bytes calldata __result,
        uint256[] memory __evaluationPoint,
        uint256[] memory __evaluations
    ) external pure {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/LagrangeBasisEvaluation.pre.sol
            function compute_evaluation_vec(length, evaluation_point_ptr) -> evaluations_ptr {
                revert(0, 0)
            }

            function verify_result_evaluations(result_ptr, evaluation_point_ptr, evaluations_ptr) {
                let result_ptr_end := add(result_ptr, calldataload(sub(result_ptr, WORD_SIZE)))

                let num_columns := shr(UINT64_PADDING_BITS, calldataload(result_ptr))
                result_ptr := add(result_ptr, UINT64_SIZE)
                if sub(num_columns, mload(evaluations_ptr)) { err(ERR_RESULT_COLUMN_COUNT_MISMATCH) }
                evaluations_ptr := add(evaluations_ptr, WORD_SIZE)

                let first := 1
                let table_len
                let eval_vec
                for {} num_columns { num_columns := sub(num_columns, 1) } {
                    let name_length := shr(UINT64_PADDING_BITS, calldataload(result_ptr))
                    result_ptr := add(result_ptr, add(UINT64_SIZE, name_length))
                    if byte(0, calldataload(result_ptr)) { err(ERR_INVALID_RESULT_COLUMN_NAME) }
                    result_ptr := add(result_ptr, 1)

                    let value := mload(evaluations_ptr)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                    let column_variant := shr(UINT32_PADDING_BITS, calldataload(result_ptr))
                    result_ptr := add(result_ptr, UINT32_SIZE)

                    let column_length := shr(UINT64_PADDING_BITS, calldataload(result_ptr))
                    result_ptr := add(result_ptr, UINT64_SIZE)

                    switch column_variant
                    case 0 { case_const(0, COLUMN_BIGINT_VARIANT) }
                    default { err(ERR_UNSUPPORTED_LITERAL_VARIANT) }

                    if first {
                        first := 0
                        table_len := column_length
                        eval_vec := compute_evaluation_vec(table_len, evaluation_point_ptr)
                    }
                    if sub(table_len, column_length) { err(ERR_INCONSISTENT_RESULT_COLUMN_LENGTHS) }

                    value := mulmod(MODULUS_MINUS_ONE, value, MODULUS)
                    for { let i := 0 } sub(table_len, i) { i := add(i, 1) } {
                        let entry
                        switch column_variant
                        case 0 {
                            case_const(0, COLUMN_BIGINT_VARIANT)
                            entry :=
                                add(
                                    MODULUS,
                                    signextend(INT64_SIZE_MINUS_ONE, shr(INT64_PADDING_BITS, calldataload(result_ptr)))
                                )
                            result_ptr := add(result_ptr, INT64_SIZE)
                        }
                        value := addmod(value, mulmod(entry, mload(add(eval_vec, mul(i, WORD_SIZE))), MODULUS), MODULUS)
                    }
                    if value { err(ERR_INCORRECT_RESULT) }
                }

                if sub(result_ptr_end, result_ptr) { err(0) }
            }
            verify_result_evaluations(__result.offset, __evaluationPoint, __evaluations)
        }
    }
}
