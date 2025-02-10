// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

/// @dev The modulus of the bn254 scalar field.
uint256 constant MODULUS = 0x30644e72_e131a029_b85045b6_8181585d_2833e848_79b97091_43e1f593_f0000001;
/// @dev The largest mask that can be applied to a 256-bit number in order to enforce that it is less than the modulus.
uint256 constant MODULUS_MASK = 0x1FFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF;
/// @dev MODULUS + 1. Needs to be explicit for Yul usage.
uint256 constant MODULUS_PLUS_ONE = 0x30644e72_e131a029_b85045b6_8181585d_2833e848_79b97091_43e1f593_f0000002;
/// @dev Size of a word in bytes: 32.
uint256 constant WORD_SIZE = 0x20;
/// @dev Size of two words in bytes.
uint256 constant WORDX2_SIZE = 0x20 * 2;
/// @dev Size of three words in bytes.
uint256 constant WORDX3_SIZE = 0x20 * 3;
/// @dev Size of four words in bytes.
uint256 constant WORDX4_SIZE = 0x20 * 4;
/// @dev Size of twelve words in bytes.
uint256 constant WORDX12_SIZE = 0x20 * 12;

/// @dev Position of the free memory pointer in the context of the EVM memory.
uint256 constant FREE_PTR = 0x40;

/// @dev Address of the ECADD precompile.
uint256 constant ECADD_ADDRESS = 0x06;
/// @dev Address of the ECMUL precompile.
uint256 constant ECMUL_ADDRESS = 0x07;
/// @dev Address of the ECPAIRING precompile.
uint256 constant ECPAIRING_ADDRESS = 0x08;
/// @dev Gas cost for the ECADD precompile.
uint256 constant ECADD_GAS = 150;
/// @dev Gas cost for the ECMUL precompile.
uint256 constant ECMUL_GAS = 6000;
/// @dev Gas cost for the ECPAIRING precompile with two pairings.
uint256 constant ECPAIRINGX2_GAS = 45000 + 2 * 34000;

/// @dev Error code for when ECADD inputs are invalid.
uint256 constant INVALID_EC_ADD_INPUTS = 0x765bcba0_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
/// @dev Error code for when ECMUL inputs are invalid.
uint256 constant INVALID_EC_MUL_INPUTS = 0xe32c7472_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
/// @dev Error code for when ECPAIRING inputs are invalid.
uint256 constant INVALID_EC_PAIRING_INPUTS = 0x4385b511_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
/// @dev Error code for when the evaluation of a round in a sumcheck proof does not match the expected value.
uint256 constant ROUND_EVALUATION_MISMATCH = 0x741f5c3f_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
/// @dev Error code for when too few challenges are provided to the verification builder.
uint256 constant TOO_FEW_CHALLENGES = 0x700caebe_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
/// @dev Error code for when too few final round mles are provided to the verification builder.
uint256 constant TOO_FEW_FINAL_ROUND_MLES = 0xfb828ab5_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

/// @dev The X coordinate of the G1 generator point.
uint256 constant G1_GEN_X = 1;
/// @dev The Y coordinate of the G1 generator point.
uint256 constant G1_GEN_Y = 2;

/// @dev The G2 generator point's x-coordinate real component.
uint256 constant G2_GEN_X_REAL = 0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed;
/// @dev The G2 generator point's x-coordinate imaginary component.
uint256 constant G2_GEN_X_IMAG = 0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2;
/// @dev The G2 generator point's y-coordinate real component.
uint256 constant G2_GEN_Y_REAL = 0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa;
/// @dev The G2 generator point's y-coordinate imaginary component.
uint256 constant G2_GEN_Y_IMAG = 0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b;

/// @dev The X coordinate of the negated G1 generator point.
uint256 constant G1_NEG_GEN_X = 1;
/// @dev The Y coordinate of the negated G1 generator point.
uint256 constant G1_NEG_GEN_Y = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd45;

/// @dev The G2 negated generator point's x-coordinate real component.
uint256 constant G2_NEG_GEN_X_REAL = 0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed;
/// @dev The G2 negated generator point's x-coordinate imaginary component.
uint256 constant G2_NEG_GEN_X_IMAG = 0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2;
/// @dev The G2 negated generator point's y-coordinate real component.
uint256 constant G2_NEG_GEN_Y_REAL = 0x1d9befcd05a5323e6da4d435f3b617cdb3af83285c2df711ef39c01571827f9d;
/// @dev The G2 negated generator point's y-coordinate imaginary component.
uint256 constant G2_NEG_GEN_Y_IMAG = 0x275dc4a288d1afb3cbb1ac09187524c7db36395df7be3b99e673b13a075a65ec;

/// @dev Size of the verification builder in bytes.
uint256 constant VERIFICATION_BUILDER_SIZE = 0x20 * 4;
/// @dev Offset of the pointer to the head of the challenge queue in the verification builder.
uint256 constant CHALLENGE_HEAD_OFFSET = 0x20 * 0;
/// @dev Offset of the pointer to the tail of the challenge queue in the verification builder.
uint256 constant CHALLENGE_TAIL_OFFSET = 0x20 * 1;
/// @dev Offset of the pointer to the head of the final round mles in the verification builder.
uint256 constant FINAL_ROUND_MLE_HEAD_OFFSET = 0x20 * 2;
/// @dev Offset of the pointer to the tail of the final round mles in the verification builder.
uint256 constant FINAL_ROUND_MLE_TAIL_OFFSET = 0x20 * 3;

/// @title Errors library
/// @notice Library containing custom error definitions.
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
    /// @notice Error thrown when too few final round mles are provided to the verification builder.
    error TooFewFinalRoundMLEs();
}
