// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol"; // solhint-disable-line no-global-import
import {Transcript} from "../base/Transcript.sol";

library Sumcheck {
    function verifySumcheckProof(uint256[1] memory transcriptPtr_, uint256 proofPtr_, uint256 numVars_, uint256 degree_)
        public
        pure
        returns (uint256 evaluationPointPtr_, uint256 expectedEvaluation_)
    {
        assembly {
            // IMPORT-YUL ../base/Transcript.sol
            function append_calldata(transcript_ptr, offset, size) {
                revert(0, 0)
            }
            // IMPORT-YUL ../base/Transcript.sol
            function draw_challenge(transcript_ptr) -> result {
                revert(0, 0)
            }

            function verify_sumcheck_proof(transcript_ptr, proof_ptr, num_vars, degree) ->
                evaluation_point_ptr,
                expected_evaluation
            {
                mstore(mload(FREE_PTR), mload(transcript_ptr))
                mstore(add(mload(FREE_PTR), 0x20), or(shl(192, degree), shl(128, num_vars)))
                mstore(transcript_ptr, keccak256(mload(FREE_PTR), 0x30))

                expected_evaluation := 0
                evaluation_point_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(evaluation_point_ptr, shl(WORD_SHIFT, num_vars)))
                let evaluation_ptr := evaluation_point_ptr
                for {} num_vars { num_vars := sub(num_vars, 1) } {
                    append_calldata(transcript_ptr, proof_ptr, shl(WORD_SHIFT, add(degree, 1)))
                    let challenge := and(mload(transcript_ptr), MODULUS_MASK)
                    mstore(evaluation_ptr, challenge)
                    evaluation_ptr := add(evaluation_ptr, WORD_SIZE)
                    let coefficient := calldataload(proof_ptr)
                    proof_ptr := add(proof_ptr, WORD_SIZE)
                    let round_evaluation := coefficient
                    let actual_sum := coefficient
                    for { let d := degree } d { d := sub(d, 1) } {
                        coefficient := calldataload(proof_ptr)
                        proof_ptr := add(proof_ptr, WORD_SIZE)
                        round_evaluation := mulmod(round_evaluation, challenge, MODULUS)
                        round_evaluation := addmod(round_evaluation, coefficient, MODULUS)
                        actual_sum := addmod(actual_sum, coefficient, MODULUS)
                    }
                    actual_sum := addmod(actual_sum, coefficient, MODULUS)
                    if sub(expected_evaluation, actual_sum) {
                        mstore(0, ROUND_EVALUATION_MISMATCH)
                        revert(0, 4)
                    }
                    expected_evaluation := round_evaluation
                }
            }
            evaluationPointPtr_, expectedEvaluation_ :=
                verify_sumcheck_proof(transcriptPtr_, proofPtr_, numVars_, degree_)
        }
    }

    function _testProof(uint256[9] calldata proof) public pure {
        uint256 proofPtr;
        assembly {
            proofPtr := proof
        }
        uint256[1] memory transcriptPtr =
            Transcript.newTranscript(0x0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF);
        (uint256 evaluationPointPtr, uint256 expectedEvaluation) = verifySumcheckProof(transcriptPtr, proofPtr, 3, 2);
        uint256[3] memory evaluationPoint;
        assembly {
            evaluationPoint := evaluationPointPtr
        }
        assert(evaluationPoint[0] == 1506992429215810386281996950811506354401571025525967036054863434659872870360);
        assert(evaluationPoint[1] == 9120220574706222362812145529259689007317395757886066657835879616598334682445);
        assert(evaluationPoint[2] == 2292740780558994771787619692364445051029044725857360148809545662675414365470);
        assert(expectedEvaluation == 12163036463765362955288542932936666766809571097626128993008190096106665950029);
    }
}

contract SumcheckTest {
    function testProof() public pure {
        Sumcheck._testProof(
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
        );
    }
}
