// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/proof/VerificationBuilder.sol";

library VerificationBuilderTest {
    function testFuzzAllocateBuilder(uint256[] memory) public pure {
        // Note: the extra parameter is simply to make the free pointer location unpredictable.
        uint256 expectedBuilder;
        assembly {
            expectedBuilder := mload(FREE_PTR)
        }
        assert(VerificationBuilder.__allocate() == expectedBuilder);
        uint256 freePtr;
        assembly {
            freePtr := mload(FREE_PTR)
        }
        assert(freePtr == expectedBuilder + VERIFICATION_BUILDER_SIZE);
    }
}
