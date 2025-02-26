// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../proof/VerificationBuilder.pre.sol";

/// @title ProofExpr
/// @dev Library for handling proof expressions which can be either column or literal expressions
library ProofExpr {
    enum ExprVariant {
        Column,
        Literal
    }

    /// @notice Evaluates a proof expression
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval
    /// ```
    /// ##### Parameters
    /// * `expr_ptr` - calldata pointer to the proof expression
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `chi_eval` - the chi value for literal evaluation
    /// ##### Return Values
    /// * `expr_ptr_out` - pointer to the remaining expression after consuming the proof expression
    /// * `eval` - the evaluation result from either column lookup or literal evaluation
    /// @dev Reads the variant and delegates to the appropriate expression evaluator
    /// @param __expr The input proof expression
    /// @param __builder The verification builder containing column evaluations
    /// @param __chiEval The chi value for literal evaluation
    /// @return __exprOut The remaining expression after consuming the proof expression
    /// @return __eval The evaluation result
    function __proofExprEvaluate( // solhint-disable-line gas-calldata-parameters
    bytes calldata __expr, VerificationBuilder.Builder memory __builder, uint256 __chiEval)
        external
        pure
        returns (bytes calldata __exprOut, uint256 __eval)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> eval {
                revert(0, 0)
            }
            // IMPORT-YUL ColumnExpr.pre.sol
            function column_expr_evaluate(expr_ptr, builder_ptr) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL LiteralExpr.pre.sol
            function literal_expr_evaluate(expr_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }

            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                let proof_expr_variant := shr(UINT32_PADDING_BITS, calldataload(expr_ptr))
                expr_ptr := add(expr_ptr, UINT32_SIZE)

                switch proof_expr_variant
                case 0 {
                    case_const(0, COLUMN_EXPR_VARIANT)
                    expr_ptr_out, eval := column_expr_evaluate(expr_ptr, builder_ptr)
                }
                case 1 {
                    case_const(1, LITERAL_EXPR_VARIANT)
                    expr_ptr_out, eval := literal_expr_evaluate(expr_ptr, chi_eval)
                }
                default { err(ERR_UNSUPPORTED_PROOF_EXPR_VARIANT) }
            }
            let __exprOutOffset
            __exprOutOffset, __eval := proof_expr_evaluate(__expr.offset, __builder, __chiEval)
            __exprOut.offset := __exprOutOffset
            // slither-disable-next-line write-after-write
            __exprOut.length := sub(__expr.length, sub(__exprOutOffset, __expr.offset))
        }
    }
}
