// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

library PointerArithmetic {
    function incrementU32(uint256 ptr_) public pure returns (uint256 ptrOut_) {
        assembly {
            function increment_u32(ptr) -> ptr_out {
                ptr_out := add(ptr, 0x04)
            }
            ptrOut_ := increment_u32(ptr_)
        }
    }

    function incrementU32s(uint256 ptr_, uint256 count_) public pure returns (uint256 ptrOut_) {
        assembly {
            function increment_u32s(ptr, count) -> ptr_out {
                ptr_out := add(ptr, shl(0x02, count))
            }
            ptrOut_ := increment_u32s(ptr_, count_)
        }
    }

    function incrementU64(uint256 ptr_) public pure returns (uint256 ptrOut_) {
        assembly {
            function increment_u64(ptr) -> ptr_out {
                ptr_out := add(ptr, 0x08)
            }
            ptrOut_ := increment_u64(ptr_)
        }
    }

    function incrementU64s(uint256 ptr_, uint256 count_) public pure returns (uint256 ptrOut_) {
        assembly {
            function increment_u64s(ptr, count) -> ptr_out {
                ptr_out := add(ptr, shl(0x03, count))
            }
            ptrOut_ := increment_u64s(ptr_, count_)
        }
    }

    function incrementWord(uint256 ptr_) public pure returns (uint256 ptrOut_) {
        assembly {
            function increment_word(ptr) -> ptr_out {
                ptr_out := add(ptr, 0x20)
            }
            ptrOut_ := increment_word(ptr_)
        }
    }

    function incrementWords(uint256 ptr_, uint256 count_) public pure returns (uint256 ptrOut_) {
        assembly {
            function increment_words(ptr, count) -> ptr_out {
                ptr_out := add(ptr, shl(0x05, count))
            }
            ptrOut_ := increment_words(ptr_, count_)
        }
    }

    function calldataloadU32(uint256 i_) public pure returns (uint256 value_) {
        assembly {
            function calldataload_u32(i) -> value {
                value := shr(0xE0, calldataload(i))
            }
            value_ := calldataload_u32(i_)
        }
    }

    function calldataloadU64(uint256 i_) public pure returns (uint256 value_) {
        assembly {
            function calldataload_u64(i) -> value {
                value := shr(0xC0, calldataload(i))
            }
            value_ := calldataload_u64(i_)
        }
    }

    function testIncrementU32() public pure {
        assert(incrementU32(0) == 0x04);
        assert(incrementU32(0x10) == 0x14);
        assert(incrementU32(0xFFFFFFFF) == 0x100000003);
    }

    function testIncrementU32s() public pure {
        assert(incrementU32s(0, 0) == 0);
        assert(incrementU32s(0, 1) == 0x04);
        assert(incrementU32s(0, 2) == 0x08);
        assert(incrementU32s(0x10, 4) == 0x20);
        assert(incrementU32s(0x20, 8) == 0x40);
    }

    function testIncrementU64() public pure {
        assert(incrementU64(0) == 0x08);
        assert(incrementU64(0x10) == 0x18);
        assert(incrementU64(0xFFFFFFFF) == 0x100000007);
    }

    function testIncrementU64s() public pure {
        assert(incrementU64s(0, 0) == 0);
        assert(incrementU64s(0, 1) == 0x08);
        assert(incrementU64s(0, 2) == 0x10);
        assert(incrementU64s(0x10, 4) == 0x30);
    }

    function testIncrementWord() public pure {
        assert(incrementWord(0) == 0x20);
        assert(incrementWord(0x10) == 0x30);
        assert(incrementWord(0xFFFFFFFF) == 0x10000001F);
    }

    function testIncrementWords() public pure {
        assert(incrementWords(0, 0) == 0);
        assert(incrementWords(0, 1) == 0x20);
        assert(incrementWords(0, 2) == 0x40);
        assert(incrementWords(0x20, 4) == 0xA0);
    }

    function _testCalldataloadU32(bytes calldata data) public pure {
        uint256 dataPtr;
        assembly {
            dataPtr := data.offset
        }
        assert(calldataloadU32(dataPtr) == 0x12345678);
    }

    function _testCalldataloadU64(bytes calldata data) public pure {
        uint256 dataPtr;
        assembly {
            dataPtr := data.offset
        }
        assert(calldataloadU64(dataPtr) == 0x1234567890ABCDEF);
    }

    function _testCalldataloadComplex(bytes calldata data) public pure {
        uint256 dataPtr;
        assembly {
            dataPtr := data.offset
        }
        assert(calldataloadU32(dataPtr) == 0x12345678);
        dataPtr = incrementU32(dataPtr);
        assert(calldataloadU64(dataPtr) == 0x1234567890ABCDEF);
        dataPtr = incrementU64(dataPtr);
        assert(calldataloadU32(dataPtr) == 0xAABBCCDD);
        dataPtr = incrementU32(dataPtr);
        assert(calldataloadU32(dataPtr) == 0xEEFF0011);
        dataPtr = incrementU32(dataPtr);
        assert(calldataloadU64(dataPtr) == 0xFEDCBA9876543210);
        dataPtr = incrementU64(dataPtr);
    }
}

contract PointerArithmeticCalldataTest {
    function testCalldataloadU32() public pure {
        PointerArithmetic._testCalldataloadU32(hex"12345678");
    }

    function testCalldataloadU64() public pure {
        PointerArithmetic._testCalldataloadU64(hex"1234567890ABCDEF");
    }

    function testCalldataloadComplex() public pure {
        PointerArithmetic._testCalldataloadComplex(
            hex"12345678" hex"1234567890ABCDEF" hex"AABBCCDD" hex"EEFF0011" hex"FEDCBA9876543210"
        );
    }
}
