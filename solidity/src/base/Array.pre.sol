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

    /// @notice Reads a uint64 array from calldata
    /// @custom:yul-function
    /// #### Yul Function
    /// ##### Signature
    /// ```yul
    /// read_uint64_array(source_ptr) -> source_ptr_out, array_ptr
    /// ```
    /// ##### Parameters
    /// * `source_ptr` - calldata pointer to array length followed by array elements
    /// ##### Return Values
    /// * `source_ptr_out` - pointer to remaining calldata after consuming the array
    /// * `array_ptr` - pointer to the array in memory containing [length, elements...]
    /// @dev Reads a uint64 array by first reading a uint64 length, then reading that many uint64 values.
    /// The values are shifted right by UINT64_PADDING_BITS to get the actual uint64 values.
    /// @param __source The input source containing the array
    /// @return __sourceOut The remaining source after consuming the array
    /// @return __array The decoded array
    function __readUint64Array(bytes calldata __source)
        external
        pure
        returns (bytes calldata __sourceOut, uint256[] memory __array)
    {
        uint256[] memory __arrayTmp;
        assembly {
            function read_uint64_array(source_ptr) -> source_ptr_out, array_ptr {
                array_ptr := mload(FREE_PTR)

                let length := shr(UINT64_PADDING_BITS, calldataload(source_ptr))
                mstore(array_ptr, length)
                source_ptr := add(source_ptr, UINT64_SIZE)

                let tmp_ptr := add(array_ptr, WORD_SIZE)
                for {} length { length := sub(length, 1) } {
                    mstore(tmp_ptr, shr(UINT64_PADDING_BITS, calldataload(source_ptr)))
                    source_ptr := add(source_ptr, UINT64_SIZE)
                    tmp_ptr := add(tmp_ptr, WORD_SIZE)
                }

                mstore(FREE_PTR, tmp_ptr)

                source_ptr_out := source_ptr
            }

            let __sourceOutOffset
            __sourceOutOffset, __arrayTmp := read_uint64_array(__source.offset)
            __sourceOut.offset := __sourceOutOffset
            // slither-disable-next-line write-after-write
            __sourceOut.length := sub(__source.length, sub(__sourceOutOffset, __source.offset))
        }
        __array = __arrayTmp;
    }
}
