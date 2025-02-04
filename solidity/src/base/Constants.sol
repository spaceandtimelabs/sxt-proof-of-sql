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

/// @title Errors library
/// @notice Library containing custom error definitions.
library Errors {
    /// @notice Error thrown when the inputs to the ECADD precompile are invalid.
    error InvalidECAddInputs();
    /// @notice Error thrown when the inputs to the ECMUL precompile are invalid.
    error InvalidECMulInputs();
    /// @notice Error thrown when the inputs to the ECPAIRING precompile are invalid.
    error InvalidECPairingInputs();
}
