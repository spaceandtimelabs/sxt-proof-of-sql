// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

/// @title AndExpr
/// @dev Library for handling running AND on two proof expressions
library AndExpr {
    /// @notice Evaluates an and expression by running AND on two sub-expressions
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// and_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval
    /// ```
    /// ##### Parameters
    /// * `expr_ptr` - calldata pointer to the expression data
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `chi_eval` - the chi value for evaluation
    /// ##### Return Values
    /// * `expr_ptr_out` - pointer to the remaining expression after consuming both sub-expressions
    /// * `eval` - the evaluation result from the builder's final round MLE
    /// @notice Evaluates two sub-expressions and multiplies them together
    /// ##### Proof Plan Encoding
    /// The and expression is encoded as follows:
    /// 1. The left hand side expression
    /// 2. The right hand side expression
    /// @param __expr The and expression data
    /// @param __builder The verification builder
    /// @param __chiEval The chi value for evaluation
    /// @return __exprOut The remaining expression after processing
    /// @return __builderOut The verification builder result
    /// @return __eval The evaluated result
    function __andExprEvaluate( // solhint-disable-line gas-calldata-parameters
    bytes calldata __expr, VerificationBuilder.Builder memory __builder, uint256 __chiEval)
        external
        pure
        returns (bytes calldata __exprOut, VerificationBuilder.Builder memory __builderOut, uint256 __eval)
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
            // IMPORT-YUL ../base/SwitchUtil.pre.sol
            function case_const(lhs, rhs) {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_get_column_evaluation(builder_ptr, column_num) -> eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Array.pre.sol
            function get_array_element(arr_ptr, index) -> value {
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
            // IMPORT-YUL EqualsExpr.pre.sol
            function equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL AddExpr.pre.sol
            function add_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL SubtractExpr.pre.sol
            function subtract_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL MultiplyExpr.pre.sol
            function multiply_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL OrExpr.pre.sol
            function or_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL NotExpr.pre.sol
            function not_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../builder/VerificationBuilder.pre.sol
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ProofExpr.pre.sol
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }

            function and_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let lhs_eval
                expr_ptr, lhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let rhs_eval
                expr_ptr, rhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                result_eval := mod(builder_consume_final_round_mle(builder_ptr), MODULUS)
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        result_eval, mulmod(MODULUS_MINUS_ONE, mulmod(lhs_eval, rhs_eval, MODULUS), MODULUS), MODULUS
                    ),
                    2
                )

                expr_ptr_out := expr_ptr
            }

            let __exprOutOffset
            __exprOutOffset, __eval := and_expr_evaluate(__expr.offset, __builder, __chiEval)
            __exprOut.offset := __exprOutOffset
            // slither-disable-next-line write-after-write
            __exprOut.length := sub(__expr.length, sub(__exprOutOffset, __expr.offset))
        }
        __builderOut = __builder;
    }
}
