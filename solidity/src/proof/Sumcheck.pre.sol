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
    /// These coefficients are encoded with the leading coefficient first.
    /// That is the coefficient of x^degree is first, and the constant term is last.
    /// This is to facilitate using Horners method to evaluate the polynomial.
    /// @param transcript0 The initial transcript state
    /// @param proofPtr0 Pointer to the proof data in calldata
    /// @param numVars0 Number of variables in the sumcheck protocol
    /// @param degree0 Degree of the polynomial being checked
    /// @return evaluationPointPtr0 Pointer to the evaluation points in memory
    /// @return expectedEvaluation0 The expected evaluation result
    function verifySumcheckProof( // solhint-disable-line function-max-lines
    uint256[1] memory transcript0, uint256 proofPtr0, uint256 numVars0, uint256 degree0)
        internal
        pure
        returns (uint256 evaluationPointPtr0, uint256 expectedEvaluation0)
    {
        assembly {
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function append_calldata(transcript_ptr, offset, size) {
                revert(0, 0)
            }
            function verify_sumcheck_proof(transcript_ptr, proof_ptr, num_vars, degree) ->
                evaluation_point_ptr,
                expected_evaluation
            {
                mstore(mload(FREE_PTR), mload(transcript_ptr))
                mstore(add(mload(FREE_PTR), 0x20), or(shl(192, degree), shl(128, num_vars)))
                mstore(transcript_ptr, keccak256(mload(FREE_PTR), 0x30))

                expected_evaluation := 0
                evaluation_point_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluation_point_ptr, mul(WORD_SIZE, num_vars)))
                let evaluation_ptr := evaluation_point_ptr
                for {} num_vars { num_vars := sub(num_vars, 1) } {
                    append_calldata(transcript_ptr, proof_ptr, mul(WORD_SIZE, add(degree, 1)))
                    let challenge := and(mload(transcript_ptr), MODULUS_MASK)
                    mstore(evaluation_ptr, challenge)
                    evaluation_ptr := add(evaluation_ptr, WORD_SIZE)
                    let coefficient := calldataload(proof_ptr)
                    proof_ptr := add(proof_ptr, WORD_SIZE)
                    let round_evaluation := coefficient
                    let actual_sum := coefficient
                    for { let d := degree } d { d := sub(d, 1) } {
                        coefficient := calldataload(proof_ptr)
                        proof_ptr := add(proof_ptr, WORD_SIZE)
                        round_evaluation := mulmod(round_evaluation, challenge, MODULUS)
                        round_evaluation := addmod(round_evaluation, coefficient, MODULUS)
                        actual_sum := addmod(actual_sum, coefficient, MODULUS)
                    }
                    actual_sum := addmod(actual_sum, coefficient, MODULUS)
                    if sub(expected_evaluation, actual_sum) { err(ERR_ROUND_EVALUATION_MISMATCH) }
                    expected_evaluation := round_evaluation
                }
            }
            evaluationPointPtr0, expectedEvaluation0 := verify_sumcheck_proof(transcript0, proofPtr0, numVars0, degree0)
        }
    }
}
