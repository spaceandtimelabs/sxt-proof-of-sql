// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

// assembly only constants
/* solhint-disable no-unused-import */
import {
    FREE_PTR,
    MODULUS,
    MODULUS_MINUS_ONE,
    MODULUS_MASK,
    WORD_SIZE,
    WORDX2_SIZE,
    WORDX3_SIZE,
    WORDX6_SIZE,
    HYPER_KZG_INCONSISTENT_V
} from "../base/Constants.sol";
/* solhint-enable no-unused-import */

/// @title HyperKZGHelpers
/// @dev Library providing helper functions for the HyperKZG polynomial commitment verifier.
library HyperKZGHelpers {
    /// @notice Runs the Fiat-Shamir transcript to generate challenges for the HyperKZG proof
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// run_transcript(com_ptr, v_ptr, w_ptr, transcript_ptr, ell) -> r, q, d
    /// ```
    /// ##### Parameters
    /// * `com_ptr` - the calldata pointer to the beginning of the data in `__com`
    /// * `v_ptr` - the calldata pointer to the beginning of the data in `__v`
    /// * `w_ptr` - the calldata pointer to the beginning of the data in `__w`
    /// * `transcript_ptr` - the memory pointer to the transcript word
    /// @dev Processes prover messages to generate random challenges
    /// using a Fiat-Shamir transformation
    /// @param __com The first prover message
    /// @param __v The second prover message
    /// @param __w The third prover message
    /// @param __transcript Initial transcript value
    /// @param __ell The size parameter for the proof
    /// @return __r First challenge (r value)
    /// @return __q Second challenge (q value)
    /// @return __d Third challenge (d value)
    function __runTranscript( // solhint-disable-line gas-calldata-parameters
    bytes calldata __com, bytes calldata __v, bytes calldata __w, uint256[1] memory __transcript, uint256 __ell)
        external
        pure
        returns (uint256 __r, uint256 __q, uint256 __d)
    {
        assembly {
            // IMPORT-YUL ../base/Transcript.sol
            function append_calldata(transcript_ptr, offset, size) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenge(transcript_ptr) -> result {
                revert(0, 0)
            }
            function run_transcript(com_ptr, v_ptr, w_ptr, transcript_ptr, ell) -> r, q, d {
                append_calldata(transcript_ptr, com_ptr, mul(WORDX2_SIZE, sub(ell, 1)))
                r := draw_challenge(transcript_ptr)

                append_calldata(transcript_ptr, v_ptr, mul(WORDX3_SIZE, ell))
                q := draw_challenge(transcript_ptr)

                append_calldata(transcript_ptr, w_ptr, WORDX6_SIZE)
                d := draw_challenge(transcript_ptr)
            }
            __r, __q, __d := run_transcript(__com.offset, __v.offset, __w.offset, __transcript, __ell)
        }
    }

    /// @notice Calculate a bivariate polynomial evaluation for a given set of coefficients
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// bivariate_evaluation(v_ptr, q, d, ell) -> b
    /// ```
    /// ##### Parameters
    /// * `v_ptr` - the calldata pointer to the beginning of the data in `__v`
    /// @dev This function computes \\[\sum_{i=0}^{\ell-1} \sum_{j=0}^2 v_{i,j} \cdot d^jq^i. \\]
    /// @dev The function is implemented using Horner's method in 2 dimensions,
    /// so it only requires \\( 3\ell - 1 \\) multiplications and additions.
    /// We do it in \\( 3\ell \\) for simplicity.
    /// @param __v Array of coefficient triplets
    /// @param __q First evaluation point
    /// @param __d Second evaluation point
    /// @return __b The evaluated polynomial value
    function __bivariateEvaluation(uint256[3][] calldata __v, uint256 __q, uint256 __d)
        external
        pure
        returns (uint256 __b)
    {
        assembly {
            function bivariate_evaluation(v_ptr, q, d, ell) -> b {
                let v_stack := add(v_ptr, mul(WORDX3_SIZE, ell))
                for {} ell { ell := sub(ell, 1) } {
                    // tmp = v2i
                    v_stack := sub(v_stack, WORD_SIZE)
                    let tmp := calldataload(v_stack)
                    // tmp = v2i * d
                    tmp := mulmod(tmp, d, MODULUS)
                    // tmp += v1i
                    v_stack := sub(v_stack, WORD_SIZE)
                    tmp := addmod(tmp, calldataload(v_stack), MODULUS)
                    // tmp *= d
                    tmp := mulmod(tmp, d, MODULUS)
                    // tmp += v0i
                    v_stack := sub(v_stack, WORD_SIZE)
                    tmp := addmod(tmp, calldataload(v_stack), MODULUS)

                    // b *= q
                    b := mulmod(b, q, MODULUS)
                    // b += tmp
                    b := addmod(b, tmp, MODULUS)
                }
            }
            __b := bivariate_evaluation(__v.offset, __q, __d, __v.length)
        }
    }

    /// @notice Check that the v array is consistent with the given r, x, and y values
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// check_v_consistency(v_ptr, r, x, y)
    /// ```
    /// ##### Parameters
    /// * `v_ptr` - the calldata pointer to the beginning of the data in `__v`
    /// * `x` - the memory pointer to `__x`. Note: this includes the length of the array as the first word
    /// @dev This function checks that the following equation holds for all \\( i \in [0, \ell) \\):
    /// \\[ r \cdot (2v_{i+1,2} + (x_i - 1) \cdot (v_{i,1} + v_{i,0})) + x_i \cdot (v_{i,1} - v_{i,0}) = 0 \\]
    /// where \\( v_{\ell,i} = y \\).
    /// @param __v Array being checked for consistency
    /// @param __r Challenge value r
    /// @param __x Array of x coordinates
    /// @param __y y value
    function __checkVConsistency( // solhint-disable-line gas-calldata-parameters
    uint256[3][] calldata __v, uint256 __r, uint256[] memory __x, uint256 __y)
        external
        pure
    {
        assert(__x.length == __v.length);
        assembly {
            function check_v_consistency(v_ptr, r, x, y) {
                let ell := mload(x)
                let v_stack := add(v_ptr, mul(WORDX3_SIZE, ell))
                x := add(x, mul(WORD_SIZE, add(ell, 1)))
                let last_v2 := y
                for {} ell { ell := sub(ell, 1) } {
                    v_stack := sub(v_stack, WORD_SIZE)
                    let v2i := calldataload(v_stack)
                    v_stack := sub(v_stack, WORD_SIZE)
                    let v1i := calldataload(v_stack)
                    v_stack := sub(v_stack, WORD_SIZE)
                    let v0i := calldataload(v_stack)
                    x := sub(x, WORD_SIZE)
                    let xi := mload(x)

                    // r * (2 * y + (xi - 1) * (v1i + v0i)) + xi * (v1i - v0i)
                    if addmod(
                        mulmod(
                            r,
                            addmod(
                                addmod(last_v2, last_v2, MODULUS),
                                mulmod(addmod(xi, MODULUS_MINUS_ONE, MODULUS), addmod(v1i, v0i, MODULUS), MODULUS),
                                MODULUS
                            ),
                            MODULUS
                        ),
                        mulmod(xi, addmod(v1i, sub(MODULUS, mod(v0i, MODULUS)), MODULUS), MODULUS),
                        MODULUS
                    ) {
                        mstore(0, HYPER_KZG_INCONSISTENT_V)
                        revert(0, 4)
                    }

                    last_v2 := v2i
                }
            }
            check_v_consistency(__v.offset, __r, __x, __y)
        }
    }
}
