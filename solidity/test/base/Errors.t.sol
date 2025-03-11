// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Errors.sol";

contract ErrorsTest is Test {
    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorInvalidECAddInputs() public {
        assert(Errors.InvalidECAddInputs.selector == bytes4(ERR_INVALID_EC_ADD_INPUTS));
        vm.expectRevert(Errors.InvalidECAddInputs.selector);
        Errors.__err(ERR_INVALID_EC_ADD_INPUTS);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorInvalidECMulInputs() public {
        assert(Errors.InvalidECMulInputs.selector == bytes4(ERR_INVALID_EC_MUL_INPUTS));
        vm.expectRevert(Errors.InvalidECMulInputs.selector);
        Errors.__err(ERR_INVALID_EC_MUL_INPUTS);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorInvalidECPairingInputs() public {
        assert(Errors.InvalidECPairingInputs.selector == bytes4(ERR_INVALID_EC_PAIRING_INPUTS));
        vm.expectRevert(Errors.InvalidECPairingInputs.selector);
        Errors.__err(ERR_INVALID_EC_PAIRING_INPUTS);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorRoundEvaluationMismatch() public {
        assert(Errors.RoundEvaluationMismatch.selector == bytes4(ERR_ROUND_EVALUATION_MISMATCH));
        vm.expectRevert(Errors.RoundEvaluationMismatch.selector);
        Errors.__err(ERR_ROUND_EVALUATION_MISMATCH);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorEmptyQueue() public {
        assert(Errors.EmptyQueue.selector == bytes4(ERR_EMPTY_QUEUE));
        vm.expectRevert(Errors.EmptyQueue.selector);
        Errors.__err(ERR_EMPTY_QUEUE);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorHyperKZGInconsistentV() public {
        assert(Errors.HyperKZGInconsistentV.selector == bytes4(ERR_HYPER_KZG_INCONSISTENT_V));
        vm.expectRevert(Errors.HyperKZGInconsistentV.selector);
        Errors.__err(ERR_HYPER_KZG_INCONSISTENT_V);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorHyperKZGEmptyPoint() public {
        assert(Errors.HyperKZGEmptyPoint.selector == bytes4(ERR_HYPER_KZG_EMPTY_POINT));
        vm.expectRevert(Errors.HyperKZGEmptyPoint.selector);
        Errors.__err(ERR_HYPER_KZG_EMPTY_POINT);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorHyperKZGPairingCheckFailed() public {
        assert(Errors.HyperKZGPairingCheckFailed.selector == bytes4(ERR_HYPER_KZG_PAIRING_CHECK_FAILED));
        vm.expectRevert(Errors.HyperKZGPairingCheckFailed.selector);
        Errors.__err(ERR_HYPER_KZG_PAIRING_CHECK_FAILED);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorConstraintDegreeTooHigh() public {
        assert(Errors.ConstraintDegreeTooHigh.selector == bytes4(ERR_CONSTRAINT_DEGREE_TOO_HIGH));
        vm.expectRevert(Errors.ConstraintDegreeTooHigh.selector);
        Errors.__err(ERR_CONSTRAINT_DEGREE_TOO_HIGH);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorIncorrectCaseConst() public {
        assert(Errors.IncorrectCaseConst.selector == bytes4(ERR_INCORRECT_CASE_CONST));
        vm.expectRevert(Errors.IncorrectCaseConst.selector);
        Errors.__err(ERR_INCORRECT_CASE_CONST);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorUnsupportedLiteralVariant() public {
        assert(Errors.UnsupportedLiteralVariant.selector == bytes4(ERR_UNSUPPORTED_LITERAL_VARIANT));
        vm.expectRevert(Errors.UnsupportedLiteralVariant.selector);
        Errors.__err(ERR_UNSUPPORTED_LITERAL_VARIANT);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorInvalidIndex() public {
        assert(Errors.InvalidIndex.selector == bytes4(ERR_INVALID_INDEX));
        vm.expectRevert(Errors.InvalidIndex.selector);
        Errors.__err(ERR_INVALID_INDEX);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testErrorUnsupportedProofExprVariant() public {
        assert(Errors.UnsupportedProofExprVariant.selector == bytes4(ERR_UNSUPPORTED_PROOF_EXPR_VARIANT));
        vm.expectRevert(Errors.UnsupportedProofExprVariant.selector);
        Errors.__err(ERR_UNSUPPORTED_PROOF_EXPR_VARIANT);
    }
}
