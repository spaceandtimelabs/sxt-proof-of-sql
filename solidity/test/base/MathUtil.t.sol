// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {MathUtil} from "../../src/base/MathUtil.sol";

library MathUtilTest {
    function testWeCanComputeLog2Up() public pure {
        /* solhint-disable gas-strict-inequalities */
        for (uint256 i = 0; i <= 2; ++i) {
            assert(MathUtil.__log2Up(i) == 1);
        }
        for (uint256 i = 3; i <= 4; ++i) {
            assert(MathUtil.__log2Up(i) == 2);
        }
        for (uint256 i = 5; i <= 8; ++i) {
            assert(MathUtil.__log2Up(i) == 3);
        }
        for (uint256 i = 9; i <= 16; ++i) {
            assert(MathUtil.__log2Up(i) == 4);
        }
        for (uint256 i = 17; i <= 32; ++i) {
            assert(MathUtil.__log2Up(i) == 5);
        }
        /* solhint-enable gas-strict-inequalities */
    }

    function testFuzzLog2Up(uint256 value) public pure {
        uint256 exponent = MathUtil.__log2Up(value);
        if (value < 2) {
            assert(exponent == 1);
            return;
        } else if (exponent < 256) {
            assert((1 << exponent) >= value); // solhint-disable-line gas-strict-inequalities
            assert((1 << (exponent - 1)) < value);
        } else {
            assert(exponent == 256);
            assert((1 << 255) < value);
        }
    }
}
