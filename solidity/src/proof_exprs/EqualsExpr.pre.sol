// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";
import {VerificationBuilder} from "../builder/VerificationBuilder.pre.sol";

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
    /// @notice ##### Constraints
    /// * Inputs: \\(L=\texttt{lhs}\\), \\(R=\texttt{rhs}\\) with length \\(n\\), and thus \\(\chi_{[0,n)}=\texttt{chi}\\).
    ///   Note: `proof_expr_evaluate` guarentees that the lengths of \\(L\\) and \\(R\\) are equal the lengths of `chi_{[0,n)}`.
    /// * Outputs: \\(E=\texttt{result}\\) with length \\(n\\)
    /// * Hints: \\(D^\star=\texttt{diff_star}\\)
    /// * Helpers: \\(D :\equiv L - R=\texttt{diff}\\)
    /// * Constraints:
    /// \\[\begin{aligned}
    /// E \cdot D &\equiv 0\\\\
    /// \chi_{[0,n)} - (D\cdot D^\star + E) &\equiv 0
    /// \end{aligned}\\]
    /// @notice ##### Proof of Correctness
    /// @notice **Theorem:** Given columns \\(L\\) and \\(R\\) of length \\(n\\),
    /// \\[E[i] = \begin{cases} 1 & L[i] = R[i] \text{ and } i < n\\\\ 0 & \text{else}\end{cases}\\]
    /// if and only if there exits a \\(D^\star\\) such that
    /// \\[\begin{aligned}
    /// E[i] \cdot D[i] &= 0\\\\
    /// \chi_{[0,n)}[i] - (D[i]\cdot D^\star[i] + E[i]) &= 0
    /// \end{aligned}\\]
    /// for all \\(i\\), where \\(D[i] = L[i] - R[i]\\).
    /// @notice **Completeness Proof:**
    /// Setting \\[D^\star[i] = \begin{cases} (D[i])^{-1} & D[i] \neq 0\\\ 0 & \text{else}\end{cases}\\] satisfies the above equations.
    /// @notice **Soundness Proof:**
    /// * If \\(i<n\\) and \\(L[i] \neq R[i]\\), then \\(D[i] \neq 0\\) and \\(E[i] = 0\\) by the first equation.
    /// * If \\(i<n\\) and \\(L[i] = R[i]\\), then \\(D[i] = 0\\) and \\(E[i] = \chi_{[0,n)}[i] = 1\\) by the second equation.
    /// * If \\(i \geq n\\), then \\(L[i] = 0 = R[i]\\) and \\(E[i] = \chi_{[0,n)}[i] = 0\\) by the second equation.
    /// ##### Proof Plan Encoding
    /// The equals expression is encoded as follows:
    /// 1. The left hand side expression
    /// 2. The right hand side expression
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

            function equals_expr_evaluate(expr_ptr, builder_ptr, chi_eval) -> expr_ptr_out, result_eval {
                let lhs_eval
                expr_ptr, lhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let rhs_eval
                expr_ptr, rhs_eval := proof_expr_evaluate(expr_ptr, builder_ptr, chi_eval)

                let diff_eval := addmod(lhs_eval, mulmod(MODULUS_MINUS_ONE, rhs_eval, MODULUS), MODULUS)
                let diff_star_eval := builder_consume_final_round_mle(builder_ptr)
                result_eval := mod(builder_consume_final_round_mle(builder_ptr), MODULUS)

                builder_produce_identity_constraint(builder_ptr, mulmod(result_eval, diff_eval, MODULUS), 2)
                builder_produce_identity_constraint(
                    builder_ptr,
                    addmod(
                        chi_eval,
                        mulmod(
                            MODULUS_MINUS_ONE,
                            addmod(mulmod(diff_eval, diff_star_eval, MODULUS), result_eval, MODULUS),
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
