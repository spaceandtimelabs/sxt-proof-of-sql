// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "./Constants.sol";

/// @title Lagrange Basis Evaluation Library
/// @notice A library for efficiently computing sums over Lagrange basis polynomials evaluated at points.
library LagrangeBasisEvaluation {
    /// @notice Computes the sum of the Lagrange basis polynomials evaluated at a given point.
    /// @notice This is a wrapper around the `compute_truncated_lagrange_basis_sum` Yul function.
    /// This wrapper is only intended to be used for testing.
    /// @param __length The length of the sum.
    /// @param __x The point at which to evaluate the Lagrange basis.
    /// @return __result The sum of the Lagrange basis polynomials evaluated at the given point.
    /// @dev Let \\(\chi_i(x)\\) be the \\(i\\)th Lagrange basis polynomial.
    /// That is, \\[\chi_i(x) = \prod_{j=0}^{\infty} (1-x_j)^{1-b_j}x_j^{b_j},\\]
    /// where \\(b_j\\) is the \\(j\\)th bit of \\(i\\).
    /// @dev This function computes \\[ \sum_{i=0}^{\ell-1}\chi_i(x_0,\ldots,x_{\nu-1},0,\ldots),\\]
    /// where \\(\ell = \texttt{length}\\) and \\(\nu = \texttt{num_vars} = \texttt{__x.length}\\).
    /// @dev The naive formula is \\(O(\ell \cdot \nu)\\). It is be computed in \\(O(\ell)\\) time using the following
    ///  formulation:
    /// Let \\(X_\nu=(x_0,\ldots,x_{\nu-1})\\) and let
    /// \\[f(\ell, X_\nu, \nu)=\sum_{i=0}^{\ell-1}\chi_i(x_0,\ldots,x_{\nu-1},0,\ldots)\\]
    /// Then, for \\(\ell<2^\nu\\), we have
    /// \\[\begin{aligned}f(\ell, X_{\nu+1}, \nu+1) &=\sum_{i=0}^{\ell-1}\chi_i(x_0,\ldots,x_{\nu},0,\ldots)\\\\
    /// &=\sum_{i=0}^{\ell-1}(1-x_\nu)\cdot\chi_i(x_0,\ldots,x_{\nu-1},0,\ldots)\\\\
    ///  &= (1-x_\nu)\cdot f(\ell, X_\nu, \nu)\\\\
    ///  f(\ell+2^\nu, X_{\nu+1}, \nu+1) &=\sum_{i=0}^{2^\nu-1}\chi_{i}(x_0,\ldots,x_{\nu},0,\ldots)+\sum_{i=0}^{\ell-1}\chi_{i+2^\nu}(x_0,\ldots,x_{\nu},0,\ldots)\\\\
    /// &=\sum_{i=0}^{2^\nu-1}(1-x_\nu)\cdot\chi_{i}(x_0,\ldots,x_{\nu-1},0,\ldots)+\sum_{i=0}^{\ell-1}x_\nu\cdot\chi_{i}(x_0,\ldots,x_{\nu-1},0,\ldots)\\\\
    ///  &= (1-x_\nu)+x_\nu\cdot f(\ell, X_\nu, \nu)
    /// \end{aligned}\\]
    /// For \\(\ell \geq 2^{\nu}\\), we have that \\(f(\ell,X_\nu,\nu)=1\\).
    function __computeTruncatedLagrangeBasisSum(uint256 __length, uint256[] memory __x)
        internal
        pure
        returns (uint256 __result)
    {
        assembly {
            function compute_truncated_lagrange_basis_sum(length, x_ptr, num_vars) -> result {
                result := 0

                // Invariant that holds within the for loop:
                // 0 <= result <= modulus + 1
                // This invariant reduces modulus operations.
                for {} num_vars {} {
                    switch and(length, 1)
                    case 0 { result := mulmod(result, sub(MODULUS_PLUS_ONE, mod(mload(x_ptr), MODULUS)), MODULUS) }
                    default {
                        result := sub(MODULUS_PLUS_ONE, mulmod(sub(MODULUS_PLUS_ONE, result), mload(x_ptr), MODULUS))
                    }
                    num_vars := sub(num_vars, 1)
                    length := shr(1, length)
                    x_ptr := add(x_ptr, WORD_SIZE)
                }
                switch length
                case 0 { result := mod(result, MODULUS) }
                default { result := 1 }
            }
            __result := compute_truncated_lagrange_basis_sum(__length, add(__x, WORD_SIZE), mload(__x))
        }
    }

    /// @notice Computes the Lagrange basis vector for a given evaluation point.
    /// @notice This is a wrapper around the `compute_lagrange_basis_vector` Yul function.
    /// @param __length The length of the vector.
    /// @param __evaluationPoint The evaluation point at which to compute the Lagrange basis vector.
    /// @return __evaluations The computed Lagrange basis vector.
    /// @dev This function computes the Lagrange basis vector for the given evaluation point.
    function __computeEvaluationVec(uint256 __length, uint256[] memory __evaluationPoint)
        internal
        pure
        returns (uint256[] memory __evaluations)
    {
        assembly {
            function compute_evaluation_vec(length, evaluation_point_ptr) -> evaluations_ptr {
                evaluations_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluations_ptr, mul(length, WORD_SIZE)))
                mstore(evaluations_ptr, 1)
                let num_vars := mload(evaluation_point_ptr)
                for { let len := 1 } num_vars { num_vars := sub(num_vars, 1) } {
                    let x := mod(mload(add(evaluation_point_ptr, mul(num_vars, WORD_SIZE))), MODULUS)
                    let one_minus_x := sub(MODULUS_PLUS_ONE, x)
                    len := mul(len, 2)
                    if gt(len, length) { len := length }
                    for { let l := len } l {} {
                        l := sub(l, 1)
                        let to_ptr := add(evaluations_ptr, mul(l, WORD_SIZE))
                        let from_ptr := add(evaluations_ptr, mul(shr(1, l), WORD_SIZE))
                        switch mod(l, 2)
                        case 0 { mstore(to_ptr, mulmod(mload(from_ptr), one_minus_x, MODULUS)) }
                        case 1 { mstore(to_ptr, mulmod(mload(from_ptr), x, MODULUS)) }
                    }
                }
            }
            mstore(FREE_PTR, add(mload(FREE_PTR), WORD_SIZE))
            __evaluations := compute_evaluation_vec(__length, __evaluationPoint)
            __evaluations := sub(__evaluations, WORD_SIZE)
            mstore(__evaluations, __length)
        }
    }

    /// @notice Computes evaluations of Lagrange basis polynomials for a given evaluation point and array.
    /// @notice This is a wrapper around the `compute_evaluations` Yul function. Note that the function
    /// does not return the evaluations, but rather modifies the input array in place.
    /// @param __evaluationPoint The evaluation point at which to compute the evaluations.
    /// @param __array The array of lengths to evaluate.
    /// @dev This could likely be batched more efficiently. For now, we just naively compute the evaluations for each length.
    function __computeEvaluations(uint256[] memory __evaluationPoint, uint256[] memory __array) internal pure {
        assembly {
            // IMPORT-YUL LagrangeBasisEvaluation.pre.sol
            function compute_truncated_lagrange_basis_sum(length, x_ptr, num_vars) -> result {
                revert(0, 0)
            }
            function compute_evaluations(evaluation_point_ptr, array_ptr) {
                let num_vars := mload(evaluation_point_ptr)
                let x := add(evaluation_point_ptr, WORD_SIZE)
                let array_len := mload(array_ptr)
                array_ptr := add(array_ptr, WORD_SIZE)
                for {} array_len { array_len := sub(array_len, 1) } {
                    mstore(array_ptr, compute_truncated_lagrange_basis_sum(mload(array_ptr), x, num_vars))
                    array_ptr := add(array_ptr, WORD_SIZE)
                }
            }
            compute_evaluations(__evaluationPoint, __array)
        }
    }

    /// @notice Computes the inner product of the Lagrange basis polynomials evaluated at two given points.
    /// @notice Reverts if `__x` and `__y` have different lengths.
    /// @notice This is a wrapper around the `compute_truncated_lagrange_basis_inner_product` Yul function.
    /// This wrapper is only intended to be used for testing.
    /// @param __length The length of the sum.
    /// @param __x The first point at which to evaluate the Lagrange basis.
    /// @param __y The second point at which to evaluate the Lagrange basis.
    /// @return __result The inner product of the Lagrange basis polynomials evaluated at the two points.
    /// @dev Let \\(\chi_i(x)\\) be the \\(i\\)th Lagrange basis polynomial as described in
    /// [__computeTruncatedLagrangeBasisSum](#__computetruncatedlagrangebasissum).
    /// @dev This function computes
    /// \\[ \sum_{i=0}^{\ell-1}\chi_i(x_0,\ldots,x_{\nu-1},0,\ldots)\chi_i(y_0,\ldots,y_{\nu-1},0,\ldots),\\]
    /// where \\(\ell = \texttt{length}\\) and
    /// \\(\nu = \texttt{num_vars} = \texttt{__x.length} = \texttt{__y.length}\\).
    /// @dev The naive formula is \\(O(\ell \cdot \nu)\\). It is be computed in \\(O(\ell)\\) time using the following
    ///  formulation:
    /// NOTE: this is the generalization of the `compute_truncated_lagrange_basis_sum` formulas, with $y_i=0$.
    /// Let \\(X_\nu=(x_0,\ldots,x_{\nu-1})\\), \\(Y_\nu=(y_0,\ldots,y_{\nu-1})\\), and let
    /// \\[\begin{aligned}
    /// g(\ell, X_\nu, Y_\nu, \nu)&=\sum_{i=0}^{\ell-1}\chi_i(x_0,\ldots,x_{\nu-1},0,\ldots)\chi_i(y_0,\ldots,y_{\nu-1},0,\ldots)\\\\
    /// h(X_\nu, Y_\nu, \nu)&=g(2^\nu,X_\nu, Y_\nu, \nu)
    /// \end{aligned}\\]
    /// Then, for \\(\ell< 2^\nu\\), we have
    /// \\[\begin{aligned}
    /// g(\ell, X_{\nu+1},Y_{\nu+1},\nu+1)&=(1-x_\nu)\cdot(1-y_\nu)\cdot g(\ell, X_\nu,Y_\nu,\nu)\\\\
    /// g(\ell+2^\nu, X_{\nu+1},Y_{\nu+1},\nu+1)&=(1-x_\nu)\cdot(1-y_\nu)\cdot h(X_\nu, Y_\nu, \nu)+x_\nu\cdot y_\nu\cdot g(\ell, X_\nu,Y_\nu,\nu)\\\\
    /// h(X_{\nu+1}, Y_{\nu+1}, \nu+1)&=((1-x_\nu)\cdot(1-y_\nu)+x_\nu\cdot y_\nu)\cdot h(X_\nu, Y_\nu, \nu)
    /// \end{aligned}\\]
    /// For \\(\ell \geq 2^{\nu}\\), we have that \\(g(\ell,X_\nu,Y_\nu,\nu)=h(X_\nu,Y_\nu,\nu)\\).
    function __computeTruncatedLagrangeBasisInnerProduct(uint256 __length, uint256[] memory __x, uint256[] memory __y)
        internal
        pure
        returns (uint256 __result)
    {
        assert(__x.length == __y.length);
        assembly {
            function compute_truncated_lagrange_basis_inner_product(length, x_ptr, y_ptr, num_vars) -> result {
                let part := 0 // This is g in the formulas
                result := 1 // This is h in the formulas
                for {} num_vars {} {
                    let x := mload(x_ptr)
                    let y := mload(y_ptr)
                    let xy := mulmod(x, y, MODULUS)
                    // let c := 1 - x
                    // let d := 1 - y
                    let cd := sub(add(MODULUS_PLUS_ONE, xy), addmod(x, y, MODULUS))
                    switch and(length, 1)
                    case 0 { part := mulmod(part, cd, MODULUS) }
                    default { part := add(mulmod(result, cd, MODULUS), mulmod(part, xy, MODULUS)) }
                    result := mulmod(result, add(cd, xy), MODULUS)
                    num_vars := sub(num_vars, 1)
                    length := shr(1, length)
                    x_ptr := add(x_ptr, WORD_SIZE)
                    y_ptr := add(y_ptr, WORD_SIZE)
                }
                if iszero(length) { result := mod(part, MODULUS) } // we return g in "short" cases
            }
            __result :=
                compute_truncated_lagrange_basis_inner_product(
                    __length, add(__x, WORD_SIZE), add(__y, WORD_SIZE), mload(__x)
                )
        }
    }
}
