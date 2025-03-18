// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import "../../src/base/Errors.sol";
import {Sumcheck} from "../../src/proof/Sumcheck.pre.sol";
import {F, FF} from "../base/FieldUtil.sol";

contract SumcheckTest is Test {
    function testValidSumcheckProof() public {
        (bytes memory proofOut, uint256[] memory evaluationPoint, uint256 expectedEvaluation, uint256 degree) = Sumcheck
            .__verifySumcheckProof(
            [0x0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF],
            abi.encodePacked(
                uint64(9),
                [
                    4,
                    21888242871839275222246405745257275088548364400416034343698204186575808495613,
                    0,
                    8,
                    21888242871839275222246405745257275088548364400416034343698204186575808495609,
                    13482256827147415255560231043645370697866844653309320323738848927359006806087,
                    16,
                    21888242871839275222246405745257275088548364400416034343698204186575808495601,
                    176390704146465256270186756134066787497652218389263480241957748236428134315
                ]
            ),
            3
        );
        assert(proofOut.length == 0);
        assert(degree == 2);
        assert(evaluationPoint.length == 3);
        assert(evaluationPoint[0] == 6701067370494165868752845430639665078072713751969036977405358223929827170211);
        assert(evaluationPoint[1] == 21768314208790652753157741739615097287906179397339766596235077580228975452);
        assert(evaluationPoint[2] == 1183296664673662085780198666589920181052785655244323942254973446406613522512);
        assert(expectedEvaluation == 3849807423725722902263889445200117544239088196146083570560273561171320426291);
    }

    function testWeRevertWithInvalidSumcheckProof() public {
        vm.expectRevert(Errors.RoundEvaluationMismatch.selector);
        Sumcheck.__verifySumcheckProof(
            [0x0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF],
            abi.encodePacked(
                uint64(9),
                [
                    4,
                    21888242871839275222246405745257275088548364400416034343698204186575808495613,
                    1,
                    8,
                    21888242871839275222246405745257275088548364400416034343698204186575808495609,
                    13482256827147415255560231043645370697866844653309320323738848927359006806087,
                    16,
                    21888242871839275222246405745257275088548364400416034343698204186575808495601,
                    176390704146465256270186756134066787497652218389263480241957748236428134315
                ]
            ),
            3
        );
    }

    function testFuzzWeRevertWhenSumcheckProofIsFullyRandom(
        uint256[1] memory transcript,
        uint256[] memory proof,
        uint8 _numVars
    ) public {
        uint256 numVars = _numVars;
        // If numVars == 0, it is an empty proof and will always verify.
        vm.assume(numVars > 0);
        vm.assume(proof.length >= numVars); // solhint-disable-line gas-strict-inequalities
        uint256 degree = proof.length / numVars - 1;
        uint64 sumcheckLength = uint64(numVars * (degree + 1));

        // The proof with entirely 0s (expect for the last prover message) will succeed and is not fully random,
        // so we should not include it
        bool allZeros = true;
        for (uint256 i = 0; i < (numVars - 1) * (degree + 1); ++i) {
            allZeros = allZeros && (proof[i] == 0);
        }
        vm.assume(!allZeros);

        // any other proof will succeed with vanishingly low probability
        vm.expectRevert(Errors.RoundEvaluationMismatch.selector);
        Sumcheck.__verifySumcheckProof(transcript, abi.encodePacked(uint64(sumcheckLength), proof), numVars);
    }

    function testFuzzWeRevertWhenSumcheckProofIsWronglySized(
        uint256[1] memory transcript,
        uint256[] memory proof,
        uint8 numVars
    ) public {
        vm.assume(numVars == 0 || proof.length == 0 || proof.length % numVars != 0);
        vm.expectRevert(Errors.InvalidSumcheckProofSize.selector);
        Sumcheck.__verifySumcheckProof(transcript, abi.encodePacked(uint64(proof.length), proof), numVars);
    }

    /// With appropriate inputs, this test covers every possible valid proof
    function testFuzzWeCanVerifyValidProofWithRandomDimensionsAndData(
        uint256[1] memory transcript,
        uint256[] memory rand,
        uint8 _numVars,
        bytes memory trailingProof
    ) public {
        uint256 numVars = _numVars;
        vm.assume(numVars > 0);
        vm.assume(rand.length >= numVars); // solhint-disable-line gas-strict-inequalities
        uint256 degree = rand.length / numVars - 1;
        uint64 sumcheckLength = uint64(numVars * (degree + 1));

        uint256[] memory validProof = new uint256[](numVars * (degree + 1));
        uint256[] memory evalPoint = new uint256[](numVars);
        FF nextSum = F.ZERO;
        {
            uint256 transcriptP = uint256(keccak256(abi.encodePacked(transcript, sumcheckLength)));
            uint256 j = 0;
            for (uint256 i = 0; i < numVars; ++i) {
                uint256[] memory curRound = new uint256[](degree + 1);
                FF curSum = F.ZERO;
                for (uint256 d = 1; d < degree + 1; ++d) {
                    curRound[d] = rand[j];
                    curSum = curSum + F.from(curRound[d]);
                    ++j;
                }
                curSum = curSum + F.from(curRound[degree]);
                curRound[0] = (nextSum - curSum).into();
                transcriptP = uint256(keccak256(abi.encodePacked(transcriptP, curRound)));
                uint256 challenge = transcriptP & MODULUS_MASK;
                nextSum = F.ZERO;
                for (uint256 d = 0; d < degree + 1; ++d) {
                    validProof[i * (degree + 1) + d] = curRound[d];
                    nextSum = nextSum * F.from(challenge) + F.from(curRound[d]);
                }
                evalPoint[i] = challenge;
            }
        }

        (bytes memory proofOut, uint256[] memory evaluationPoint, uint256 expectedEvaluation, uint256 degreeOut) =
        Sumcheck.__verifySumcheckProof(transcript, abi.encodePacked(sumcheckLength, validProof, trailingProof), numVars);

        uint256 proofOutLength = proofOut.length;
        assert(proofOutLength == trailingProof.length);
        for (uint256 i = 0; i < proofOutLength; ++i) {
            assert(proofOut[i] == trailingProof[i]);
        }

        assert(degreeOut == degree);

        for (uint256 i = 0; i < numVars; ++i) {
            assert(evaluationPoint[i] == evalPoint[i]);
        }
        assert(expectedEvaluation == nextSum.into());
    }
}
