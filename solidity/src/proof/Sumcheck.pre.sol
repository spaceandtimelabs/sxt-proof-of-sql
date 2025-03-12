// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title Sumcheck Protocol Verification Library
/// @notice This library provides functions to verify sumcheck proofs in zero-knowledge protocols.
library Sumcheck {
    /// @notice Verifies a sumcheck proof
    /// @dev NOTE: there are num_vars messages in the proof, which are polynomials of degree `degree`.
    /// THe proof begins with a 64-bit integer indicating the length of the proof.
    /// The proof is then divided into num_vars sections, each of which contains degree + 1 coefficients.
    /// These coefficients are encoded with the leading coefficient first.
    /// That is the coefficient of x^degree is first, and the constant term is last.
    /// This is to facilitate using Horners method to evaluate the polynomial.
    /// @dev WARNING: the `num_vars` value is public input but not added to the transcript by this function,
    /// so it must be added to the transcript before calling this function
    /// @dev WARNING #2: the degree of the prover messages is dictated by the prover, and is returned by this function
    /// It must be validated by the caller to ensure it is acceptable. More concretely, the verifier must ensure that
    /// the degree is at least as large as the degree of the largest constraint.
    function __verifySumcheckProof( // solhint-disable-line gas-calldata-parameters
    uint256[1] memory __transcript, bytes calldata __proof, uint256 __numVars)
        external
        pure
        returns (
            bytes calldata __proofOut,
            uint256[] memory __evaluationPoint,
            uint256 __expectedEvaluation,
            uint256 __degree
        )
    {
        __evaluationPoint = new uint256[](__numVars);
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function append_calldata(transcript_ptr, offset, size) {
                revert(0, 0)
            }

            // actual_sum = coefficient_i + sum_{i=0}^{degree} coefficient_i,
            //                  where coefficient_i is the coefficient of x^i
            // round_evaluation = sum_{i=0}^{degree} coefficient_i * challenge^i
            // NOTE: the coefficients are in "reverse" order, with the leading coefficient first
            //       as a result, round_evaluation is computed with Horner's method
            function process_round(proof_ptr, degree, challenge) -> proof_ptr_out, round_evaluation, actual_sum {
                let coefficient := calldataload(proof_ptr)
                proof_ptr := add(proof_ptr, WORD_SIZE)
                round_evaluation := coefficient
                actual_sum := coefficient
                for {} degree { degree := sub(degree, 1) } {
                    coefficient := calldataload(proof_ptr)
                    proof_ptr := add(proof_ptr, WORD_SIZE)
                    round_evaluation := mulmod(round_evaluation, challenge, MODULUS)
                    round_evaluation := addmod(round_evaluation, coefficient, MODULUS)
                    actual_sum := addmod(actual_sum, coefficient, MODULUS)
                }
                actual_sum := addmod(actual_sum, coefficient, MODULUS)
                proof_ptr_out := proof_ptr
            }

            function verify_sumcheck_proof(transcript_ptr, proof_ptr, num_vars) ->
                proof_ptr_out,
                evaluation_point_ptr,
                expected_evaluation,
                degree
            {
                append_calldata(transcript_ptr, proof_ptr, UINT64_SIZE)
                let sumcheck_length := shr(UINT64_PADDING_BITS, calldataload(proof_ptr))
                proof_ptr := add(proof_ptr, UINT64_SIZE)
                if or(or(iszero(num_vars), iszero(sumcheck_length)), mod(sumcheck_length, num_vars)) {
                    err(ERR_INVALID_SUMCHECK_PROOF_SIZE)
                }
                degree := sub(div(sumcheck_length, num_vars), 1)

                expected_evaluation := 0
                evaluation_point_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluation_point_ptr, mul(WORD_SIZE, num_vars)))
                let evaluation_ptr := evaluation_point_ptr
                for {} num_vars { num_vars := sub(num_vars, 1) } {
                    append_calldata(transcript_ptr, proof_ptr, mul(WORD_SIZE, add(degree, 1)))
                    let challenge := and(mload(transcript_ptr), MODULUS_MASK)
                    mstore(evaluation_ptr, challenge)
                    evaluation_ptr := add(evaluation_ptr, WORD_SIZE)
                    let round_evaluation, actual_sum
                    proof_ptr, round_evaluation, actual_sum := process_round(proof_ptr, degree, challenge)
                    if sub(expected_evaluation, actual_sum) { err(ERR_ROUND_EVALUATION_MISMATCH) }
                    expected_evaluation := round_evaluation
                }
                proof_ptr_out := proof_ptr
            }
            let __proofOutOffset
            let __evaluationPointDataPtr
            __proofOutOffset, __evaluationPointDataPtr, __expectedEvaluation, __degree :=
                verify_sumcheck_proof(__transcript, __proof.offset, __numVars)
            __proofOut.offset := __proofOutOffset
            // slither-disable-next-line write-after-write
            __proofOut.length := sub(__proof.length, sub(__proofOutOffset, __proof.offset))
            mcopy(add(__evaluationPoint, WORD_SIZE), __evaluationPointDataPtr, mul(WORD_SIZE, __numVars))
        }
    }
}
