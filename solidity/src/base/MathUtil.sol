// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

/// @title Math Utilities Library
/// @notice Provides functions to perform various math operations
library MathUtil {
    /// @notice Computes `max(1,ceil(log_2(value)))`
    /// @dev The smallest integer greater than or equal to the base 2 logarithm of a number.
    /// If the number is less than 2, the result is 1.
    /// @param __value The input value for which to compute the logarithm
    /// @return __exponent The computed logarithm value
    function __log2Up(uint256 __value) internal pure returns (uint256 __exponent) {
        assembly {
            function log2_up(value) -> exponent {
                if value { value := sub(value, 1) }
                exponent := 1
                if gt(value, 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF) {
                    exponent := add(exponent, 128)
                    value := shr(128, value)
                }
                if gt(value, 0xFFFFFFFFFFFFFFFF) {
                    exponent := add(exponent, 64)
                    value := shr(64, value)
                }
                if gt(value, 0xFFFFFFFF) {
                    exponent := add(exponent, 32)
                    value := shr(32, value)
                }
                if gt(value, 0xFFFF) {
                    exponent := add(exponent, 16)
                    value := shr(16, value)
                }
                if gt(value, 0xFF) {
                    exponent := add(exponent, 8)
                    value := shr(8, value)
                }
                if gt(value, 0xF) {
                    exponent := add(exponent, 4)
                    value := shr(4, value)
                }
                if gt(value, 0x3) {
                    exponent := add(exponent, 2)
                    value := shr(2, value)
                }
                if gt(value, 0x1) {
                    exponent := add(exponent, 1)
                    value := shr(1, value)
                }
            }
            __exponent := log2_up(__value)
        }
    }
}
