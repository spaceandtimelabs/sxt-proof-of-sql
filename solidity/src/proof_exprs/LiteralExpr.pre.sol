// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title LiteralExpr
/// @dev Library for handling literal expressions
library LiteralExpr {
    enum LiteralVariant {
        BigInt
    }

    /// @notice Evaluates a literal expression
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// literal_expr_evaluate(expr_ptr_in, chi_eval) -> expr_ptr, eval
    /// ```
    /// ##### Parameters
    /// * `expr_ptr_in` - the calldata pointer to the beginning of the expression data
    /// * `chi_eval` - the chi value for evaluation
    /// ##### Return Values
    /// * `expr_ptr` - the pointer to the remaining expression after consuming the literal expression
    /// * `eval` - the evaluated result
    /// @dev This function evaluates a literal expression by multiplying the literal value by chi_eval.
    /// This is because `chi_eval` is the evaluation of a column of ones of the appropriate length.
    /// @param __exprIn The literal expression data
    /// @param __chiEval The chi value for evaluation
    /// @return __exprOut The remaining expression data after processing
    /// @return __eval The evaluated result
    function __literalExprEvaluate(bytes calldata __exprIn, uint256 __chiEval)
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

            function literal_expr_evaluate(expr_ptr_in, chi_eval) -> expr_ptr, eval {
                expr_ptr := expr_ptr_in

                let literal_variant := shr(UINT32_PADDING_BITS, calldataload(expr_ptr))
                expr_ptr := add(expr_ptr, UINT32_SIZE)

                switch literal_variant
                case 0 {
                    case_const(0, LITERAL_BIGINT_VARIANT)
                    eval :=
                        add(signextend(INT64_SIZE_MINUS_ONE, shr(INT64_PADDING_BITS, calldataload(expr_ptr))), MODULUS)
                    expr_ptr := add(expr_ptr, INT64_SIZE)
                }
                default { err(ERR_UNSUPPORTED_LITERAL_VARIANT) }
                eval := mulmod(eval, chi_eval, MODULUS)
            }
            let __exprOutOffset
            __exprOutOffset, __eval := literal_expr_evaluate(__exprIn.offset, __chiEval)
            __exprOut.offset := __exprOutOffset
            // slither-disable-next-line write-after-write
            __exprOut.length := sub(__exprIn.length, sub(__exprOutOffset, __exprIn.offset))
        }
    }
}
