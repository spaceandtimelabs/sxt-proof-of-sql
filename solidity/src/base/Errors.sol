// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

/// @dev Error code for when ECADD inputs are invalid.
uint32 constant ERR_INVALID_EC_ADD_INPUTS = 0x765bcba0;
/// @dev Error code for when ECMUL inputs are invalid.
uint32 constant ERR_INVALID_EC_MUL_INPUTS = 0xe32c7472;
/// @dev Error code for when ECPAIRING inputs are invalid.
uint32 constant ERR_INVALID_EC_PAIRING_INPUTS = 0x4385b511;
/// @dev Error code for when the evaluation of a round in a sumcheck proof does not match the expected value.
uint32 constant ERR_ROUND_EVALUATION_MISMATCH = 0x741f5c3f;
/// @dev Error code for when a dequeue attempt was made on an empty queue.
uint32 constant ERR_EMPTY_QUEUE = 0x31dcf2b5;
/// @dev Error code for when the HyperKZG proof has an inconsistent v.
uint32 constant ERR_HYPER_KZG_INCONSISTENT_V = 0x6a5ae827;
/// @dev Error code for when the produces constraint degree is higher than the provided proof.
uint32 constant ERR_CONSTRAINT_DEGREE_TOO_HIGH = 0x8568ae69;

library Errors {
    /// @notice Error thrown when the inputs to the ECADD precompile are invalid.
    error InvalidECAddInputs();
    /// @notice Error thrown when the inputs to the ECMUL precompile are invalid.
    error InvalidECMulInputs();
    /// @notice Error thrown when the inputs to the ECPAIRING precompile are invalid.
    error InvalidECPairingInputs();
    /// @notice Error thrown when the evaluation of a round in a sumcheck proof does not match the expected value.
    error RoundEvaluationMismatch();
    /// @notice Error thrown when a dequeue attempt was made on an empty queue.
    error EmptyQueue();
    /// @notice Error thrown when the HyperKZG proof has an inconsistent v.
    error HyperKZGInconsistentV();
    /// @notice Error thrown when the produces constraint degree is higher than the provided proof.
    error ConstraintDegreeTooHigh();

    function __err(uint32 __code) internal pure {
        assembly {
            function err(code) {
                mstore(0, code)
                revert(28, 4)
            }
            err(__code)
        }
    }
}
