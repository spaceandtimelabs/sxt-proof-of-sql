// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

// assembly only constants
// solhint-disable-next-line no-unused-import
import {FREE_PTR, MODULUS_MASK, WORD_SIZE} from "./Constants.sol";

/// @title Transcript Library
/// @notice Provides functions to manage a simple public coin transcript
/// @dev The transcript is a sequence of messages and challenges, where each challenge is a scalar (less than MODULUS).
library Transcript {
    /// @notice Draw a challenge from the transcript, and update the state of the transcript.
    /// @dev The result should be a scalar value (less than MODULUS).
    /// The challenges is a masked version of the current state of the transcript.
    /// The new state of the transcript is the hash of the current state.
    /// @param transcript0 The current state of the transcript
    /// @return result0 The drawn challenge
    function drawChallenge(uint256[1] memory transcript0) internal pure returns (uint256 result0) {
        assembly {
            function draw_challenge(transcript_ptr) -> result {
                result := and(mload(transcript_ptr), MODULUS_MASK)
                mstore(transcript_ptr, keccak256(transcript_ptr, WORD_SIZE))
            }
            result0 := draw_challenge(transcript0)
        }
    }

    /// @notice Draw multiple challenges from the transcript, and update the state of the transcript.
    /// @dev This is equivalent to calling `drawChallenge` multiple times.
    /// The returned value is a pointer to newly allocated memory containing the challenges.
    /// The first entry is NOT the length of the results. That is count.
    /// @param transcript0 The current state of the transcript
    /// @param count0 The number of challenges to draw
    /// @return resultPtr0 A pointer to the memory containing the drawn challenges
    function drawChallenges(uint256[1] memory transcript0, uint256 count0) internal pure returns (uint256 resultPtr0) {
        assembly {
            function draw_challenges(transcript_ptr, count) -> result_ptr {
                // allocate `count` words
                let free_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(free_ptr, mul(count, WORD_SIZE)))
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
            resultPtr0 := draw_challenges(transcript0, count0)
        }
    }

    /// @notice Append calldata to the transcript, and update the state of the transcript.
    /// @dev This is achieved by hashing the current transcript state with the new calldata.
    /// @param transcript0 The current state of the transcript
    /// @param data0 The calldata to append
    /// @return transcriptOut0 The updated state of the transcript
    function appendCalldata( // solhint-disable-line gas-calldata-parameters
    uint256[1] memory transcript0, bytes calldata data0)
        external
        pure
        returns (uint256[1] memory transcriptOut0)
    {
        assembly {
            function append_calldata(transcript_ptr, offset, size) {
                let free_ptr := mload(FREE_PTR)
                mstore(free_ptr, mload(transcript_ptr))
                calldatacopy(add(free_ptr, WORD_SIZE), offset, size)
                mstore(transcript_ptr, keccak256(free_ptr, add(size, WORD_SIZE)))
            }
            append_calldata(transcript0, data0.offset, data0.length)
        }
        transcriptOut0 = transcript0;
    }
}
