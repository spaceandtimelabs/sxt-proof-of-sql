// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";

import {Sumcheck} from "../../src/proof/Sumcheck.pre.sol";
import {Errors, MODULUS, MODULUS_MASK, WORD_SIZE} from "../../src/base/Constants.sol";

library SumcheckTestWrapper {
    /// @notice Wrapper function to verify a sumcheck proof
    /// @dev This function is used to get the return values instead of a pointer
    /// @param transcript0 The initial transcript state
    /// @param proof0 The proof data in calldata
    /// @param numVars0 Number of variables in the sumcheck protocol
    /// @param degree0 Degree of the polynomial being checked
    /// @return evaluationPoint0 Array of evaluation points
    /// @return expectedEvaluation0 The expected evaluation result
    function verifySumcheckProof(
        uint256[1] calldata transcript0,
        bytes calldata proof0,
        uint256 numVars0,
        uint256 degree0
    ) external pure returns (uint256[] memory evaluationPoint0, uint256 expectedEvaluation0) {
        uint256 evaluationPointPtr0;
        uint256 proofPtr0;
        assembly {
            proofPtr0 := proof0.offset
        }
        (evaluationPointPtr0, expectedEvaluation0) =
            Sumcheck.verifySumcheckProof(transcript0, proofPtr0, numVars0, degree0);
        evaluationPoint0 = new uint256[](numVars0);
        for (uint256 i = 0; i < numVars0; ++i) {
            uint256 position = evaluationPointPtr0 + i * WORD_SIZE;
            uint256 value;
            assembly {
                value := mload(position)
            }
            evaluationPoint0[i] = value;
        }
    }
}

contract SumcheckTest is Test {
    function testValidSumcheckProof() public pure {
        SumcheckTestWrapper.verifySumcheckProof(
            [0x0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF],
            abi.encodePacked(
                [
                    202,
                    21888242871839275222246405745257275088548364400416034343698204186575808494599,
                    408,
                    0,
                    18915076809012878152013313149420939913818201595997650713446358010917568651211,
                    4821950864711900890543716581523492116861394263754152560342078214775139490553,
                    6328943293927711276825713532334998471262965289280119596396941649435012360810,
                    2386198368190398642909095286002342871574765628704748036160073239584938195323,
                    21405268855561213139254933443545185649703963970095593336009275275578128655038
                ]
            ),
            3,
            2
        );
    }

    function testWeRevertWithInvalidSumcheckProof() public {
        vm.expectRevert(Errors.RoundEvaluationMismatch.selector);
        SumcheckTestWrapper.verifySumcheckProof(
            [0x0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF],
            abi.encodePacked(
                [
                    202,
                    21888242871839275222246405745257275088548364400416034343698204186575808494599,
                    408,
                    1,
                    18915076809012878152013313149420939913818201595997650713446358010917568651211,
                    4821950864711900890543716581523492116861394263754152560342078214775139490553,
                    6328943293927711276825713532334998471262965289280119596396941649435012360810,
                    2386198368190398642909095286002342871574765628704748036160073239584938195323,
                    21405268855561213139254933443545185649703963970095593336009275275578128655038
                ]
            ),
            3,
            2
        );
    }

    function testFuzzWeRevertWhenSumcheckProofIsFullyRandom(
        uint256[1] memory transcript,
        uint256[] memory proof,
        uint8 numVars,
        uint8 degree
    ) public {
        // If numVars == 0, it is an empty proof and will always verify.
        vm.assume(numVars > 0);
        // The proof with entirely 0s will succeed and is not fully random, so we should not include it
        bool allZeros = true;
        uint256 proofLength = proof.length;
        for (uint256 i = 0; i < proofLength; ++i) {
            allZeros = allZeros && (proof[i] == 0);
        }
        vm.assume(!allZeros);

        // any other proof will succeed with vanishingly low probability
        vm.expectRevert(Errors.RoundEvaluationMismatch.selector);
        SumcheckTestWrapper.verifySumcheckProof(transcript, abi.encodePacked(proof), numVars, degree);
    }

    /// With appropriate inputs, this test covers every possible valid proof
    function testFuzzWeCanVerifyValidProofWithRandomDimensionsAndData(
        uint256[1] memory transcript,
        uint256[] memory rand,
        uint8 numVars0,
        uint8 degree0
    ) public pure {
        uint256 numVars = numVars0;
        uint256 degree = degree0;
        vm.assume(numVars * (degree + 1) < 1000);

        uint256 transcriptP = uint256(keccak256(abi.encodePacked(transcript, uint64(degree), uint64(numVars))));
        uint256[] memory validProof = new uint256[](numVars * (degree + 1));
        uint256[] memory evalPoint = new uint256[](numVars);

        uint256 nextSum = 0;
        uint256 j = 0;
        for (uint256 i = 0; i < numVars; ++i) {
            uint256[] memory curRound = new uint256[](degree + 1);
            uint256 curSum = 0;
            for (uint256 d = 1; d < degree + 1; ++d) {
                curRound[d] = j < rand.length ? rand[j] : 0;
                curSum = addmod(curSum, curRound[d], MODULUS);
                ++j;
            }
            curSum = addmod(curSum, curRound[degree], MODULUS);
            curRound[0] = addmod(nextSum, (MODULUS - (curSum % MODULUS)), MODULUS);
            transcriptP = uint256(keccak256(abi.encodePacked(transcriptP, curRound)));
            uint256 challenge = transcriptP & MODULUS_MASK;
            nextSum = 0;
            for (uint256 d = 0; d < degree + 1; ++d) {
                validProof[i * (degree + 1) + d] = curRound[d];
                nextSum = addmod(mulmod(nextSum, challenge, MODULUS), curRound[d], MODULUS);
            }
            evalPoint[i] = challenge;
        }

        (uint256[] memory evaluationPoint, uint256 expectedEvaluation) =
            SumcheckTestWrapper.verifySumcheckProof(transcript, abi.encodePacked(validProof), numVars, degree);

        for (uint256 i = 0; i < numVars; ++i) {
            assert(evaluationPoint[i] == evalPoint[i]);
        }
        assert(expectedEvaluation == nextSum);
    }
}
