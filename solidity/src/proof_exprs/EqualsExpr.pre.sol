// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../proof/VerificationBuilder.pre.sol";

/// @title EqualsExpr
/// @dev Library for handling equality comparison expressions between two proof expressions
library EqualsExpr {
    /// @notice Evaluates an equals expression by comparing two sub-expressions
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval
    /// ```
    /// ##### Parameters
    /// * `expr_ptr` - calldata pointer to the expression data
    /// * `builder_ptr` - memory pointer to the verification builder
    /// * `chi_eval` - the chi value for evaluation
    /// ##### Return Values
    /// * `expr_ptr_out` - pointer to the remaining expression after consuming both sub-expressions
    /// * `eval` - the evaluation result from the builder's final round MLE
    /// @notice Evaluates two sub-expressions and produces identity constraints checking their equality
    /// ##### Constraints
    /// We will denote the input columns with \\(L\\) and \\(R\\). The length of the columns are all \\(n\\).
    /// The result of the evaluation is \\(E\\).
    /// We use one hint column, \\(D^\ast\\).
    ///
    /// For ease of notation, we say that \\(D[i] = L[i] - R[i]\\) for all \\(i\\).
    ///
    /// The correct result is
    /// \\[E[i] := \begin{cases}1 & L[i]=R[i] \text{ and }i \in [0,n)\\\\ 0 & \text{else}\end{cases}\\]
    /// One can show that \\(E' \equiv E\\) if and only if there exists some \\(D^\ast\\) such that
    /// \\[\begin{aligned}
    /// E'[i] \cdot D[i] &= 0\\\\
    /// \chi_{[0,n)}[i] - (D[i]\cdot D^\ast[i] + E'[i]) &= 0
    /// \end{aligned}\\]
    /// for all \\(i\\).
    /// @param __expr The equals expression data
    /// @param __builder The verification builder
    /// @param __chiEval The chi value for evaluation
    /// @return __exprOut The remaining expression after processing
    /// @return __builderOut The verification builder result
    /// @return __eval The evaluated result
    function __equalsExprEvaluate( // solhint-disable-line gas-calldata-parameters
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
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_consume_final_round_mle(builder_ptr) -> value {
                revert(0, 0)
            }
            // IMPORT-YUL ../proof/VerificationBuilder.pre.sol
            function builder_produce_identity_constraint(builder_ptr, evaluation, degree) {
                revert(0, 0)
            }
            // IMPORT-YUL ProofExpr.pre.sol
            function proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                revert(0, 0)
            }

            function equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, eval {
                let lhs_eval
                expr_ptr, lhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let rhs_eval
                expr_ptr, rhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let diff_eval := addmod(lhs_eval, mulmod(MODULUS_MINUS_ONE, rhs_eval, MODULUS), MODULUS)
                let diff_star_eval := builder_consume_final_round_mle(builder_ptr)
                eval := mod(builder_consume_final_round_mle(builder_ptr), MODULUS)

                builder_produce_identity_constraint(builder_ptr, mulmod(eval, diff_eval, MODULUS), 2)
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        chi_eval,
                        mulmod(
                            MODULUS_MINUS_ONE,
                            addmod(mulmod(diff_eval, diff_star_eval, MODULUS), eval, MODULUS),
                            MODULUS
                        ),
                        MODULUS
                    ),
                    2
                )

                expr_ptr_out := expr_ptr
            }

            let __exprOutOffset
            __exprOutOffset, __eval := equals_expr_evaluate(__expr.offset, __builder, __chiEval)
            __exprOut.offset := __exprOutOffset
            // slither-disable-next-line write-after-write
            __exprOut.length := sub(__expr.length, sub(__exprOutOffset, __expr.offset))
        }
        __builderOut = __builder;
    }
}
