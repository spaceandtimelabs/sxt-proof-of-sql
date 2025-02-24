// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "./Errors.sol";

/// @title SwitchUtil
/// @dev Library providing helper functions for switch statement validation.
library SwitchUtil {
    /// @notice Validates that two values in a switch case statement match
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// case_const(lhs, rhs)
    /// ```
    /// ##### Parameters
    /// * `lhs` - the left-hand side value to compare
    /// * `rhs` - the right-hand side value to compare
    /// @dev This function reverts with ERR_INCORRECT_CASE_CONST if the values don't match
    /// @dev Note: This function is designed to be used with constant values. When both lhs and rhs
    /// @dev are constants and the --optimize flag is used, the entire function call will be eliminated
    /// @dev at compile time. The compiler will either:
    /// @dev 1. Remove the call entirely if the constants match
    /// @dev 2. Replace it with a direct revert if they don't match
    /// @dev This means there is zero runtime overhead for switch statement validation in the intended usage.
    /// @param __lhs The left-hand side value
    /// @param __rhs The right-hand side value
    function __caseConst(uint256 __lhs, uint256 __rhs) internal pure {
        assembly {
            // IMPORT-YUL Errors.sol
            function err(code) {
                revert(0, 0)
            }
            function case_const(lhs, rhs) {
                if sub(lhs, rhs) { err(ERR_INCORRECT_CASE_CONST) }
            }
            case_const(__lhs, __rhs)
        }
    }
}
