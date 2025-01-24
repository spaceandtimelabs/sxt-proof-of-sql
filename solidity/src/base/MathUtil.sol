// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

library MathUtil {
    function log2Up(uint256 value_) public pure returns (uint256 exponent_) {
        assembly {
            // returns max(1,ceiling(log_2(value)))
            function log2_up(value) -> exponent {
                if value { value := sub(value, 1) }
                exponent := 1
                for {} shr(exponent, value) {} { exponent := add(exponent, 1) }
            }
            exponent_ := log2_up(value_)
        }
    }

    function testWeCanComputeLog2Up() public pure {
        /* solhint-disable gas-strict-inequalities */
        for (uint256 i = 0; i <= 2; ++i) {
            assert(log2Up(i) == 1);
        }
        for (uint256 i = 3; i <= 4; ++i) {
            assert(log2Up(i) == 2);
        }
        for (uint256 i = 5; i <= 8; ++i) {
            assert(log2Up(i) == 3);
        }
        for (uint256 i = 9; i <= 16; ++i) {
            assert(log2Up(i) == 4);
        }
        for (uint256 i = 17; i <= 32; ++i) {
            assert(log2Up(i) == 5);
        }
        /* solhint-enable gas-strict-inequalities */
    }
}
