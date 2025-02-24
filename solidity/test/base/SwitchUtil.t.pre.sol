// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Errors.sol";
import {SwitchUtil} from "../../src/base/SwitchUtil.pre.sol";

contract SwitchUtilTest is Test {
    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzCaseConst(uint256 lhs, uint256 rhs) public {
        if (lhs != rhs) {
            vm.expectRevert(Errors.IncorrectCaseConst.selector);
        }
        SwitchUtil.__caseConst(lhs, rhs);
    }
}
