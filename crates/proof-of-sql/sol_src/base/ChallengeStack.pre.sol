// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract ChallengeStack {
    function challengeStackMethods() public pure {
        assembly {
            function challenge_stack_new(count, transcript) -> stack_ptr, new_transcript {
                stack_ptr := mload(0x40)
                mstore(stack_ptr, shl(5, count))
                let current_ptr := add(stack_ptr, 32)
                for {} count {} {
                    mstore(current_ptr, transcript)
                    transcript := keccak256(current_ptr, 32)
                    current_ptr := add(current_ptr, 32)
                    count := sub(count, 1)
                }
                mstore(0x40, current_ptr)
                new_transcript := transcript
            }
            function challenge_stack_pop(ptr) -> value, offset {
                offset := mload(ptr)
                if offset {
                    value := mload(add(ptr, offset))
                    mstore(ptr, sub(offset, 32))
                }
            }
            function challenge_stack_unchecked_pop(ptr) -> value {
                let offset := mload(ptr)
                value := mload(add(ptr, offset))
                mstore(ptr, sub(offset, 32))
            }
        }
    }

    function testChallengeStackNew() public pure {
        assembly {
            // IMPORT-YUL ChallengeStack.pre.sol
            function challenge_stack_new(count, transcript) -> stack_ptr, new_transcript {}

            let free_ptr := mload(0x40)
            let start_transcript := 0x1234567890abcdef
            let stack_ptr, final_transcript := challenge_stack_new(3, start_transcript)

            // The stack_ptr should point to the old free pointer
            if iszero(eq(stack_ptr, free_ptr)) { revert(0, 0) }
            // The free pointer should have been moved 4 words forward
            if iszero(eq(mload(0x40), add(free_ptr, 128))) { revert(0, 0) }

            // The size of the stack should be 3
            if iszero(eq(mload(stack_ptr), 96)) { revert(0, 0) }

            // The first element of the stack should be the start transcript
            if iszero(eq(mload(add(stack_ptr, 32)), start_transcript)) { revert(0, 0) }

            // The second element of the stack not match either the start transcript or 0
            if eq(mload(add(stack_ptr, 64)), start_transcript) { revert(0, 0) }
            if eq(mload(add(stack_ptr, 64)), 0) { revert(0, 0) }
            if eq(mload(add(stack_ptr, 64)), final_transcript) { revert(0, 0) }

            // The third element of the stack not match either the start transcript or 0
            if eq(mload(add(stack_ptr, 96)), start_transcript) { revert(0, 0) }
            if eq(mload(add(stack_ptr, 96)), 0) { revert(0, 0) }
            if eq(mload(add(stack_ptr, 96)), final_transcript) { revert(0, 0) }

            // The final transcript should not match the start transcript or 0
            if eq(final_transcript, start_transcript) { revert(0, 0) }
            if eq(final_transcript, 0) { revert(0, 0) }
        }
    }

    function testChallengeStackPop() public pure {
        assembly {
            // IMPORT-YUL ChallengeStack.pre.sol
            function challenge_stack_new(count, transcript) -> stack_ptr, new_transcript {}
            // IMPORT-YUL ChallengeStack.pre.sol
            function challenge_stack_pop(ptr) -> value, offset {}

            let start_transcript := 0x1234567890abcdef
            // solhint-disable-next-line no-unused-vars
            let stack_ptr, final_transcript := challenge_stack_new(3, start_transcript)
            let a := mload(add(stack_ptr, 32))
            let b := mload(add(stack_ptr, 64))
            let c := mload(add(stack_ptr, 96))

            let value_c, offset_c := challenge_stack_pop(stack_ptr)
            if iszero(eq(offset_c, 96)) { revert(0, 0) }
            if iszero(eq(c, value_c)) { revert(0, 0) }

            let value_b, offset_b := challenge_stack_pop(stack_ptr)
            if iszero(eq(offset_b, 64)) { revert(0, 0) }
            if iszero(eq(b, value_b)) { revert(0, 0) }

            let value_a, offset_a := challenge_stack_pop(stack_ptr)
            if iszero(eq(offset_a, 32)) { revert(0, 0) }
            if iszero(eq(a, value_a)) { revert(0, 0) }

            // solhint-disable-next-line no-unused-vars
            let value_invalid, offset_invalid := challenge_stack_pop(stack_ptr)
            if offset_invalid { revert(0, 0) }
        }
    }

    function testChallengeUncheckedStackPop() public pure {
        assembly {
            // IMPORT-YUL ChallengeStack.pre.sol
            function challenge_stack_new(count, transcript) -> stack_ptr, new_transcript {}
            // IMPORT-YUL ChallengeStack.pre.sol
            function challenge_stack_unchecked_pop(ptr) -> value {}

            let start_transcript := 0x1234567890abcdef
            // solhint-disable-next-line no-unused-vars
            let stack_ptr, final_transcript := challenge_stack_new(3, start_transcript)
            let a := mload(add(stack_ptr, 32))
            let b := mload(add(stack_ptr, 64))
            let c := mload(add(stack_ptr, 96))

            let value_c := challenge_stack_unchecked_pop(stack_ptr)
            if iszero(eq(c, value_c)) { revert(0, 0) }

            let value_b := challenge_stack_unchecked_pop(stack_ptr)
            if iszero(eq(b, value_b)) { revert(0, 0) }

            let value_a := challenge_stack_unchecked_pop(stack_ptr)
            if iszero(eq(a, value_a)) { revert(0, 0) }
        }
    }
}
