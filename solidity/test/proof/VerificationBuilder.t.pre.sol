// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/proof/VerificationBuilder.pre.sol";

contract VerificationBuilderTest is Test {
    function testBuilderNewAllocatesValidMemory(bytes memory) public pure {
        uint256 startFreePtr;
        uint256 endFreePtr;
        uint256 builderPtr;
        assembly {
            startFreePtr := mload(FREE_PTR)
        }
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        assembly {
            endFreePtr := mload(FREE_PTR)
            builderPtr := builder
        }
        // NOTE: because solidity allocates more memory than it needs, we end up with a gap between the
        // end of the builder and the beginning of any new memory allocated.
        // This is why we this is an inequality instead of an equality.
        // This is also why we have the `testYulBuilderNewAllocatesValidMemory` check.
        assert(builderPtr >= startFreePtr); // solhint-disable-line gas-strict-inequalities
        assert(endFreePtr - builderPtr == VERIFICATION_BUILDER_SIZE);
    }

    function testYulBuilderNew(bytes memory) public pure {
        uint256 startFreePtr;
        uint256 endFreePtr;
        uint256 builderPtr;
        assembly {
            /// IMPORT-YUL ../../src/proof/VerificationBuilder.pre.sol
            function builder_new() -> builder {
                revert(0, 0)
            }
            startFreePtr := mload(FREE_PTR)
            builderPtr := builder_new()
            endFreePtr := mload(FREE_PTR)
        }
        assert(builderPtr == startFreePtr);
        assert(endFreePtr - builderPtr == VERIFICATION_BUILDER_SIZE);
    }

    function testSetZeroChallenges() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory challenges = new uint256[](0);
        VerificationBuilder.__setChallenges(builder, challenges);
        assert(builder.challenges.length == 0);
    }

    function testSetChallenges() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory challenges = new uint256[](3);
        challenges[0] = 0x12345678;
        challenges[1] = 0x23456789;
        challenges[2] = 0x3456789A;
        VerificationBuilder.__setChallenges(builder, challenges);
        assert(builder.challenges.length == 3);
        assert(builder.challenges[0] == 0x12345678);
        assert(builder.challenges[1] == 0x23456789);
        assert(builder.challenges[2] == 0x3456789A);
    }

    function testFuzzSetChallenges(uint256[] memory values) public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        VerificationBuilder.__setChallenges(builder, values);
        assert(builder.challenges.length == values.length);
        uint256 valuesLength = values.length;
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(builder.challenges[i] == values[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testConsumeZeroChallenges() public {
        VerificationBuilder.Builder memory builder;
        builder.challenges = new uint256[](0);
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeChallenge(builder);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testConsumeChallenges() public {
        VerificationBuilder.Builder memory builder;
        builder.challenges = new uint256[](3);
        builder.challenges[0] = 0x12345678;
        builder.challenges[1] = 0x23456789;
        builder.challenges[2] = 0x3456789A;
        assert(VerificationBuilder.__consumeChallenge(builder) == 0x12345678);
        assert(VerificationBuilder.__consumeChallenge(builder) == 0x23456789);
        assert(VerificationBuilder.__consumeChallenge(builder) == 0x3456789A);
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeChallenge(builder);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzConsumeChallenges(uint256[] memory values) public {
        VerificationBuilder.Builder memory builder;
        uint256 valuesLength = values.length;
        builder.challenges = new uint256[](valuesLength);
        for (uint256 i = 0; i < valuesLength; ++i) {
            builder.challenges[i] = values[i];
        }
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(VerificationBuilder.__consumeChallenge(builder) == values[i]);
        }
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeChallenge(builder);
    }

    function testSetZeroFirstRoundMLEs() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory values = new uint256[](0);
        VerificationBuilder.__setFirstRoundMLEs(builder, values);
        assert(builder.firstRoundMLEs.length == 0);
    }

    function testSetFirstRoundMLEs() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory values = new uint256[](3);
        values[0] = 0x12345678;
        values[1] = 0x23456789;
        values[2] = 0x3456789A;
        VerificationBuilder.__setFirstRoundMLEs(builder, values);
        assert(builder.firstRoundMLEs.length == 3);
        assert(builder.firstRoundMLEs[0] == 0x12345678);
        assert(builder.firstRoundMLEs[1] == 0x23456789);
        assert(builder.firstRoundMLEs[2] == 0x3456789A);
    }

    function testFuzzSetFirstRoundMLEs(uint256[] memory values) public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        VerificationBuilder.__setFirstRoundMLEs(builder, values);
        assert(builder.firstRoundMLEs.length == values.length);
        uint256 valuesLength = values.length;
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(builder.firstRoundMLEs[i] == values[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testConsumeZeroFirstRoundMLEs() public {
        VerificationBuilder.Builder memory builder;
        builder.firstRoundMLEs = new uint256[](0);
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeFirstRoundMLE(builder);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testConsumeFirstRoundMLEs() public {
        VerificationBuilder.Builder memory builder;
        builder.firstRoundMLEs = new uint256[](3);
        builder.firstRoundMLEs[0] = 0x12345678;
        builder.firstRoundMLEs[1] = 0x23456789;
        builder.firstRoundMLEs[2] = 0x3456789A;
        assert(VerificationBuilder.__consumeFirstRoundMLE(builder) == 0x12345678);
        assert(VerificationBuilder.__consumeFirstRoundMLE(builder) == 0x23456789);
        assert(VerificationBuilder.__consumeFirstRoundMLE(builder) == 0x3456789A);
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeFirstRoundMLE(builder);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzConsumeFirstRoundMLEs(uint256[] memory values) public {
        VerificationBuilder.Builder memory builder;
        uint256 valuesLength = values.length;
        builder.firstRoundMLEs = new uint256[](valuesLength);
        for (uint256 i = 0; i < valuesLength; ++i) {
            builder.firstRoundMLEs[i] = values[i];
        }
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(VerificationBuilder.__consumeFirstRoundMLE(builder) == values[i]);
        }
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeFirstRoundMLE(builder);
    }

    function testSetZeroFinalRoundMLEs() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory values = new uint256[](0);
        VerificationBuilder.__setFinalRoundMLEs(builder, values);
        assert(builder.finalRoundMLEs.length == 0);
    }

    function testSetFinalRoundMLEs() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory values = new uint256[](3);
        values[0] = 0x12345678;
        values[1] = 0x23456789;
        values[2] = 0x3456789A;
        VerificationBuilder.__setFinalRoundMLEs(builder, values);
        assert(builder.finalRoundMLEs.length == 3);
        assert(builder.finalRoundMLEs[0] == 0x12345678);
        assert(builder.finalRoundMLEs[1] == 0x23456789);
        assert(builder.finalRoundMLEs[2] == 0x3456789A);
    }

    function testFuzzSetFinalRoundMLEs(uint256[] memory values) public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        VerificationBuilder.__setFinalRoundMLEs(builder, values);
        assert(builder.finalRoundMLEs.length == values.length);
        uint256 valuesLength = values.length;
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(builder.finalRoundMLEs[i] == values[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testConsumeZeroFinalRoundMLEs() public {
        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = new uint256[](0);
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeFinalRoundMLE(builder);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testConsumeFinalRoundMLEs() public {
        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = new uint256[](3);
        builder.finalRoundMLEs[0] = 0x12345678;
        builder.finalRoundMLEs[1] = 0x23456789;
        builder.finalRoundMLEs[2] = 0x3456789A;
        assert(VerificationBuilder.__consumeFinalRoundMLE(builder) == 0x12345678);
        assert(VerificationBuilder.__consumeFinalRoundMLE(builder) == 0x23456789);
        assert(VerificationBuilder.__consumeFinalRoundMLE(builder) == 0x3456789A);
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeFinalRoundMLE(builder);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzConsumeFinalRoundMLEs(uint256[] memory values) public {
        VerificationBuilder.Builder memory builder;
        uint256 valuesLength = values.length;
        builder.finalRoundMLEs = new uint256[](valuesLength);
        for (uint256 i = 0; i < valuesLength; ++i) {
            builder.finalRoundMLEs[i] = values[i];
        }
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(VerificationBuilder.__consumeFinalRoundMLE(builder) == values[i]);
        }
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeFinalRoundMLE(builder);
    }

    function testSetZeroChiEvaluations() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory values = new uint256[](0);
        VerificationBuilder.__setChiEvaluations(builder, values);
        assert(builder.chiEvaluations.length == 0);
    }

    function testSetChiEvaluations() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory values = new uint256[](3);
        values[0] = 0x12345678;
        values[1] = 0x23456789;
        values[2] = 0x3456789A;
        VerificationBuilder.__setChiEvaluations(builder, values);
        assert(builder.chiEvaluations.length == 3);
        assert(builder.chiEvaluations[0] == 0x12345678);
        assert(builder.chiEvaluations[1] == 0x23456789);
        assert(builder.chiEvaluations[2] == 0x3456789A);
    }

    function testFuzzSetChiEvaluations(uint256[] memory values) public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        VerificationBuilder.__setChiEvaluations(builder, values);
        assert(builder.chiEvaluations.length == values.length);
        uint256 valuesLength = values.length;
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(builder.chiEvaluations[i] == values[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testConsumeZeroChiEvaluations() public {
        VerificationBuilder.Builder memory builder;
        builder.chiEvaluations = new uint256[](0);
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeChiEvaluation(builder);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testConsumeChiEvaluations() public {
        VerificationBuilder.Builder memory builder;
        builder.chiEvaluations = new uint256[](3);
        builder.chiEvaluations[0] = 0x12345678;
        builder.chiEvaluations[1] = 0x23456789;
        builder.chiEvaluations[2] = 0x3456789A;
        assert(VerificationBuilder.__consumeChiEvaluation(builder) == 0x12345678);
        assert(VerificationBuilder.__consumeChiEvaluation(builder) == 0x23456789);
        assert(VerificationBuilder.__consumeChiEvaluation(builder) == 0x3456789A);
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeChiEvaluation(builder);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzConsumeChiEvaluations(uint256[] memory values) public {
        VerificationBuilder.Builder memory builder;
        uint256 valuesLength = values.length;
        builder.chiEvaluations = new uint256[](valuesLength);
        for (uint256 i = 0; i < valuesLength; ++i) {
            builder.chiEvaluations[i] = values[i];
        }
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(VerificationBuilder.__consumeChiEvaluation(builder) == values[i]);
        }
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeChiEvaluation(builder);
    }

    function testSetZeroRhoEvaluations() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory values = new uint256[](0);
        VerificationBuilder.__setRhoEvaluations(builder, values);
        assert(builder.rhoEvaluations.length == 0);
    }

    function testSetRhoEvaluations() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory values = new uint256[](3);
        values[0] = 0x12345678;
        values[1] = 0x23456789;
        values[2] = 0x3456789A;
        VerificationBuilder.__setRhoEvaluations(builder, values);
        assert(builder.rhoEvaluations.length == 3);
        assert(builder.rhoEvaluations[0] == 0x12345678);
        assert(builder.rhoEvaluations[1] == 0x23456789);
        assert(builder.rhoEvaluations[2] == 0x3456789A);
    }

    function testFuzzSetRhoEvaluations(uint256[] memory values) public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        VerificationBuilder.__setRhoEvaluations(builder, values);
        assert(builder.rhoEvaluations.length == values.length);
        uint256 valuesLength = values.length;
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(builder.rhoEvaluations[i] == values[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testConsumeZeroRhoEvaluations() public {
        VerificationBuilder.Builder memory builder;
        builder.rhoEvaluations = new uint256[](0);
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeRhoEvaluation(builder);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testConsumeRhoEvaluations() public {
        VerificationBuilder.Builder memory builder;
        builder.rhoEvaluations = new uint256[](3);
        builder.rhoEvaluations[0] = 0x12345678;
        builder.rhoEvaluations[1] = 0x23456789;
        builder.rhoEvaluations[2] = 0x3456789A;
        assert(VerificationBuilder.__consumeRhoEvaluation(builder) == 0x12345678);
        assert(VerificationBuilder.__consumeRhoEvaluation(builder) == 0x23456789);
        assert(VerificationBuilder.__consumeRhoEvaluation(builder) == 0x3456789A);
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeRhoEvaluation(builder);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzConsumeRhoEvaluations(uint256[] memory values) public {
        VerificationBuilder.Builder memory builder;
        uint256 valuesLength = values.length;
        builder.rhoEvaluations = new uint256[](valuesLength);
        for (uint256 i = 0; i < valuesLength; ++i) {
            builder.rhoEvaluations[i] = values[i];
        }
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(VerificationBuilder.__consumeRhoEvaluation(builder) == values[i]);
        }
        vm.expectRevert(Errors.EmptyQueue.selector);
        VerificationBuilder.__consumeRhoEvaluation(builder);
    }

    function testSetColumnEvaluations() public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        uint256[] memory values = new uint256[](3);
        values[0] = 0x12345678;
        values[1] = 0x23456789;
        values[2] = 0x3456789A;
        VerificationBuilder.__setColumnEvaluations(builder, values);
        assert(builder.columnEvaluations.length == 3);
        assert(builder.columnEvaluations[0] == 0x12345678);
        assert(builder.columnEvaluations[1] == 0x23456789);
        assert(builder.columnEvaluations[2] == 0x3456789A);
    }

    function testFuzzSetColumnEvaluations(uint256[] memory values) public pure {
        VerificationBuilder.Builder memory builder = VerificationBuilder.__builderNew();
        VerificationBuilder.__setColumnEvaluations(builder, values);
        assert(builder.columnEvaluations.length == values.length);
        uint256 valuesLength = values.length;
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(builder.columnEvaluations[i] == values[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testGetColumnEvaluationInvalidIndex() public {
        VerificationBuilder.Builder memory builder;
        uint256[] memory values = new uint256[](2);
        builder.columnEvaluations = values;
        vm.expectRevert(Errors.InvalidColumnIndex.selector);
        VerificationBuilder.__getColumnEvaluation(builder, 2);
    }

    function testGetColumnEvaluation() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory values = new uint256[](3);
        values[0] = 0x12345678;
        values[1] = 0x23456789;
        values[2] = 0x3456789A;
        builder.columnEvaluations = values;
        assert(VerificationBuilder.__getColumnEvaluation(builder, 0) == 0x12345678);
        assert(VerificationBuilder.__getColumnEvaluation(builder, 1) == 0x23456789);
        assert(VerificationBuilder.__getColumnEvaluation(builder, 2) == 0x3456789A);
        assert(VerificationBuilder.__getColumnEvaluation(builder, 2) == 0x3456789A);
        assert(VerificationBuilder.__getColumnEvaluation(builder, 0) == 0x12345678);
        assert(VerificationBuilder.__getColumnEvaluation(builder, 1) == 0x23456789);
    }

    function testFuzzGetColumnEvaluation(uint256[] memory values) public pure {
        vm.assume(values.length > 0);
        VerificationBuilder.Builder memory builder;
        builder.columnEvaluations = values;
        uint256 valuesLength = values.length;
        for (uint256 i = 0; i < valuesLength; ++i) {
            assert(VerificationBuilder.__getColumnEvaluation(builder, i) == values[i]);
        }
    }
}
