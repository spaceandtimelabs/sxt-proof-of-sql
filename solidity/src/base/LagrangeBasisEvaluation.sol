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
    /// That is, \\[\chi_i(x) = \prod_{j=0}^{\nu-1} (1-x_j)^{1-b_j}x_j^{b_j},\\]
    /// where \\(b_j\\) is the \\(j\\)th bit of \\(i\\).
    /// @dev This function computes \\[ \sum_{i=0}^{\ell-1}\chi_i(x_0,\ldots,x_{\nu-1},0,\ldots),\\]
    /// where \\(\ell = \texttt{length}\\) and \\(\nu = \texttt{num_vars} = \texttt{__x.length}\\).
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
    function __computeTruncatedLagrangeBasisInnerProduct(uint256 __length, uint256[] memory __x, uint256[] memory __y)
        internal
        pure
        returns (uint256 __result)
    {
        assert(__x.length == __y.length);
        assembly {
            function compute_truncated_lagrange_basis_inner_product(length, x_ptr, y_ptr, num_vars) -> result {
                let part := 0
                result := 1
                for { let i := 0 } sub(num_vars, i) { i := add(i, 1) } {
                    let x := mload(x_ptr)
                    let y := mload(y_ptr)
                    x_ptr := add(x_ptr, WORD_SIZE)
                    y_ptr := add(y_ptr, WORD_SIZE)
                    let xy := mulmod(x, y, MODULUS)
                    let cd := sub(add(MODULUS_PLUS_ONE, xy), addmod(x, y, MODULUS))
                    switch and(shr(i, length), 1)
                    case 0 { part := mulmod(part, cd, MODULUS) }
                    default { part := add(mulmod(result, cd, MODULUS), mulmod(part, xy, MODULUS)) }
                    result := mulmod(result, add(cd, xy), MODULUS)
                }
                if lt(length, shl(num_vars, 1)) { result := mod(part, MODULUS) }
            }
            __result :=
                compute_truncated_lagrange_basis_inner_product(
                    __length, add(__x, WORD_SIZE), add(__y, WORD_SIZE), mload(__x)
                )
        }
    }
}
