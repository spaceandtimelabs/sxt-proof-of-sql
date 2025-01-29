// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "./Constants.sol"; // solhint-disable-line no-global-import

library Transcript {
    function newTranscript(uint256 state_) public pure returns (uint256[1] memory transcriptPtr_) {
        assembly {
            function new_transcript(state) -> transcript_ptr {
                transcript_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(transcript_ptr, WORD_SIZE))
                mstore(transcript_ptr, state)
            }
            transcriptPtr_ := new_transcript(state_)
        }
    }

    function drawChallenge(uint256[1] memory transcriptPtr_) public pure returns (uint256 result_) {
        assembly {
            function draw_challenge(transcript_ptr) -> result {
                result := and(mload(transcript_ptr), MODULUS_MASK)
                mstore(transcript_ptr, keccak256(transcript_ptr, WORD_SIZE))
            }
            result_ := draw_challenge(transcriptPtr_)
        }
    }

    function drawChallenges(uint256[1] memory transcriptPtr_, uint256 count_)
        public
        pure
        returns (uint256 resultPtr_)
    {
        assembly {
            function draw_challenges(transcript_ptr, count) -> result_ptr {
                // allocate `count` words
                let free_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(free_ptr, shl(WORD_SHIFT, count)))
                // result is the pointer to the first word
                result_ptr := free_ptr
                // first challenge is the current transcript state
                let challenge := mload(transcript_ptr)
                for {} count {} {
                    mstore(transcript_ptr, challenge)

                    // store challenge in next word
                    mstore(free_ptr, and(challenge, MODULUS_MASK))
                    // hash challenge to get next challenge
                    challenge := keccak256(transcript_ptr, WORD_SIZE)
                    // increment to next word
                    free_ptr := add(free_ptr, WORD_SIZE)
                    // decrement count
                    count := sub(count, 1)
                }
                // The last (unused) challenge is the current state of the transcript
                mstore(transcript_ptr, challenge)
            }
            resultPtr_ := draw_challenges(transcriptPtr_, count_)
        }
    }

    function appendCalldata(uint256[1] memory transcriptPtr_, uint256 offset_, uint256 size_) public pure {
        assembly {
            function append_calldata(transcript_ptr, offset, size) {
                let free_ptr := mload(FREE_PTR)
                mstore(free_ptr, mload(transcript_ptr))
                calldatacopy(add(free_ptr, 0x20), offset, size)

                mstore(transcript_ptr, keccak256(free_ptr, add(size, 0x20)))
            }
            append_calldata(transcriptPtr_, offset_, size_)
        }
    }

    function testWeCanDrawChallenge() public pure {
        uint256[1] memory transcriptAPtr = newTranscript(12345);
        uint256[1] memory transcriptBPtr = newTranscript(6789);
        uint256 challengeA1 = drawChallenge(transcriptAPtr);
        uint256 challengeA2 = drawChallenge(transcriptAPtr);
        uint256 challengeB1 = drawChallenge(transcriptBPtr);
        uint256 challengeB2 = drawChallenge(transcriptBPtr);
        assert(challengeA1 == 12345);
        assert(challengeA1 != challengeA2);
        assert(challengeB1 == 6789);
        assert(challengeB1 != challengeB2);
        assert(challengeA1 != challengeB2);
        assert(challengeB1 != challengeA2);
        assert(challengeA2 != challengeB2);
    }

    function testWeCanDrawMultipleChallenges() public pure {
        uint256[1] memory transcriptAPtr = newTranscript(12345);
        uint256[1] memory transcriptBPtr = newTranscript(12345);
        uint256 challengeA1 = drawChallenge(transcriptAPtr);
        uint256 challengeA2 = drawChallenge(transcriptAPtr);
        uint256 challengeA3 = drawChallenge(transcriptAPtr);
        uint256 challengeA4 = drawChallenge(transcriptAPtr);

        uint256 resultBPtr = drawChallenges(transcriptBPtr, 4);
        uint256[4] memory resultB;
        assembly {
            resultB := resultBPtr
        }

        assert(challengeA1 == resultB[0]);
        assert(challengeA2 == resultB[1]);
        assert(challengeA3 == resultB[2]);
        assert(challengeA4 == resultB[3]);
    }

    function _testAppendCalldata(bytes calldata data) public pure {
        uint256[1] memory transcriptPtr =
            newTranscript(0x0123456789ABCDEF_0123456789ABCDEF_0123456789ABCDEF_0123456789ABCDEF);

        uint256 dataPtr;
        assembly {
            dataPtr := data.offset
        }
        appendCalldata(transcriptPtr, dataPtr, 4);

        uint256 state;
        assembly {
            state := mload(transcriptPtr)
        }

        uint256 expectedState = uint256(
            keccak256(
                hex"0123456789ABCDEF" hex"0123456789ABCDEF" hex"0123456789ABCDEF" hex"0123456789ABCDEF" hex"C001C0DE"
            )
        );
        assert(state == expectedState);
    }
}

contract TranscriptTest {
    function testAppendCalldata() public pure {
        Transcript._testAppendCalldata(hex"C001C0DE");
    }
}
