// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Errors.sol";

contract ErrorsTest is Test {
    function testErrorConstantsMatchSelectors() public pure {
        bytes4[7] memory selectors = [
            Errors.InvalidECAddInputs.selector,
            Errors.InvalidECMulInputs.selector,
            Errors.InvalidECPairingInputs.selector,
            Errors.RoundEvaluationMismatch.selector,
            Errors.EmptyQueue.selector,
            Errors.HyperKZGInconsistentV.selector,
            Errors.ConstraintDegreeTooHigh.selector
        ];
        uint32[7] memory selectorConstants = [
            ERR_INVALID_EC_ADD_INPUTS,
            ERR_INVALID_EC_MUL_INPUTS,
            ERR_INVALID_EC_PAIRING_INPUTS,
            ERR_ROUND_EVALUATION_MISMATCH,
            ERR_EMPTY_QUEUE,
            ERR_HYPER_KZG_INCONSISTENT_V,
            ERR_CONSTRAINT_DEGREE_TOO_HIGH
        ];
        assert(selectors.length == selectorConstants.length);
        uint256 length = selectors.length;
        for (uint256 i = 0; i < length; ++i) {
            assert(selectors[i] == bytes4(selectorConstants[i]));
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedInvalidECAddInputs() public {
        vm.expectRevert(Errors.InvalidECAddInputs.selector);
        Errors.__err(ERR_INVALID_EC_ADD_INPUTS);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedInvalidECMulInputs() public {
        vm.expectRevert(Errors.InvalidECMulInputs.selector);
        Errors.__err(ERR_INVALID_EC_MUL_INPUTS);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedInvalidECPairingInputs() public {
        vm.expectRevert(Errors.InvalidECPairingInputs.selector);
        Errors.__err(ERR_INVALID_EC_PAIRING_INPUTS);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedRoundEvaluationMismatch() public {
        vm.expectRevert(Errors.RoundEvaluationMismatch.selector);
        Errors.__err(ERR_ROUND_EVALUATION_MISMATCH);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedEmptyQueue() public {
        vm.expectRevert(Errors.EmptyQueue.selector);
        Errors.__err(ERR_EMPTY_QUEUE);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedHyperKZGInconsistentV() public {
        vm.expectRevert(Errors.HyperKZGInconsistentV.selector);
        Errors.__err(ERR_HYPER_KZG_INCONSISTENT_V);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedConstraintDegreeTooHigh() public {
        vm.expectRevert(Errors.ConstraintDegreeTooHigh.selector);
        Errors.__err(ERR_CONSTRAINT_DEGREE_TOO_HIGH);
    }
}
