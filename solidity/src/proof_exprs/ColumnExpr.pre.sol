// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../proof/VerificationBuilder.pre.sol";

/// @title ColumnExpr
/// @dev Library for handling column expressions
library ColumnExpr {
    /// @notice Evaluates a column expression
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// column_expr_evaluate(expr_ptr, builder_ptr) -> expr_ptr_out, eval
    /// ```
    /// ##### Parameters
    /// * `expr_ptr` - calldata pointer to the column expression
    /// * `builder_ptr` - memory pointer to the verification builder
    /// ##### Return Values
    /// * `expr_ptr_out` - pointer to the remaining expression after consuming the column expression
    /// * `eval` - the evaluation result from looking up the column value
    /// @dev Reads a uint64 column index from the expression and looks up its evaluation
    /// @param __expr The input column expression
    /// @param __builder The verification builder containing column evaluations
    /// @return __exprOut The remaining expression after consuming the column index
    /// @return __eval The evaluation result for the column
    function __columnExprEvaluate( // solhint-disable-line gas-calldata-parameters
    bytes calldata __expr, VerificationBuilder.Builder memory __builder)
        external
        pure
        returns (bytes calldata __exprOut, uint256 __eval)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> eval {
                revert(0, 0)
            }

            function column_expr_evaluate(expr_ptr, builder_ptr) -> expr_ptr_out, eval {
                let column_num := shr(UINT64_PADDING_BITS, calldataload(expr_ptr))
                expr_ptr := add(expr_ptr, UINT64_SIZE)

                eval := builder_get_column_evaluation(builder_ptr, column_num)

                expr_ptr_out := expr_ptr
            }
            let __exprOutOffset
            __exprOutOffset, __eval := column_expr_evaluate(__expr.offset, __builder)
            __exprOut.offset := __exprOutOffset
            // slither-disable-next-line write-after-write
            __exprOut.length := sub(__expr.length, sub(__exprOutOffset, __expr.offset))
        }
    }
}
