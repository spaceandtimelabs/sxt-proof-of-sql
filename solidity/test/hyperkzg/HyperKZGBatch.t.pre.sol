// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Errors.sol";
import {Transcript} from "../../src/base/Transcript.sol";
import {HyperKZGBatch} from "../../src/hyperkzg/HyperKZGBatch.pre.sol";
import {ECPrecompilesTestHelper} from "../base/ECPrecompiles.t.pre.sol";
import {F, FF} from "../base/FieldUtil.sol";

contract HyperKZGBatchTest is Test {
    function testBatchHyperKZG() public view {
        uint256 startingState = 0xabcdef;

        uint256[] memory challenges = Transcript.__drawChallenges([startingState], 2);

        uint256[5] memory args;
        uint256 argPow = 101;
        (args[0], args[1]) = ECPrecompilesTestHelper.ecBasePower(argPow);

        uint256[] memory commitmentPows = new uint256[](2);
        commitmentPows[0] = 0;
        commitmentPows[1] = 0;
        uint256[] memory commitments = new uint256[](4);
        (commitments[0], commitments[1]) = ECPrecompilesTestHelper.ecBasePower(commitmentPows[0]);
        (commitments[2], commitments[3]) = ECPrecompilesTestHelper.ecBasePower(commitmentPows[1]);

        uint256 batchEval = 201;
        uint256[] memory evaluations = new uint256[](2);
        evaluations[0] = 202;
        evaluations[1] = 203;

        FF expectedBatchEval = F.from(batchEval);
        expectedBatchEval = expectedBatchEval + F.from(evaluations[0]) * F.from(challenges[0]);
        expectedBatchEval = expectedBatchEval + F.from(evaluations[1]) * F.from(challenges[1]);
        FF expectedBatchCommitmentPow = F.from(argPow);
        expectedBatchCommitmentPow = expectedBatchCommitmentPow + F.from(commitmentPows[0]) * F.from(challenges[0]);
        expectedBatchCommitmentPow = expectedBatchCommitmentPow + F.from(commitmentPows[1]) * F.from(challenges[1]);
        (uint256 expectedX, uint256 expectedY) = ECPrecompilesTestHelper.ecBasePower(expectedBatchCommitmentPow.into());

        (batchEval, args) = HyperKZGBatch.__batchPCS({
            __args: args,
            __transcript: [startingState],
            __commitments: commitments,
            __evaluations: evaluations,
            __batchEval: batchEval
        });

        assert(batchEval == expectedBatchEval.into());
        assert(args[0] == expectedX);
        assert(args[1] == expectedY);
    }

    function testFuzzBatchHyperKZG(
        uint256 initBatchEval,
        uint256 initCommitmentPow,
        uint256[] memory commitmentPows,
        uint256[] memory evaluations,
        uint256 transcriptState
    ) public view {
        uint256[] memory commitments;
        FF expectedBatchEval;
        uint256 expectedX;
        uint256 expectedY;

        // generate commitments and expected values
        {
            uint256 batchLength = evaluations.length;
            // solhint-disable-next-line gas-strict-inequalities
            vm.assume(commitmentPows.length >= batchLength);
            commitments = new uint256[](2 * batchLength);

            expectedBatchEval = F.from(initBatchEval);
            FF expectedCommitmentPow = F.from(initCommitmentPow);
            uint256[] memory challenges = Transcript.__drawChallenges([transcriptState], batchLength);

            for (uint256 i = 0; i < batchLength; ++i) {
                (commitments[2 * i], commitments[2 * i + 1]) = ECPrecompilesTestHelper.ecBasePower(commitmentPows[i]);

                expectedBatchEval = expectedBatchEval + F.from(evaluations[i]) * F.from(challenges[i]);
                expectedCommitmentPow = expectedCommitmentPow + F.from(commitmentPows[i]) * F.from(challenges[i]);
            }
            (expectedX, expectedY) = ECPrecompilesTestHelper.ecBasePower(expectedCommitmentPow.into());
        }

        uint256[5] memory args;
        (args[0], args[1]) = ECPrecompilesTestHelper.ecBasePower(initCommitmentPow);
        args[2] = 0xDEAD_1234;
        args[3] = 0xDEAD_5678;
        args[4] = 0xDEAD_9ABC;

        uint256 batchEval;
        (batchEval, args) = HyperKZGBatch.__batchPCS({
            __args: args,
            __transcript: [transcriptState],
            __commitments: commitments,
            __evaluations: evaluations,
            __batchEval: initBatchEval
        });

        assert(batchEval == expectedBatchEval.into());
        assert(args[0] == expectedX);
        assert(args[1] == expectedY);
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testFuzzBatchHyperKZGFailsOnLengthMismatch(
        uint256[5] memory args,
        uint256[1] memory transcript,
        uint256[] memory commitments,
        uint256[] memory evaluations,
        uint256 batchEval
    ) public {
        vm.assume(commitments.length / 2 != evaluations.length);
        vm.expectRevert(Errors.PCSBatchLengthMismatch.selector);
        HyperKZGBatch.__batchPCS({
            __args: args,
            __transcript: transcript,
            __commitments: commitments,
            __evaluations: evaluations,
            __batchEval: batchEval
        });
    }
}
