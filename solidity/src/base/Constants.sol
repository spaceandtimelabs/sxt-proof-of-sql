// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

/// @dev The modulus of the bn254 scalar field.
uint256 constant MODULUS = 0x30644e72_e131a029_b85045b6_8181585d_2833e848_79b97091_43e1f593_f0000001;
/// @dev The largest mask that can be applied to a 256-bit number in order to enforce that it is less than the modulus.
uint256 constant MODULUS_MASK = 0x1FFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF;
/// @dev MODULUS + 1. Needs to be explicit for Yul usage.
uint256 constant MODULUS_PLUS_ONE = 0x30644e72_e131a029_b85045b6_8181585d_2833e848_79b97091_43e1f593_f0000002;
/// @dev MODULUS - 1. Needs to be explicit for Yul usage.
uint256 constant MODULUS_MINUS_ONE = 0x30644e72_e131a029_b85045b6_8181585d_2833e848_79b97091_43e1f593_f0000000;
/// @dev Size of a word in bytes: 32.
uint256 constant WORD_SIZE = 0x20;
/// @dev Size of two words in bytes.
uint256 constant WORDX2_SIZE = 0x20 * 2;
/// @dev Size of three words in bytes.
uint256 constant WORDX3_SIZE = 0x20 * 3;
/// @dev Size of four words in bytes.
uint256 constant WORDX4_SIZE = 0x20 * 4;
/// @dev Size of five words in bytes.
uint256 constant WORDX5_SIZE = 0x20 * 5;
/// @dev Size of six words in bytes.
uint256 constant WORDX6_SIZE = 0x20 * 6;
/// @dev Size of eight words in bytes.
uint256 constant WORDX8_SIZE = 0x20 * 8;
/// @dev Size of nine words in bytes.
uint256 constant WORDX9_SIZE = 0x20 * 9;
/// @dev Size of ten words in bytes.
uint256 constant WORDX10_SIZE = 0x20 * 10;
/// @dev Size of eleven words in bytes.
uint256 constant WORDX11_SIZE = 0x20 * 11;
/// @dev Size of twelve words in bytes.
uint256 constant WORDX12_SIZE = 0x20 * 12;

/// @dev Size of uint32 in bytes
uint256 constant UINT32_SIZE = 0x04;
/// @dev Number of bits needed to pad uint32 to 256 bits
/// @dev This is useful for shifting a uint256 to the right to extract a uint32
uint256 constant UINT32_PADDING_BITS = 0xE0;
/// @dev Size of uint64 in bytes
uint256 constant UINT64_SIZE = 0x08;
/// @dev Number of bits needed to pad uint64 to 256 bits
/// @dev This is useful for shifting a uint256 to the right to extract a uint64
uint256 constant UINT64_PADDING_BITS = 0xC0;
/// @dev Size of int64 in bytes
uint256 constant INT64_SIZE = 0x08;
/// @dev Number of bits needed to pad int64 to 256 bits
/// @dev This is useful for shifting a uint256 to the right to extract a int64
uint256 constant INT64_PADDING_BITS = 0xC0;
/// @dev Size of int64 minus one byte
uint256 constant INT64_SIZE_MINUS_ONE = 0x07;

/// @dev Column variant constant for proof expressions
uint32 constant COLUMN_EXPR_VARIANT = 0;
/// @dev Literal variant constant for proof expressions
uint32 constant LITERAL_EXPR_VARIANT = 1;
/// @dev Equals variant constant for proof expressions
uint32 constant EQUALS_EXPR_VARIANT = 2;
/// @dev Add variant constant for proof expressions
uint32 constant ADD_EXPR_VARIANT = 3;
/// @dev Subtract variant constant for proof expressions
uint32 constant SUBTRACT_EXPR_VARIANT = 4;
/// @dev Cast variant constant for proof expressions
uint32 constant CAST_EXPR_VARIANT = 5;

/// @dev Filter variant constant for proof plans
uint32 constant FILTER_EXEC_VARIANT = 0;

/// @dev BigInt variant constant for literal expressions
uint32 constant LITERAL_BIGINT_VARIANT = 0;
/// @dev BigInt variant constant for result columns
uint32 constant COLUMN_BIGINT_VARIANT = 0;

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

uint256 constant VK_TAU_HX_REAL = 0x285b1f14edd7e6632340a37dfae9005ff762edcfecfe1c732a7474c0708bef80;
uint256 constant VK_TAU_HX_IMAG = 0x17cc93077f56f654da727c1def86010339c2b4131094547285adb083e48c197b;
uint256 constant VK_TAU_HY_REAL = 0x2bad9a374aec49d329ec66e8f530f68509313450580c4c17c6db5ddb9bde7fd0;
uint256 constant VK_TAU_HY_IMAG = 0x219edfceee1723de674f5b2f6fdb69d9e32dd53b15844956a630d3c7cdaa6ed9;

/// @dev Size of the verification builder in bytes.
uint256 constant VERIFICATION_BUILDER_SIZE = 0x20 * 13;
/// @dev Offset of the pointer to the challenge queue in the verification builder.
uint256 constant BUILDER_CHALLENGES_OFFSET = 0x20 * 0;
/// @dev Offset of the pointer to the first round MLEs in the verification builder.
uint256 constant BUILDER_FIRST_ROUND_MLES_OFFSET = 0x20 * 1;
/// @dev Offset of the pointer to the final round MLEs in the verification builder.
uint256 constant BUILDER_FINAL_ROUND_MLES_OFFSET = 0x20 * 2;
/// @dev Offset of the pointer to the chi evaluations in the verification builder.
uint256 constant BUILDER_CHI_EVALUATIONS_OFFSET = 0x20 * 3;
/// @dev Offset of the pointer to the rho evaluations in the verification builder.
uint256 constant BUILDER_RHO_EVALUATIONS_OFFSET = 0x20 * 4;
/// @dev Offset of the pointer to the constraint multipliers in the verification builder.
uint256 constant BUILDER_CONSTRAINT_MULTIPLIERS_OFFSET = 0x20 * 5;
/// @dev Offset of the max degree in the verification builder.
uint256 constant BUILDER_MAX_DEGREE_OFFSET = 0x20 * 6;
/// @dev Offset of the aggregate evaluation in the verification builder.
uint256 constant BUILDER_AGGREGATE_EVALUATION_OFFSET = 0x20 * 7;
/// @dev Offset of the row multipliers evaluation in the verification builder.
uint256 constant BUILDER_ROW_MULTIPLIERS_EVALUATION_OFFSET = 0x20 * 8;
/// @dev Offset of the pointer to the column evaluations in the verification builder.
uint256 constant BUILDER_COLUMN_EVALUATIONS_OFFSET = 0x20 * 9;
/// @dev Offset of the pointer to the table chi evaluations in the verification builder.
uint256 constant BUILDER_TABLE_CHI_EVALUATIONS_OFFSET = 0x20 * 10;
/// @dev Offset of the pointer to the first round commitments in the verification builder.
uint256 constant BUILDER_FIRST_ROUND_COMMITMENTS_OFFSET = 0x20 * 11;
/// @dev Offset of the pointer to the final round commitments in the verification builder.
uint256 constant BUILDER_FINAL_ROUND_COMMITMENTS_OFFSET = 0x20 * 12;

/// @dev The initial transcript state. This is the hash of the empty string.
uint256 constant INITIAL_TRANSCRIPT_STATE = 0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470;
