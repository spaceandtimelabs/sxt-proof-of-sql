// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";

contract ConstantsTest is Test {
    function testModulusMaskIsCorrect() public pure {
        assert(MODULUS > MODULUS_MASK);
        assert(MODULUS < (MODULUS_MASK << 1));

        // Check that the bits of MODULUS_MASK are a few 0s followed by all 1s.
        uint256 mask = MODULUS_MASK;
        while (mask & 1 == 1) {
            mask >>= 1;
        }
        assert(mask == 0);
    }

    function testModulusPlusAndMinusOneAreCorrect() public pure {
        assert(MODULUS_PLUS_ONE == MODULUS + 1);
        assert(MODULUS_MINUS_ONE == MODULUS - 1);
    }

    function testWordSizesAreCorrect() public pure {
        assert(WORD_SIZE == 32);
        assert(WORDX2_SIZE == 2 * WORD_SIZE);
        assert(WORDX3_SIZE == 3 * WORD_SIZE);
        assert(WORDX4_SIZE == 4 * WORD_SIZE);
        assert(WORDX6_SIZE == 6 * WORD_SIZE);
        assert(WORDX12_SIZE == 12 * WORD_SIZE);
    }

    function testVerificationBuilderOffsetsAreValid() public pure {
        uint256[10] memory offsets = [
            CHALLENGE_HEAD_OFFSET,
            CHALLENGE_TAIL_OFFSET,
            FIRST_ROUND_MLE_HEAD_OFFSET,
            FIRST_ROUND_MLE_TAIL_OFFSET,
            FINAL_ROUND_MLE_HEAD_OFFSET,
            FINAL_ROUND_MLE_TAIL_OFFSET,
            CHI_EVALUATION_HEAD_OFFSET,
            CHI_EVALUATION_TAIL_OFFSET,
            RHO_EVALUATION_HEAD_OFFSET,
            RHO_EVALUATION_TAIL_OFFSET
        ];
        uint256 offsetsLength = offsets.length;
        assert(VERIFICATION_BUILDER_SIZE == offsetsLength * WORD_SIZE);
        for (uint256 i = 0; i < offsetsLength; ++i) {
            assert(offsets[i] % WORD_SIZE == 0); // Offsets must be word-aligned
            assert(offsets[i] < VERIFICATION_BUILDER_SIZE); // Offsets must be within the builder
            for (uint256 j = i + 1; j < offsetsLength; ++j) {
                assert(offsets[i] != offsets[j]); // Offsets must be unique
            }
        }
    }
}
