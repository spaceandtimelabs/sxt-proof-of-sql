// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Errors.sol";

contract ErrorsTest is Test {
    function testErrorConstantsMatchSelectors() public pure {
        bytes4[10] memory selectors = [
            Errors.InvalidECAddInputs.selector,
            Errors.InvalidECMulInputs.selector,
            Errors.InvalidECPairingInputs.selector,
            Errors.RoundEvaluationMismatch.selector,
            Errors.TooFewChallenges.selector,
            Errors.TooFewFirstRoundMLEs.selector,
            Errors.TooFewFinalRoundMLEs.selector,
            Errors.TooFewChiEvaluations.selector,
            Errors.TooFewRhoEvaluations.selector,
            Errors.HyperKZGInconsistentV.selector
        ];
        uint32[10] memory selectorConstants = [
            ERR_INVALID_EC_ADD_INPUTS,
            ERR_INVALID_EC_MUL_INPUTS,
            ERR_INVALID_EC_PAIRING_INPUTS,
            ERR_ROUND_EVALUATION_MISMATCH,
            ERR_TOO_FEW_CHALLENGES,
            ERR_TOO_FEW_FIRST_ROUND_MLES,
            ERR_TOO_FEW_FINAL_ROUND_MLES,
            ERR_TOO_FEW_CHI_EVALUATIONS,
            ERR_TOO_FEW_RHO_EVALUATIONS,
            ERR_HYPER_KZG_INCONSISTENT_V
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
    function testErrorFailedTooFewChallenges() public {
        vm.expectRevert(Errors.TooFewChallenges.selector);
        Errors.__err(ERR_TOO_FEW_CHALLENGES);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedTooFewFirstRoundMLEs() public {
        vm.expectRevert(Errors.TooFewFirstRoundMLEs.selector);
        Errors.__err(ERR_TOO_FEW_FIRST_ROUND_MLES);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedTooFewFinalRoundMLEs() public {
        vm.expectRevert(Errors.TooFewFinalRoundMLEs.selector);
        Errors.__err(ERR_TOO_FEW_FINAL_ROUND_MLES);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedTooFewChiEvaluations() public {
        vm.expectRevert(Errors.TooFewChiEvaluations.selector);
        Errors.__err(ERR_TOO_FEW_CHI_EVALUATIONS);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedTooFewRhoEvaluations() public {
        vm.expectRevert(Errors.TooFewRhoEvaluations.selector);
        Errors.__err(ERR_TOO_FEW_RHO_EVALUATIONS);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorFailedHyperKZGInconsistentV() public {
        vm.expectRevert(Errors.HyperKZGInconsistentV.selector);
        Errors.__err(ERR_HYPER_KZG_INCONSISTENT_V);
    }
}
