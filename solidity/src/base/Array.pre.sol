// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "./Constants.sol";
import "./Errors.sol";

/// @title Array
/// @dev Library providing array utility functions with bounds checking.
library Array {
    /// @notice Gets an element from an array with bounds checking
    /// @custom:as-yul-wrapper
    /// #### Wrapped Yul Function
    /// ##### Signature
    /// ```yul
    /// get_array_element(arr_ptr, index) -> value
    /// ```
    /// ##### Parameters
    /// * `arr_ptr` - pointer to the array in memory. In Solidity memory layout,
    ///   this points to where the array length is stored, followed by the array elements
    /// * `index` - the index of the element to retrieve
    /// ##### Return Values
    /// * `value` - the element at the specified index
    /// @dev Retrieves an element at the specified index with bounds checking.
    /// Reverts with Errors.InvalidIndex if the index is out of bounds.
    /// @param __array Single-element array containing the array to get element from
    /// @param __index The index of the element to retrieve
    /// @return __value The element at the specified index
    function __getArrayElement(uint256[][1] memory __array, uint256 __index) internal pure returns (uint256 __value) {
        assembly {
            // IMPORT-YUL Errors.sol
            function err(code) {
                revert(0, 0)
            }
            function get_array_element(arr_ptr, index) -> value {
                let arr := mload(arr_ptr)
                let length := mload(arr)
                if iszero(lt(index, length)) { err(ERR_INVALID_INDEX) }
                value := mload(add(add(arr, WORD_SIZE), mul(index, WORD_SIZE)))
            }
            __value := get_array_element(__array, __index)
        }
    }
}
