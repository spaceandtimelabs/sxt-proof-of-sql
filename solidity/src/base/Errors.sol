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
/// @dev Error code for when too few challenges are provided to the verification builder.
uint32 constant ERR_TOO_FEW_CHALLENGES = 0x700caebe;
/// @dev Error code for when too few first round mles are provided to the verification builder.
uint32 constant ERR_TOO_FEW_FIRST_ROUND_MLES = 0x82a47d4f;
/// @dev Error code for when too few final round mles are provided to the verification builder.
uint32 constant ERR_TOO_FEW_FINAL_ROUND_MLES = 0xfb828ab5;
/// @dev Error code for when too few chi evaluations are provided to the verification builder.
uint32 constant ERR_TOO_FEW_CHI_EVALUATIONS = 0x8ef4e6c9;
/// @dev Error code for when too few rho evaluations are provided to the verification builder.
uint32 constant ERR_TOO_FEW_RHO_EVALUATIONS = 0x3784ad97;
/// @dev Error code for when the HyperKZG proof has an inconsistent v.
uint32 constant ERR_HYPER_KZG_INCONSISTENT_V = 0x6a5ae827;

library Errors {
    /// @notice Error thrown when the inputs to the ECADD precompile are invalid.
    error InvalidECAddInputs();
    /// @notice Error thrown when the inputs to the ECMUL precompile are invalid.
    error InvalidECMulInputs();
    /// @notice Error thrown when the inputs to the ECPAIRING precompile are invalid.
    error InvalidECPairingInputs();
    /// @notice Error thrown when the evaluation of a round in a sumcheck proof does not match the expected value.
    error RoundEvaluationMismatch();
    /// @notice Error thrown when too few challenges are provided to the verification builder.
    error TooFewChallenges();
    /// @notice Error thrown when too few first round mles are provided to the verification builder.
    error TooFewFirstRoundMLEs();
    /// @notice Error thrown when too few final round mles are provided to the verification builder.
    error TooFewFinalRoundMLEs();
    /// @notice Error thrown when too few chi evaluations are provided to the verification builder.
    error TooFewChiEvaluations();
    /// @notice Error thrown when too few rho evaluations are provided to the verification builder.
    error TooFewRhoEvaluations();
    /// @notice Error thrown when the HyperKZG proof has an inconsistent v.
    error HyperKZGInconsistentV();

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
