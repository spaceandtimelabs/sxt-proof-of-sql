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
        assert(WORDX5_SIZE == 5 * WORD_SIZE);
        assert(WORDX6_SIZE == 6 * WORD_SIZE);
        assert(WORDX8_SIZE == 8 * WORD_SIZE);
        assert(WORDX9_SIZE == 9 * WORD_SIZE);
        assert(WORDX10_SIZE == 10 * WORD_SIZE);
        assert(WORDX11_SIZE == 11 * WORD_SIZE);
        assert(WORDX12_SIZE == 12 * WORD_SIZE);
    }

    function testUint32SizesAreCorrect() public pure {
        assert(UINT32_SIZE * 8 == 32);
        assert(UINT32_PADDING_BITS == 256 - 32);
    }

    function testUint64SizesAreCorrect() public pure {
        assert(UINT64_SIZE * 8 == 64);
        assert(UINT64_PADDING_BITS == 256 - 64);
    }

    function testInt64SizesAreCorrect() public pure {
        assert(INT64_SIZE * 8 == 64);
        assert(INT64_PADDING_BITS == 256 - 64);
        assert(INT64_SIZE_MINUS_ONE == INT64_SIZE - 1);
    }

    function testVerificationBuilderOffsetsAreValid() public pure {
        uint256[11] memory offsets = [
            BUILDER_CHALLENGES_OFFSET,
            BUILDER_FIRST_ROUND_MLES_OFFSET,
            BUILDER_FINAL_ROUND_MLES_OFFSET,
            BUILDER_CHI_EVALUATIONS_OFFSET,
            BUILDER_RHO_EVALUATIONS_OFFSET,
            BUILDER_CONSTRAINT_MULTIPLIERS_OFFSET,
            BUILDER_MAX_DEGREE_OFFSET,
            BUILDER_AGGREGATE_EVALUATION_OFFSET,
            BUILDER_ROW_MULTIPLIERS_EVALUATION_OFFSET,
            BUILDER_COLUMN_EVALUATIONS_OFFSET,
            BUILDER_TABLE_CHI_EVALUATIONS_OFFSET
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
