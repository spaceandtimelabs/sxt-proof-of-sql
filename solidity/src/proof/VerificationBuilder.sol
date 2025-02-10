// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../base/Constants.sol"; // solhint-disable-line no-global-import

library VerificationBuilder {
    /// @notice Allocates and reserves a block of memory for a verification builder.
    /// @return __builderPtr The pointer to the allocated builder region.
    function __allocate() internal pure returns (uint256 __builderPtr) {
        assembly {
            function builder_allocate() -> builder_ptr {
                builder_ptr := mload(FREE_PTR)
                mstore(FREE_PTR, add(builder_ptr, VERIFICATION_BUILDER_SIZE))
            }
            __builderPtr := builder_allocate()
        }
    }

    /// @notice Sets the challenges in the verification builder.
    /// @param __builderPtr The pointer to the verification builder.
    /// @param __challengePtr The pointer to the challenges.
    /// @param __challengeLength The number of challenges.
    /// This is assumed to be "small", i.e. anything less than 2^64 will work.
    function __setChallenges(uint256 __builderPtr, uint256 __challengePtr, uint256 __challengeLength) internal pure {
        assembly {
            function builder_set_challenges(builder_ptr, challenge_ptr, challenge_length) {
                mstore(add(builder_ptr, CHALLENGE_HEAD_OFFSET), challenge_ptr)
                mstore(add(builder_ptr, CHALLENGE_TAIL_OFFSET), add(challenge_ptr, mul(WORD_SIZE, challenge_length)))
            }
            builder_set_challenges(__builderPtr, __challengePtr, __challengeLength)
        }
    }

    /// @notice Consumes a challenge from the verification builder.
    /// @param __builderPtr The pointer to the verification builder.
    /// @return __challenge The consumed challenge.
    /// @dev This function will revert if there are no challenges left to consume.
    function __consumeChallenge(uint256 __builderPtr) internal pure returns (uint256 __challenge) {
        assembly {
            function builder_consume_challenge(builder_ptr) -> challenge {
                let head_ptr := mload(add(builder_ptr, CHALLENGE_HEAD_OFFSET))
                challenge := mload(head_ptr)
                head_ptr := add(head_ptr, WORD_SIZE)
                if gt(head_ptr, mload(add(builder_ptr, CHALLENGE_TAIL_OFFSET))) {
                    mstore(0, TOO_FEW_CHALLENGES)
                    revert(0, 4)
                }
                mstore(add(builder_ptr, CHALLENGE_HEAD_OFFSET), head_ptr)
            }
            __challenge := builder_consume_challenge(__builderPtr)
        }
    }
}
