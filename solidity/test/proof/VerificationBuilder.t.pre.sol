// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/proof/VerificationBuilder.pre.sol";

library VerificationBuilderTestHelper {
    function setChallenges(uint256 builderPtr, uint256[] memory challenges) internal pure {
        uint256 challengePtr;
        assembly {
            challengePtr := add(challenges, WORD_SIZE)
        }
        VerificationBuilder.__setChallenges(builderPtr, challengePtr, challenges.length);
    }

    function setFirstRoundMLEs(uint256 builderPtr, uint256[] memory firstRoundMLEs) internal pure {
        uint256 firstRoundMLEsPtr;
        assembly {
            firstRoundMLEsPtr := add(firstRoundMLEs, WORD_SIZE)
        }
        VerificationBuilder.__setFirstRoundMLEs(builderPtr, firstRoundMLEsPtr, firstRoundMLEs.length);
    }

    function setFinalRoundMLEs(uint256 builderPtr, uint256[] memory finalRoundMLEs) internal pure {
        uint256 finalRoundMLEsPtr;
        assembly {
            finalRoundMLEsPtr := add(finalRoundMLEs, WORD_SIZE)
        }
        VerificationBuilder.__setFinalRoundMLEs(builderPtr, finalRoundMLEsPtr, finalRoundMLEs.length);
    }

    function setChiEvaluations(uint256 builderPtr, uint256[] memory chiEvaluations) internal pure {
        uint256 chiEvaluationsPtr;
        assembly {
            chiEvaluationsPtr := add(chiEvaluations, WORD_SIZE)
        }
        VerificationBuilder.__setChiEvaluations(builderPtr, chiEvaluationsPtr, chiEvaluations.length);
    }

    function setRhoEvaluations(uint256 builderPtr, uint256[] memory rhoEvaluations) internal pure {
        uint256 rhoEvaluationsPtr;
        assembly {
            rhoEvaluationsPtr := add(rhoEvaluations, WORD_SIZE)
        }
        VerificationBuilder.__setRhoEvaluations(builderPtr, rhoEvaluationsPtr, rhoEvaluations.length);
    }
}

