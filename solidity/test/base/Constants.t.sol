// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";

import {Errors} from "../../src/base/Constants.sol";
// solhint-disable-next-line no-unused-import
import {INVALID_EC_PAIRING_INPUTS} from "../../src/base/Constants.sol";

contract ConstantsTest is Test {
    function testErrorFailedPrecompileStaticcall() public {
        vm.expectPartialRevert(Errors.InvalidECPairingInputs.selector);
        assembly {
            mstore(0, INVALID_EC_PAIRING_INPUTS)
            revert(0, 4)
        }
    }
}
