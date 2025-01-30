// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

// This is the modulus of the bn254 scalar field.
uint256 constant MODULUS = 0x30644e72_e131a029_b85045b6_8181585d_2833e848_79b97091_43e1f593_f0000001;
// This is largest mask that can be applied to a 256-bit number in order to enforce that it is less than the modulus.
uint256 constant MODULUS_MASK = 0x1FFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF_FFFFFFFF;
// This is the size of a word in bytes: 32.
uint256 constant WORD_SIZE = 0x20;
// This is the position of the free memory pointer in the context of the EVM memory.
uint256 constant FREE_PTR = 0x40;
