// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol";
import "../base/Errors.sol";

/// @title HyperKZG Batch Processing Library
/// @notice A library for batch processing polynomial commitment schemes (PCS) using HyperKZG.
library HyperKZGBatch {
    /// @notice Processes a batch of polynomial commitment schemes (PCS).
    /// @notice This is a wrapper around the `batch_pcs` Yul function.
    /// This wrapper is only intended to be used for testing.
    /// @param __args Memory pointer to the arguments for the batch PCS.
    /// @param __transcript Memory pointer to the transcript.
    /// @param __commitments Array of commitments.
    /// @param __evaluations Array of evaluations.
    /// @param __batchEval Initial batch evaluation value.
    /// @return __batchEvalOut The final batch evaluation value.
    function __batchPCS(
        uint256[5] memory __args,
        uint256[1] memory __transcript,
        uint256[] memory __commitments,
        uint256[] memory __evaluations,
        uint256 __batchEval
    ) internal view returns (uint256 __batchEvalOut, uint256[5] memory __argsOut) {
        assembly {
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_add(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_mul(args_ptr) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function ec_mul_assign(args_ptr, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/ECPrecompiles.pre.sol
            function constant_ec_mul_add_assign(args_ptr, c_x, c_y, scalar) {
                pop(staticcall(0, 0, 0, 0, 0, 0))
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Errors.sol
            function err(code) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenge(transcript_ptr) -> result {
                revert(0, 0)
            }

            function batch_pcs(args_ptr, transcript_ptr, commitments_ptr, evaluations_ptr, batch_eval) -> batch_eval_out
            {
                let num_commitments := mload(commitments_ptr)
                commitments_ptr := add(commitments_ptr, WORD_SIZE)
                let num_evaluations := mload(evaluations_ptr)
                evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                if sub(num_commitments, num_evaluations) { err(ERR_PCS_BATCH_LENGTH_MISMATCH) }
                for {} num_commitments { num_commitments := sub(num_commitments, 1) } {
                    let challenge := draw_challenge(transcript_ptr)
                    constant_ec_mul_add_assign(
                        args_ptr, mload(commitments_ptr), mload(add(commitments_ptr, WORD_SIZE)), challenge
                    )
                    commitments_ptr := add(commitments_ptr, WORDX2_SIZE)
                    batch_eval := addmod(batch_eval, mulmod(mload(evaluations_ptr), challenge, MODULUS), MODULUS)
                    evaluations_ptr := add(evaluations_ptr, WORD_SIZE)
                }
                batch_eval_out := mod(batch_eval, MODULUS)
            }

            // divide by 2 since the commitments are (x, y) pairs
            mstore(__commitments, div(mload(__commitments), 2))
            __batchEvalOut := batch_pcs(__args, __transcript, __commitments, __evaluations, __batchEval)
        }
        __argsOut = __args;
    }
}