contract VerificationBuilderTest is Test {
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

    function testSetChallenges() public pure {
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilder.__setChallenges(builderPtr, 0xABCD, 0x1234);
        uint256 head;
        uint256 tail;
        assembly {
            head := mload(add(builderPtr, CHALLENGE_HEAD_OFFSET))
            tail := mload(add(builderPtr, CHALLENGE_TAIL_OFFSET))
        }
        assert(head == 0xABCD);
        assert(tail == 0xABCD + WORD_SIZE * 0x1234);
    }

    function testFuzzSetChallenges(uint256[] memory, uint256 challengePtr, uint64 challengeLength) public pure {
        vm.assume(challengePtr < 2 ** 64);
        vm.assume(challengeLength < 2 ** 64);
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilder.__setChallenges(builderPtr, challengePtr, challengeLength);
        uint256 head;
        uint256 tail;
        assembly {
            head := mload(add(builderPtr, CHALLENGE_HEAD_OFFSET))
            tail := mload(add(builderPtr, CHALLENGE_TAIL_OFFSET))
        }
        assert(head == challengePtr);
        assert(tail == challengePtr + WORD_SIZE * challengeLength);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeZeroChallenges() public {
        uint256[] memory challenges = new uint256[](0);
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setChallenges(builderPtr, challenges);
        vm.expectRevert(Errors.TooFewChallenges.selector);
        VerificationBuilder.__consumeChallenge(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeOneChallenge() public {
        uint256[] memory challenges = new uint256[](1);
        challenges[0] = 0x12345678;
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setChallenges(builderPtr, challenges);
        assert(VerificationBuilder.__consumeChallenge(builderPtr) == 0x12345678);
        vm.expectRevert(Errors.TooFewChallenges.selector);
        VerificationBuilder.__consumeChallenge(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeChallenges() public {
        uint256[] memory challenges = new uint256[](3);
        challenges[0] = 0x12345678;
        challenges[1] = 0x23456789;
        challenges[2] = 0x3456789A;
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setChallenges(builderPtr, challenges);
        assert(VerificationBuilder.__consumeChallenge(builderPtr) == 0x12345678);
        assert(VerificationBuilder.__consumeChallenge(builderPtr) == 0x23456789);
        assert(VerificationBuilder.__consumeChallenge(builderPtr) == 0x3456789A);
        vm.expectRevert(Errors.TooFewChallenges.selector);
        VerificationBuilder.__consumeChallenge(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzSetAndConsumeChallenges(uint256[] memory, uint256[] memory challenges) public {
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setChallenges(builderPtr, challenges);
        uint256 challengesLength = challenges.length;
        for (uint256 i = 0; i < challengesLength; ++i) {
            assert(VerificationBuilder.__consumeChallenge(builderPtr) == challenges[i]);
        }
        vm.expectRevert(Errors.TooFewChallenges.selector);
        VerificationBuilder.__consumeChallenge(builderPtr);
    }

    function testSetFirstRoundMLEs() public pure {
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilder.__setFirstRoundMLEs(builderPtr, 0xABCD, 0x1234);
        uint256 head;
        uint256 tail;
        assembly {
            head := mload(add(builderPtr, FIRST_ROUND_MLE_HEAD_OFFSET))
            tail := mload(add(builderPtr, FIRST_ROUND_MLE_TAIL_OFFSET))
        }
        assert(head == 0xABCD);
        assert(tail == 0xABCD + WORD_SIZE * 0x1234);
    }

    function testFuzzSetFirstRoundMLEs(uint256[] memory, uint256 firstRoundMLEPtr, uint64 firstRoundMLELength)
        public
        pure
    {
        vm.assume(firstRoundMLEPtr < 2 ** 64);
        vm.assume(firstRoundMLELength < 2 ** 64);
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilder.__setFirstRoundMLEs(builderPtr, firstRoundMLEPtr, firstRoundMLELength);
        uint256 head;
        uint256 tail;
        assembly {
            head := mload(add(builderPtr, FIRST_ROUND_MLE_HEAD_OFFSET))
            tail := mload(add(builderPtr, FIRST_ROUND_MLE_TAIL_OFFSET))
        }
        assert(head == firstRoundMLEPtr);
        assert(tail == firstRoundMLEPtr + WORD_SIZE * firstRoundMLELength);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeZeroFirstRoundMLEs() public {
        uint256[] memory firstRoundMLEs = new uint256[](0);
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setFirstRoundMLEs(builderPtr, firstRoundMLEs);
        vm.expectRevert(Errors.TooFewFirstRoundMLEs.selector);
        VerificationBuilder.__consumeFirstRoundMLE(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeOneFirstRoundMLE() public {
        uint256[] memory firstRoundMLEs = new uint256[](1);
        firstRoundMLEs[0] = 0x12345678;
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setFirstRoundMLEs(builderPtr, firstRoundMLEs);
        assert(VerificationBuilder.__consumeFirstRoundMLE(builderPtr) == 0x12345678);
        vm.expectRevert(Errors.TooFewFirstRoundMLEs.selector);
        VerificationBuilder.__consumeFirstRoundMLE(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeFirstRoundMLEs() public {
        uint256[] memory firstRoundMLEs = new uint256[](3);
        firstRoundMLEs[0] = 0x12345678;
        firstRoundMLEs[1] = 0x23456789;
        firstRoundMLEs[2] = 0x3456789A;
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setFirstRoundMLEs(builderPtr, firstRoundMLEs);
        assert(VerificationBuilder.__consumeFirstRoundMLE(builderPtr) == 0x12345678);
        assert(VerificationBuilder.__consumeFirstRoundMLE(builderPtr) == 0x23456789);
        assert(VerificationBuilder.__consumeFirstRoundMLE(builderPtr) == 0x3456789A);
        vm.expectRevert(Errors.TooFewFirstRoundMLEs.selector);
        VerificationBuilder.__consumeFirstRoundMLE(builderPtr);
    }

    function testSetFinalRoundMLEs() public pure {
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilder.__setFinalRoundMLEs(builderPtr, 0xABCD, 0x1234);
        uint256 head;
        uint256 tail;
        assembly {
            head := mload(add(builderPtr, FINAL_ROUND_MLE_HEAD_OFFSET))
            tail := mload(add(builderPtr, FINAL_ROUND_MLE_TAIL_OFFSET))
        }
        assert(head == 0xABCD);
        assert(tail == 0xABCD + WORD_SIZE * 0x1234);
    }

    function testFuzzSetFinalRoundMLEs(uint256[] memory, uint256 finalRoundMLEPtr, uint64 finalRoundMLELength)
        public
        pure
    {
        vm.assume(finalRoundMLEPtr < 2 ** 64);
        vm.assume(finalRoundMLELength < 2 ** 64);
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilder.__setFinalRoundMLEs(builderPtr, finalRoundMLEPtr, finalRoundMLELength);
        uint256 head;
        uint256 tail;
        assembly {
            head := mload(add(builderPtr, FINAL_ROUND_MLE_HEAD_OFFSET))
            tail := mload(add(builderPtr, FINAL_ROUND_MLE_TAIL_OFFSET))
        }
        assert(head == finalRoundMLEPtr);
        assert(tail == finalRoundMLEPtr + WORD_SIZE * finalRoundMLELength);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeZeroFinalRoundMLEs() public {
        uint256[] memory finalRoundMLEs = new uint256[](0);
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setFinalRoundMLEs(builderPtr, finalRoundMLEs);
        vm.expectRevert(Errors.TooFewFinalRoundMLEs.selector);
        VerificationBuilder.__consumeFinalRoundMLE(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeOneFinalRoundMLE() public {
        uint256[] memory finalRoundMLEs = new uint256[](1);
        finalRoundMLEs[0] = 0x12345678;
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setFinalRoundMLEs(builderPtr, finalRoundMLEs);
        assert(VerificationBuilder.__consumeFinalRoundMLE(builderPtr) == 0x12345678);
        vm.expectRevert(Errors.TooFewFinalRoundMLEs.selector);
        VerificationBuilder.__consumeFinalRoundMLE(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeFinalRoundMLEs() public {
        uint256[] memory finalRoundMLEs = new uint256[](3);
        finalRoundMLEs[0] = 0x12345678;
        finalRoundMLEs[1] = 0x23456789;
        finalRoundMLEs[2] = 0x3456789A;
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setFinalRoundMLEs(builderPtr, finalRoundMLEs);
        assert(VerificationBuilder.__consumeFinalRoundMLE(builderPtr) == 0x12345678);
        assert(VerificationBuilder.__consumeFinalRoundMLE(builderPtr) == 0x23456789);
        assert(VerificationBuilder.__consumeFinalRoundMLE(builderPtr) == 0x3456789A);
        vm.expectRevert(Errors.TooFewFinalRoundMLEs.selector);
        VerificationBuilder.__consumeFinalRoundMLE(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzSetAndConsumeFinalRoundMLEs(uint256[] memory, uint256[] memory finalRoundMLEs) public {
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setFinalRoundMLEs(builderPtr, finalRoundMLEs);
        uint256 finalRoundMLEsLength = finalRoundMLEs.length;
        for (uint256 i = 0; i < finalRoundMLEsLength; ++i) {
            assert(VerificationBuilder.__consumeFinalRoundMLE(builderPtr) == finalRoundMLEs[i]);
        }
        vm.expectRevert(Errors.TooFewFinalRoundMLEs.selector);
        VerificationBuilder.__consumeFinalRoundMLE(builderPtr);
    }

    function testSetChiEvaluations() public pure {
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilder.__setChiEvaluations(builderPtr, 0xABCD, 0x1234);
        uint256 head;
        uint256 tail;
        assembly {
            head := mload(add(builderPtr, CHI_EVALUATION_HEAD_OFFSET))
            tail := mload(add(builderPtr, CHI_EVALUATION_TAIL_OFFSET))
        }
        assert(head == 0xABCD);
        assert(tail == 0xABCD + WORD_SIZE * 0x1234);
    }

    function testFuzzSetChiEvaluations(uint256[] memory, uint256 chiEvaluationPtr, uint64 chiEvaluationLength)
        public
        pure
    {
        vm.assume(chiEvaluationPtr < 2 ** 64);
        vm.assume(chiEvaluationLength < 2 ** 64);
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilder.__setChiEvaluations(builderPtr, chiEvaluationPtr, chiEvaluationLength);
        uint256 head;
        uint256 tail;
        assembly {
            head := mload(add(builderPtr, CHI_EVALUATION_HEAD_OFFSET))
            tail := mload(add(builderPtr, CHI_EVALUATION_TAIL_OFFSET))
        }
        assert(head == chiEvaluationPtr);
        assert(tail == chiEvaluationPtr + WORD_SIZE * chiEvaluationLength);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeZeroChiEvaluations() public {
        uint256[] memory chiEvaluations = new uint256[](0);
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setChiEvaluations(builderPtr, chiEvaluations);
        vm.expectRevert(Errors.TooFewChiEvaluations.selector);
        VerificationBuilder.__consumeChiEvaluation(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeOneChiEvaluation() public {
        uint256[] memory chiEvaluations = new uint256[](1);
        chiEvaluations[0] = 0x12345678;
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setChiEvaluations(builderPtr, chiEvaluations);
        assert(VerificationBuilder.__consumeChiEvaluation(builderPtr) == 0x12345678);
        vm.expectRevert(Errors.TooFewChiEvaluations.selector);
        VerificationBuilder.__consumeChiEvaluation(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeChiEvaluations() public {
        uint256[] memory chiEvaluations = new uint256[](3);
        chiEvaluations[0] = 0x12345678;
        chiEvaluations[1] = 0x23456789;
        chiEvaluations[2] = 0x3456789A;
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setChiEvaluations(builderPtr, chiEvaluations);
        assert(VerificationBuilder.__consumeChiEvaluation(builderPtr) == 0x12345678);
        assert(VerificationBuilder.__consumeChiEvaluation(builderPtr) == 0x23456789);
        assert(VerificationBuilder.__consumeChiEvaluation(builderPtr) == 0x3456789A);
        vm.expectRevert(Errors.TooFewChiEvaluations.selector);
        VerificationBuilder.__consumeChiEvaluation(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzSetAndConsumeChiEvaluations(uint256[] memory, uint256[] memory chiEvaluations) public {
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setChiEvaluations(builderPtr, chiEvaluations);
        uint256 chiEvaluationsLength = chiEvaluations.length;
        for (uint256 i = 0; i < chiEvaluationsLength; ++i) {
            assert(VerificationBuilder.__consumeChiEvaluation(builderPtr) == chiEvaluations[i]);
        }
        vm.expectRevert(Errors.TooFewChiEvaluations.selector);
        VerificationBuilder.__consumeChiEvaluation(builderPtr);
    }

    function testSetRhoEvaluations() public pure {
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilder.__setRhoEvaluations(builderPtr, 0xABCD, 0x1234);
        uint256 head;
        uint256 tail;
        assembly {
            head := mload(add(builderPtr, RHO_EVALUATION_HEAD_OFFSET))
            tail := mload(add(builderPtr, RHO_EVALUATION_TAIL_OFFSET))
        }
        assert(head == 0xABCD);
        assert(tail == 0xABCD + WORD_SIZE * 0x1234);
    }

    function testFuzzSetRhoEvaluations(uint256[] memory, uint256 rhoEvaluationPtr, uint64 rhoEvaluationLength)
        public
        pure
    {
        vm.assume(rhoEvaluationPtr < 2 ** 64);
        vm.assume(rhoEvaluationLength < 2 ** 64);
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilder.__setRhoEvaluations(builderPtr, rhoEvaluationPtr, rhoEvaluationLength);
        uint256 head;
        uint256 tail;
        assembly {
            head := mload(add(builderPtr, RHO_EVALUATION_HEAD_OFFSET))
            tail := mload(add(builderPtr, RHO_EVALUATION_TAIL_OFFSET))
        }
        assert(head == rhoEvaluationPtr);
        assert(tail == rhoEvaluationPtr + WORD_SIZE * rhoEvaluationLength);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeZeroRhoEvaluations() public {
        uint256[] memory rhoEvaluations = new uint256[](0);
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setRhoEvaluations(builderPtr, rhoEvaluations);
        vm.expectRevert(Errors.TooFewRhoEvaluations.selector);
        VerificationBuilder.__consumeRhoEvaluation(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeOneRhoEvaluation() public {
        uint256[] memory rhoEvaluations = new uint256[](1);
        rhoEvaluations[0] = 0x12345678;
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setRhoEvaluations(builderPtr, rhoEvaluations);
        assert(VerificationBuilder.__consumeRhoEvaluation(builderPtr) == 0x12345678);
        vm.expectRevert(Errors.TooFewRhoEvaluations.selector);
        VerificationBuilder.__consumeRhoEvaluation(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testSetAndConsumeRhoEvaluations() public {
        uint256[] memory rhoEvaluations = new uint256[](3);
        rhoEvaluations[0] = 0x12345678;
        rhoEvaluations[1] = 0x23456789;
        rhoEvaluations[2] = 0x3456789A;
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setRhoEvaluations(builderPtr, rhoEvaluations);
        assert(VerificationBuilder.__consumeRhoEvaluation(builderPtr) == 0x12345678);
        assert(VerificationBuilder.__consumeRhoEvaluation(builderPtr) == 0x23456789);
        assert(VerificationBuilder.__consumeRhoEvaluation(builderPtr) == 0x3456789A);
        vm.expectRevert(Errors.TooFewRhoEvaluations.selector);
        VerificationBuilder.__consumeRhoEvaluation(builderPtr);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzSetAndConsumeRhoEvaluations(uint256[] memory, uint256[] memory rhoEvaluations) public {
        uint256 builderPtr = VerificationBuilder.__allocate();
        VerificationBuilderTestHelper.setRhoEvaluations(builderPtr, rhoEvaluations);
        uint256 rhoEvaluationsLength = rhoEvaluations.length;
        for (uint256 i = 0; i < rhoEvaluationsLength; ++i) {
            assert(VerificationBuilder.__consumeRhoEvaluation(builderPtr) == rhoEvaluations[i]);
        }
        vm.expectRevert(Errors.TooFewRhoEvaluations.selector);
        VerificationBuilder.__consumeRhoEvaluation(builderPtr);
    }
}
