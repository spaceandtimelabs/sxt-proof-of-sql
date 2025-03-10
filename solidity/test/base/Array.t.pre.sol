// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import {Array} from "../../src/base/Array.pre.sol";
import {Errors} from "../../src/base/Errors.sol";

contract ArrayTest is Test {
    /// forge-config: default.allow_internal_expect_revert = true
    function testGetArrayElementEmptyArray() public {
        uint256[][1] memory array = [new uint256[](0)];

        vm.expectRevert(Errors.InvalidIndex.selector);
        Array.__getArrayElement(array, 0);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testGetArrayElementOutOfBounds() public {
        uint256[][1] memory array = [new uint256[](3)];
        array[0][0] = 0x12345678;
        array[0][1] = 0x23456789;
        array[0][2] = 0x3456789A;

        vm.expectRevert(Errors.InvalidIndex.selector);
        Array.__getArrayElement(array, 3);
    }

    function testGetArrayElement() public pure {
        uint256[][1] memory array = [new uint256[](3)];
        array[0][0] = 0x12345678;
        array[0][1] = 0x23456789;
        array[0][2] = 0x3456789A;

        assert(Array.__getArrayElement(array, 0) == 0x12345678);
        assert(Array.__getArrayElement(array, 1) == 0x23456789);
        assert(Array.__getArrayElement(array, 2) == 0x3456789A);
    }

    function testFuzzGetArrayElement(uint256[] memory values) public pure {
        vm.assume(values.length > 0);

        uint256[][1] memory array = [values];

        uint256 valuesLength = values.length;
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(Array.__getArrayElement(array, i) == values[i]);
        }
    }
}
