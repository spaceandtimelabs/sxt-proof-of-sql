// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

library Sumcheck {
    error InvalidProofLength();
    error RoundEvaluationMismatch();

    struct Proof {
        uint256[] coefficients;
    }

    struct Subclaim {
        uint256[] evaluationPoint;
        uint256 expectedEvaluation;
    }

    function verifyWithoutEvaluation(
        Proof calldata proof,
        bytes32 transcript,
        uint64 degree,
        uint64 numVariables,
        uint256 claimedSum,
        uint256 modulus
    ) public pure returns (Subclaim memory result, bytes32 resultTranscript) {
        if (proof.coefficients.length != numVariables * (degree + 1)) revert InvalidProofLength();
        result.evaluationPoint = new uint256[](numVariables);
        transcript = keccak256(abi.encodePacked(transcript, degree, numVariables));
        for (uint64 roundIndex = 0; roundIndex < numVariables; ++roundIndex) {
            uint256 startIndex = roundIndex * (degree + 1);
            transcript = keccak256(abi.encodePacked(transcript, proof.coefficients[startIndex:startIndex + degree + 1]));
            uint256 roundEvaluationPoint = uint256(transcript);
            result.evaluationPoint[roundIndex] = roundEvaluationPoint % modulus;
            uint256 roundEvaluation = proof.coefficients[startIndex];
            uint256 actualSum = addmod(roundEvaluation, proof.coefficients[startIndex + degree], modulus);
            for (
                uint256 coefficientIndex = startIndex + 1;
                coefficientIndex < startIndex + degree + 1;
                ++coefficientIndex
            ) {
                roundEvaluation = addmod(
                    mulmod(roundEvaluation, roundEvaluationPoint, modulus),
                    proof.coefficients[coefficientIndex],
                    modulus
                );
                actualSum = addmod(actualSum, proof.coefficients[coefficientIndex], modulus);
            }
            if (actualSum != claimedSum) revert RoundEvaluationMismatch();
            claimedSum = roundEvaluation;
        }
        result.expectedEvaluation = claimedSum;
        resultTranscript = transcript;
    }
}
