// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

/// @dev Error code for when ECADD inputs are invalid.
uint32 constant ERR_INVALID_EC_ADD_INPUTS = 0x765bcba0;
/// @dev Error code for when ECMUL inputs are invalid.
uint32 constant ERR_INVALID_EC_MUL_INPUTS = 0xe32c7472;
/// @dev Error code for when ECPAIRING inputs are invalid.
uint32 constant ERR_INVALID_EC_PAIRING_INPUTS = 0x4385b511;
/// @dev Error code for when the size of a sumcheck proof is incorrect.
uint32 constant ERR_INVALID_SUMCHECK_PROOF_SIZE = 0x3f889a17;
/// @dev Error code for when the evaluation of a round in a sumcheck proof does not match the expected value.
uint32 constant ERR_ROUND_EVALUATION_MISMATCH = 0x741f5c3f;
/// @dev Error code for when a dequeue attempt was made on an empty queue.
uint32 constant ERR_EMPTY_QUEUE = 0x31dcf2b5;
/// @dev Error code for when the HyperKZG proof has an inconsistent v.
uint32 constant ERR_HYPER_KZG_INCONSISTENT_V = 0x6a5ae827;
/// @dev Error code for when the HyperKZG proof has an empty x point.
uint32 constant ERR_HYPER_KZG_EMPTY_POINT = 0xf1c6069e;
/// @dev Error code for when the HyperKZG proof fails the pairing check.
uint32 constant ERR_HYPER_KZG_PAIRING_CHECK_FAILED = 0xa41148a3;
/// @dev Error code for when the produces constraint degree is higher than the provided proof.
uint32 constant ERR_CONSTRAINT_DEGREE_TOO_HIGH = 0x8568ae69;
/// @dev Error code for when the case literal in a switch statement is incorrect.
uint32 constant ERR_INCORRECT_CASE_CONST = 0x9324fb03;
/// @dev Error code for when a literal variant is unsupported.
uint32 constant ERR_UNSUPPORTED_LITERAL_VARIANT = 0xed9d5b00;
/// @dev Error code for when an index is invalid.
uint32 constant ERR_INVALID_INDEX = 0x63df8171;
/// @dev Error code for when a proof expression variant is unsupported.
uint32 constant ERR_UNSUPPORTED_PROOF_EXPR_VARIANT = 0xb8a26620;
/// @dev Error code for when PCS batch lengths don't match.
uint32 constant ERR_PCS_BATCH_LENGTH_MISMATCH = 0x5a64ac85;
/// @dev Error code for when result column counts don't match.
uint32 constant ERR_RESULT_COLUMN_COUNT_MISMATCH = 0x4b08a100;
/// @dev Error code for when a result column name is invalid.
uint32 constant ERR_INVALID_RESULT_COLUMN_NAME = 0xc5a456b6;
/// @dev Error code for when result column lengths are inconsistent.
uint32 constant ERR_INCONSISTENT_RESULT_COLUMN_LENGTHS = 0x68c99843;
/// @dev Error code for when the result is incorrect.
uint32 constant ERR_INCORRECT_RESULT = 0x3ad072a3;
/// @dev Error code for when HyperKZG proof size doesn't match.
uint32 constant ERR_HYPER_KZG_PROOF_SIZE_MISMATCH = 0xbe285ccd;
/// @dev Error code for when aggregate evaluation doesn't match.
uint32 constant ERR_AGGREGATE_EVALUATION_MISMATCH = 0xf5c6cb38;
/// @dev Error code for when proof type is unsupported.
uint32 constant ERR_UNSUPPORTED_PROOF = 0x6f1c50d9;
/// @dev Error code for when a proof plan variant is unsupported.
uint32 constant ERR_UNSUPPORTED_PROOF_PLAN_VARIANT = 0xe5503cfa;
/// @dev Error code for when a data type variant is unsupported.
uint32 constant ERR_UNSUPPORTED_DATA_TYPE_VARIANT = 0xbd12560e;

library Errors {
    /// @notice Error thrown when the inputs to the ECADD precompile are invalid.
    error InvalidECAddInputs();
    /// @notice Error thrown when the inputs to the ECMUL precompile are invalid.
    error InvalidECMulInputs();
    /// @notice Error thrown when the inputs to the ECPAIRING precompile are invalid.
    error InvalidECPairingInputs();
    /// @notice Error thrown when the size of a sumcheck proof is incorrect.
    error InvalidSumcheckProofSize();
    /// @notice Error thrown when the evaluation of a round in a sumcheck proof does not match the expected value.
    error RoundEvaluationMismatch();
    /// @notice Error thrown when a dequeue attempt was made on an empty queue.
    error EmptyQueue();
    /// @notice Error thrown when the HyperKZG proof has an inconsistent v.
    error HyperKZGInconsistentV();
    /// @notice Error thrown when the HyperKZG proof has an empty x point.
    error HyperKZGEmptyPoint();
    /// @notice Error thrown when the HyperKZG proof fails the pairing check.
    error HyperKZGPairingCheckFailed();
    /// @notice Error thrown when the produces constraint degree is higher than the provided proof.
    error ConstraintDegreeTooHigh();
    /// @notice Error thrown when the case literal in a switch statement is incorrect.
    error IncorrectCaseConst();
    /// @notice Error thrown when a literal variant is unsupported.
    error UnsupportedLiteralVariant();
    /// @notice Error thrown when an index is invalid.
    error InvalidIndex();
    /// @notice Error thrown when a proof expression variant is unsupported.
    error UnsupportedProofExprVariant();
    /// @notice Error thrown when PCS batch lengths don't match.
    error PCSBatchLengthMismatch();
    /// @notice Error thrown when result column counts don't match.
    error ResultColumnCountMismatch();
    /// @notice Error thrown when a result column name is invalid.
    error InvalidResultColumnName();
    /// @notice Error thrown when result column lengths are inconsistent.
    error InconsistentResultColumnLengths();
    /// @notice Error thrown when the result is incorrect.
    error IncorrectResult();
    /// @notice Error thrown when HyperKZG proof size doesn't match.
    error HyperKZGProofSizeMismatch();
    /// @notice Error thrown when aggregate evaluation doesn't match.
    error AggregateEvaluationMismatch();
    /// @notice Error thrown when proof type is unsupported.
    error UnsupportedProof();
    /// @notice Error thrown when a proof plan variant is unsupported.
    error UnsupportedProofPlanVariant();
    /// @notice Error thrown when a data type variant is unsupported.
    error UnsupportedDataTypeVariant();

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
