// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

// assembly only constants
// solhint-disable-next-line no-unused-import
import {MODULUS, MODULUS_PLUS_ONE, WORD_SIZE} from "./Constants.sol";

/// @title Lagrange Basis Evaluation Library
/// @notice A library for efficiently computing sums over Lagrange basis polynomials evaluated at points.
library LagrangeBasisEvaluation {
    /// @notice Computes the sum of the Lagrange basis polynomials evaluated at a given point.
    /// @dev Given the point `point` (or `a`) with length nu, we can evaluate the lagrange basis of length 2^nu at that point.
    /// This is what `compute_evaluation_vector` in the rust code does.
    ///
    /// NOTE: if length is greater than 2^nu, this function will pad `point` with 0s, which
    /// will result in padding the basis with 0s.
    ///
    /// Call the resulting evaluation vector A. This function computes `sum A[i] for i in 0..length`. That is:
    /// ```text
    /// (1-a[0])(1-a[1])...(1-a[nu-1]) +
    /// (a[0])(1-a[1])...(1-a[nu-1]) +
    /// (1-a[0])(a[1])...(1-a[nu-1]) +
    /// (a[0])(a[1])...(1-a[nu-1]) + ...
    /// ```
    /// @param length0 The length of the evaluation vector.
    /// @param point0 The point at which to evaluate the Lagrange basis.
    /// @return result0 The sum of the Lagrange basis polynomials evaluated at the given point.
    function computeTruncatedLagrangeBasisSum(uint256 length0, uint256[] memory point0)
        internal
        pure
        returns (uint256 result0)
    {
        assembly {
            function compute_truncated_lagrange_basis_sum(length, point_ptr, num_vars) -> result {
                result := 0

                // Invariant that holds within the for loop:
                // 0 <= result <= modulus + 1
                // This invariant reduces modulus operations.
                for {} num_vars {} {
                    switch and(length, 1)
                    case 0 { result := mulmod(result, sub(MODULUS_PLUS_ONE, mod(mload(point_ptr), MODULUS)), MODULUS) }
                    default {
                        result :=
                            sub(MODULUS_PLUS_ONE, mulmod(sub(MODULUS_PLUS_ONE, result), mload(point_ptr), MODULUS))
                    }
                    num_vars := sub(num_vars, 1)
                    length := shr(1, length)
                    point_ptr := add(point_ptr, WORD_SIZE)
                }
                switch length
                case 0 { result := mod(result, MODULUS) }
                default { result := 1 }
            }
            result0 := compute_truncated_lagrange_basis_sum(length0, add(point0, WORD_SIZE), mload(point0))
        }
    }

    /// @notice Computes the inner product of the Lagrange basis polynomials evaluated at two given points.
    /// @dev Given the points a and b with length nu, we can evaluate the lagrange basis of length 2^nu at the two points.
    /// This is what `compute_evaluation_vector` in the rust code does.
    /// Call the resulting evaluation vectors A and B. This function computes `sum A[i] * B[i] for i in 0..length`. That is:
    /// ```text
    /// (1-a[0])(1-a[1])...(1-a[nu-1]) * (1-b[0])(1-b[1])...(1-b[nu-1]) +
    /// (a[0])(1-a[1])...(1-a[nu-1]) * (b[0])(1-b[1])...(1-b[nu-1]) +
    /// (1-a[0])(a[1])...(1-a[nu-1]) * (1-b[0])(b[1])...(1-b[nu-1]) +
    /// (a[0])(a[1])...(1-a[nu-1]) * (b[0])(b[1])...(1-b[nu-1]) + ...
    /// ```
    /// @param length0 The length of the evaluation vectors.
    /// @param a0 The first point at which to evaluate the Lagrange basis.
    /// @param b0 The second point at which to evaluate the Lagrange basis.
    /// @return result0 The inner product of the Lagrange basis polynomials evaluated at the two given points.
    function computeTruncatedLagrangeBasisInnerProduct(uint256 length0, uint256[] memory a0, uint256[] memory b0)
        internal
        pure
        returns (uint256 result0)
    {
        assert(a0.length == b0.length);
        assembly {
            function compute_truncated_lagrange_basis_inner_product(length, a_ptr, b_ptr, num_vars) -> result {
                let part := 0
                result := 1
                for { let i := 0 } sub(num_vars, i) { i := add(i, 1) } {
                    let a := mload(a_ptr)
                    let b := mload(b_ptr)
                    a_ptr := add(a_ptr, WORD_SIZE)
                    b_ptr := add(b_ptr, WORD_SIZE)
                    let ab := mulmod(a, b, MODULUS)
                    let cd := sub(add(MODULUS_PLUS_ONE, ab), addmod(a, b, MODULUS))
                    switch and(shr(i, length), 1)
                    case 0 { part := mulmod(part, cd, MODULUS) }
                    default { part := add(mulmod(result, cd, MODULUS), mulmod(part, ab, MODULUS)) }
                    result := mulmod(result, add(cd, ab), MODULUS)
                }
                if lt(length, shl(num_vars, 1)) { result := mod(part, MODULUS) }
            }
            result0 :=
                compute_truncated_lagrange_basis_inner_product(length0, add(a0, WORD_SIZE), add(b0, WORD_SIZE), mload(a0))
        }
    }
}
