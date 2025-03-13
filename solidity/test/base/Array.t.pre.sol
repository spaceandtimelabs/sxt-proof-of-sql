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

    function testEmptyReadUint64Array() public pure {
        bytes memory source = abi.encodePacked(uint64(0), hex"abcdef");

        (bytes memory sourceOut, uint256[] memory array) = Array.__readUint64Array(source);

        assert(array.length == 0);
        assert(sourceOut.length == 3);
        assert(sourceOut[0] == 0xab);
        assert(sourceOut[1] == 0xcd);
        assert(sourceOut[2] == 0xef);
    }

    function testSimpleReadUint64Array() public pure {
        bytes memory source = abi.encodePacked(uint64(3), uint64(1), uint64(2), uint64(3), hex"abcdef");

        (bytes memory sourceOut, uint256[] memory array) = Array.__readUint64Array(source);

        assert(array.length == 3);
        assert(array[0] == 1);
        assert(array[1] == 2);
        assert(array[2] == 3);
        assert(sourceOut.length == 3);
        assert(sourceOut[0] == 0xab);
        assert(sourceOut[1] == 0xcd);
        assert(sourceOut[2] == 0xef);
    }

    function testFuzzReadUint64Array(bytes calldata source) public pure {
        uint256 sourceLength = source.length;
        vm.assume(sourceLength > 7);
        uint256 length = uint256(uint64(bytes8(source[0:8])));
        vm.assume(sourceLength > 7 + length * 8);

        (bytes memory sourceOut, uint256[] memory array) = Array.__readUint64Array(source);

        for (uint256 i = 0; i < length; ++i) {
            assert(array[i] == uint256(uint64(bytes8(source[8 + i * 8:16 + i * 8]))));
        }
        for (uint256 i = 8 + length * 8; i < sourceLength; ++i) {
            assert(sourceOut[i - 8 - length * 8] == source[i]);
        }
    }

    function testEmptyReadWordArray() public pure {
        bytes memory source = abi.encodePacked(uint64(0), hex"abcdef");

        (bytes memory sourceOut, uint256[] memory array) = Array.__readWordArray(source);

        assert(array.length == 0);
        assert(sourceOut.length == 3);
        assert(sourceOut[0] == 0xab);
        assert(sourceOut[1] == 0xcd);
        assert(sourceOut[2] == 0xef);
    }

    function testSimpleReadWordArray() public pure {
        bytes memory source = abi.encodePacked(uint64(3), uint256(1), uint256(2), uint256(3), hex"abcdef");

        (bytes memory sourceOut, uint256[] memory array) = Array.__readWordArray(source);

        assert(array.length == 3);
        assert(array[0] == 1);
        assert(array[1] == 2);
        assert(array[2] == 3);
        assert(sourceOut.length == 3);
        assert(sourceOut[0] == 0xab);
        assert(sourceOut[1] == 0xcd);
        assert(sourceOut[2] == 0xef);
    }

    function testFuzzReadWordArray(bytes calldata source) public pure {
        uint256 sourceLength = source.length;
        vm.assume(sourceLength > 7);
        uint256 length = uint256(uint64(bytes8(source[0:8])));
        vm.assume(sourceLength > 7 + length * 32);

        (bytes memory sourceOut, uint256[] memory array) = Array.__readWordArray(source);

        for (uint256 i = 0; i < length; ++i) {
            assert(array[i] == uint256(bytes32(source[8 + i * 32:40 + i * 32])));
        }
        for (uint256 i = 8 + length * 32; i < sourceLength; ++i) {
            assert(sourceOut[i - 8 - length * 32] == source[i]);
        }
    }

    function testEmptyReadWordx2Array() public pure {
        bytes memory source = abi.encodePacked(uint64(0), hex"abcdef");

        (bytes memory sourceOut, uint256[2][] memory array) = Array.__readWordx2Array(source);

        assert(array.length == 0);
        assert(sourceOut.length == 3);
        assert(sourceOut[0] == 0xab);
        assert(sourceOut[1] == 0xcd);
        assert(sourceOut[2] == 0xef);
    }

    function testSimpleReadWordx2Array() public pure {
        // Create test data with two words per element
        bytes memory source = abi.encodePacked(
            uint64(2), // length
            uint256(1),
            uint256(2), // first element
            uint256(3),
            uint256(4), // second element
            hex"abcdef" // remainder
        );

        (bytes memory sourceOut, uint256[2][] memory array) = Array.__readWordx2Array(source);

        assert(array.length == 2);
        assert(array[0][0] == 1);
        assert(array[0][1] == 2);
        assert(array[1][0] == 3);
        assert(array[1][1] == 4);
        assert(sourceOut.length == 3);
        assert(sourceOut[0] == 0xab);
        assert(sourceOut[1] == 0xcd);
        assert(sourceOut[2] == 0xef);
    }

    function testFuzzReadWordx2Array(bytes calldata source) public pure {
        uint256 sourceLength = source.length;
        vm.assume(sourceLength > 7);
        uint256 length = uint256(uint64(bytes8(source[0:8])));
        vm.assume(sourceLength > 7 + length * 64); // 64 bytes per element (2 words)

        (bytes memory sourceOut, uint256[2][] memory array) = Array.__readWordx2Array(source);

        for (uint256 i = 0; i < length; ++i) {
            assert(array[i][0] == uint256(bytes32(source[8 + i * 64:40 + i * 64])));
            assert(array[i][1] == uint256(bytes32(source[40 + i * 64:72 + i * 64])));
        }
        for (uint256 i = 8 + length * 64; i < sourceLength; ++i) {
            assert(sourceOut[i - 8 - length * 64] == source[i]);
        }
    }
}
