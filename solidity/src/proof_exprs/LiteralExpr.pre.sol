// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title LiteralExpr
/// @dev Library for handling literal expressions
library LiteralExpr {
    enum LiteralVariant {
        BigInt,
        Int,
        SmallInt,
        TinyInt,
        Boolean
    }

    /// @notice Evaluates a literal expression
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// literal_expr_evaluate(expr_ptr, chi_eval) -> expr_ptr_out, eval
    /// ```
    /// ##### Parameters
    /// * `expr_ptr` - the calldata pointer to the beginning of the expression data
    /// * `chi_eval` - the chi value for evaluation
    /// ##### Return Values
    /// * `expr_ptr_out` - the pointer to the remaining expression after consuming the literal expression
    /// * `eval` - the evaluated result
    /// ##### Proof Plan Encoding
    /// The literal expression is encoded as follows:
    /// 1. The literal variant (as a uint32)
    /// 2. The literal value, which is variant-specific
    ///     a. BigInt: The literal value as a signed int64
    ///     b. Other variants are unsupported at this time
    /// @dev This function evaluates a literal expression by multiplying the literal value by chi_eval.
    /// This is because `chi_eval` is the evaluation of a column of ones of the appropriate length.
    /// @param __expr The literal expression data
    /// @param __chiEval The chi value for evaluation
    /// @return __exprOut The remaining expression data after processing
    /// @return __eval The evaluated result
    function __literalExprEvaluate(bytes calldata __expr, uint256 __chiEval)
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

            function literal_expr_evaluate(expr_ptr, chi_eval) -> expr_ptr_out, eval {
                let literal_variant := shr(UINT32_PADDING_BITS, calldataload(expr_ptr))
                expr_ptr := add(expr_ptr, UINT32_SIZE)

                switch literal_variant
                case 0 {
                    case_const(0, LITERAL_BIGINT_VARIANT)
                    eval :=
                        add(signextend(INT64_SIZE_MINUS_ONE, shr(INT64_PADDING_BITS, calldataload(expr_ptr))), MODULUS)
                    expr_ptr := add(expr_ptr, INT64_SIZE)
                }
                case 1 {
                    case_const(1, LITERAL_INT_VARIANT)
                    eval :=
                        add(signextend(INT32_SIZE_MINUS_ONE, shr(INT32_PADDING_BITS, calldataload(expr_ptr))), MODULUS)
                    expr_ptr := add(expr_ptr, INT32_SIZE)
                }
                case 2 {
                    case_const(2, LITERAL_SMALLINT_VARIANT)
                    eval :=
                        add(signextend(INT16_SIZE_MINUS_ONE, shr(INT16_PADDING_BITS, calldataload(expr_ptr))), MODULUS)
                    expr_ptr := add(expr_ptr, INT16_SIZE)
                }
                case 3 {
                    case_const(3, LITERAL_TINYINT_VARIANT)
                    eval :=
                        add(signextend(INT8_SIZE_MINUS_ONE, shr(INT8_PADDING_BITS, calldataload(expr_ptr))), MODULUS)
                    expr_ptr := add(expr_ptr, INT8_SIZE)
                }
                default { err(ERR_UNSUPPORTED_LITERAL_VARIANT) }
                eval := mulmod(eval, chi_eval, MODULUS)

                expr_ptr_out := expr_ptr
            }
            let __exprOutOffset
            __exprOutOffset, __eval := literal_expr_evaluate(__expr.offset, __chiEval)
            __exprOut.offset := __exprOutOffset
            // slither-disable-next-line write-after-write
            __exprOut.length := sub(__expr.length, sub(__exprOutOffset, __expr.offset))
        }
    }
}
