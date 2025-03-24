// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../../src/base/Constants.sol";
import {Transcript} from "../../src/base/Transcript.sol";

library TranscriptTest {
    function testWeCanDrawChallenge() public pure {
        uint256[1] memory transcriptAPtr = [uint256(12345)];
        uint256[1] memory transcriptBPtr = [uint256(6789)];
        uint256 challengeA1 = Transcript.__drawChallenge(transcriptAPtr);
        uint256 challengeA2 = Transcript.__drawChallenge(transcriptAPtr);
        uint256 challengeB1 = Transcript.__drawChallenge(transcriptBPtr);
        uint256 challengeB2 = Transcript.__drawChallenge(transcriptBPtr);
        assert(challengeA1 == 12345);
        assert(challengeA1 != challengeA2);
        assert(challengeB1 == 6789);
        assert(challengeB1 != challengeB2);
        assert(challengeA1 != challengeB2);
        assert(challengeB1 != challengeA2);
        assert(challengeA2 != challengeB2);
        assert(challengeA1 < MODULUS);
        assert(challengeA2 < MODULUS);
        assert(challengeB1 < MODULUS);
        assert(challengeB2 < MODULUS);
    }

    function testWeCanDrawMultipleChallenges() public pure {
        uint256[1] memory transcriptAPtr = [uint256(12345)];
        uint256[1] memory transcriptBPtr = [uint256(12345)];
        uint256 challengeA1 = Transcript.__drawChallenge(transcriptAPtr);
        uint256 challengeA2 = Transcript.__drawChallenge(transcriptAPtr);
        uint256 challengeA3 = Transcript.__drawChallenge(transcriptAPtr);
        uint256 challengeA4 = Transcript.__drawChallenge(transcriptAPtr);

        uint256[] memory resultB = Transcript.__drawChallenges(transcriptBPtr, 4);

        assert(resultB.length == 4);
        assert(challengeA1 == resultB[0]);
        assert(challengeA2 == resultB[1]);
        assert(challengeA3 == resultB[2]);
        assert(challengeA4 == resultB[3]);
        assert(challengeA1 < MODULUS);
        assert(challengeA2 < MODULUS);
        assert(challengeA3 < MODULUS);
        assert(challengeA4 < MODULUS);
    }

    function testFuzzTwoTranscriptsGiveDifferentChallengesUnlessTheyAreTheSame(
        uint256[1] memory transcriptA,
        uint256[1] memory transcriptB
    ) public pure {
        uint256 challengeA1 = Transcript.__drawChallenge(transcriptA);
        uint256 challengeB1 = Transcript.__drawChallenge(transcriptB);
        uint256 challengeA2 = Transcript.__drawChallenge(transcriptA);
        uint256 challengeB2 = Transcript.__drawChallenge(transcriptB);
        if (transcriptA[0] == transcriptB[0]) {
            assert(challengeA1 == challengeB1);
            assert(challengeA2 == challengeB2);
        } else {
            assert(challengeA1 != challengeB1);
            assert(challengeA2 != challengeB2);
        }
        assert(challengeA1 != challengeA2);
        assert(challengeA1 != challengeB2);
        assert(challengeA2 != challengeB1);
        assert(challengeB1 != challengeB2);
        assert(challengeA1 < MODULUS);
        assert(challengeA2 < MODULUS);
        assert(challengeB1 < MODULUS);
        assert(challengeB2 < MODULUS);
    }

    function testFuzzSameTranscriptsGiveSameChallenges(uint256 state) public pure {
        testFuzzTwoTranscriptsGiveDifferentChallengesUnlessTheyAreTheSame([state], [state]);
    }

    function testFuzzDrawingMultipleChallengesGivesTheSameChallengesAsIndividually(uint256 state, uint8 count)
        public
        pure
    {
        uint256[1] memory transcriptA = [state];
        uint256[1] memory transcriptB = [state];
        uint256 freePtrBefore;
        assembly {
            freePtrBefore := mload(FREE_PTR)
        }
        uint256[] memory challenges = Transcript.__drawChallenges(transcriptA, count);
        uint256 freePtrAfter;
        assembly {
            freePtrAfter := mload(FREE_PTR)
        }
        assert(freePtrBefore + (uint256(count) + 1) * WORD_SIZE == freePtrAfter);
        for (uint256 i = 0; i < count; ++i) {
            uint256 challenge = Transcript.__drawChallenge(transcriptB);
            assert(challenge == challenges[i]);
            assert(challenge < MODULUS);
        }
    }

    function testAppendCalldata() public pure {
        uint256[1] memory state = Transcript.__appendCalldata(
            [0x0123456789ABCDEF_0123456789ABCDEF_0123456789ABCDEF_0123456789ABCDEF], hex"C001C0DE"
        );
        uint256 expectedState = uint256(
            keccak256(
                hex"0123456789ABCDEF" hex"0123456789ABCDEF" hex"0123456789ABCDEF" hex"0123456789ABCDEF" hex"C001C0DE"
            )
        );
        assert(state[0] == expectedState);
    }

    function testFuzzAppendCalldata(uint256 start, bytes calldata data) public pure {
        uint256[1] memory state = Transcript.__appendCalldata([start], data);
        uint256 expectedState = uint256(keccak256(abi.encodePacked(start, data)));
        assert(state[0] == expectedState);
    }

    function testAppendArray() public pure {
        uint256[1] memory transcript = [uint256(0x123)];
        uint256[] memory data = new uint256[](3);
        data[0] = 0xabc;
        data[1] = 0xdef;
        data[2] = 0x789;
        uint256 expectedState = uint256(keccak256(abi.encodePacked(transcript, data)));

        transcript = Transcript.__appendArray(transcript, data);

        // Verify data is preserved
        assert(data.length == 3);
        assert(data[0] == 0xabc);
        assert(data[1] == 0xdef);
        assert(data[2] == 0x789);

        // Verify state changed
        assert(transcript[0] == expectedState);
    }

    function testFuzzAppendArray(uint256[1] memory transcript, uint256[] memory array) public pure {
        uint256 len = array.length;
        uint256[] memory originalArray = array;

        uint256 expectedState = uint256(keccak256(abi.encodePacked(transcript, array)));
        transcript = Transcript.__appendArray(transcript, array);

        // Verify data preserved
        assert(array.length == len);
        for (uint256 i = 0; i < len; ++i) {
            assert(array[i] == originalArray[i]);
        }
        assert(transcript[0] == expectedState);
    }
}
