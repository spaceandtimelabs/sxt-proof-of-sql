// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "./Constants.sol";

/// @title Transcript Library
/// @notice Provides functions to manage a simple public coin transcript
/// @dev The transcript is a sequence of messages and challenges, where each challenge is a scalar (less than MODULUS).
library Transcript {
    /// @notice Draw a challenge from the transcript, and update the state of the transcript.
    /// @dev The result should be a scalar value (less than MODULUS).
    /// The challenges is a masked version of the current state of the transcript.
    /// The new state of the transcript is the hash of the current state.
    /// @param __transcript The current state of the transcript
    /// @return __result The drawn challenge
    function __drawChallenge(uint256[1] memory __transcript) internal pure returns (uint256 __result) {
        assembly {
            function draw_challenge(transcript_ptr) -> result {
                result := and(mload(transcript_ptr), MODULUS_MASK)
                mstore(transcript_ptr, keccak256(transcript_ptr, WORD_SIZE))
            }
            __result := draw_challenge(__transcript)
        }
    }

    /// @notice Draw multiple challenges from the transcript, and update the state of the transcript.
    /// @dev This is equivalent to calling `__drawChallenge` multiple times.
    /// The returned value is a pointer to newly allocated memory containing the challenges.
    /// The first entry is NOT the length of the results. That is count.
    /// @param __transcript The current state of the transcript
    /// @param __count The number of challenges to draw
    /// @return __resultPtr A pointer to the memory containing the drawn challenges
    function __drawChallenges(uint256[1] memory __transcript, uint256 __count)
        internal
        pure
        returns (uint256[] memory __resultPtr)
    {
        assembly {
            function draw_challenges(transcript_ptr, count) -> result_ptr {
                // allocate `count` words
                let free_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(free_ptr, mul(add(count, 1), WORD_SIZE)))
                // result is the pointer to the first word
                result_ptr := free_ptr
                // store count in the first word
                mstore(result_ptr, count)
                // increment to next word
                free_ptr := add(free_ptr, WORD_SIZE)
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
            __resultPtr := draw_challenges(__transcript, __count)
        }
    }

    /// @notice Append calldata to the transcript, and update the state of the transcript.
    /// @dev This is achieved by hashing the current transcript state with the new calldata.
    /// @param __transcript The current state of the transcript
    /// @param __data The calldata to append
    /// @return __resultTranscript The updated state of the transcript
    function __appendCalldata( // solhint-disable-line gas-calldata-parameters
    uint256[1] memory __transcript, bytes calldata __data)
        external
        pure
        returns (uint256[1] memory __resultTranscript)
    {
        assembly {
            function append_calldata(transcript_ptr, offset, size) {
                let free_ptr := mload(FREE_PTR)
                mstore(free_ptr, mload(transcript_ptr))
                calldatacopy(add(free_ptr, WORD_SIZE), offset, size)
                mstore(transcript_ptr, keccak256(free_ptr, add(size, WORD_SIZE)))
            }
            append_calldata(__transcript, __data.offset, __data.length)
        }
        __resultTranscript = __transcript;
    }
}
