// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Sumcheck} from "./Sumcheck.sol";

library SumcheckTests {
    error EvaluationPointMismatch();
    error ExpectedEvaluationMismatch();
    error TranscriptMismatch();

    function rustTestVerification(
        bytes calldata rawProof,
        bytes32 transcript,
        uint16 degree,
        uint16 numVariables,
        uint256 claimedSum,
        uint256 modulus,
        bytes calldata rawExpectedResult,
        bytes32 expectedChallenge
    ) public pure {
        (Sumcheck.Proof memory proof) = abi.decode(rawProof, (Sumcheck.Proof));
        (Sumcheck.Subclaim memory expectedResult) = abi.decode(rawExpectedResult, (Sumcheck.Subclaim));
        (Sumcheck.Subclaim memory result, bytes32 resultTranscript) =
            Sumcheck.verifyWithoutEvaluation(proof, transcript, degree, numVariables, claimedSum, modulus);
        if (expectedResult.evaluationPoint.length != result.evaluationPoint.length) revert EvaluationPointMismatch();
        uint256 evaluationPointLength = expectedResult.evaluationPoint.length;
        for (uint256 i = 0; i < evaluationPointLength; ++i) {
            if (expectedResult.evaluationPoint[i] != result.evaluationPoint[i]) revert EvaluationPointMismatch();
        }
        if (expectedResult.expectedEvaluation != result.expectedEvaluation) revert ExpectedEvaluationMismatch();
        bytes32 challenge = keccak256(abi.encode(resultTranscript));
        if (expectedChallenge != challenge) revert TranscriptMismatch();
    }
}
