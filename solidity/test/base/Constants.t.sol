// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";

library ErrorTest {
    function causeInvalidECAddInputs() public pure {
        assembly {
            mstore(0, INVALID_EC_ADD_INPUTS)
            revert(0, 4)
        }
    }

    function causeInvalidECMulInputs() public pure {
        assembly {
            mstore(0, INVALID_EC_MUL_INPUTS)
            revert(0, 4)
        }
    }

    function causeInvalidECPairingInputs() public pure {
        assembly {
            mstore(0, INVALID_EC_PAIRING_INPUTS)
            revert(0, 4)
        }
    }

    function causeRoundEvaluationMismatch() public pure {
        assembly {
            mstore(0, ROUND_EVALUATION_MISMATCH)
            revert(0, 4)
        }
    }
}

contract ConstantsTest is Test {
    function testErrorFailedInvalidECAddInputs() public {
        vm.expectRevert(Errors.InvalidECAddInputs.selector);
        ErrorTest.causeInvalidECAddInputs();
    }

    function testErrorFailedInvalidECMulInputs() public {
        vm.expectRevert(Errors.InvalidECMulInputs.selector);
        ErrorTest.causeInvalidECMulInputs();
    }

    function testErrorFailedInvalidECPairingInputs() public {
        vm.expectRevert(Errors.InvalidECPairingInputs.selector);
        ErrorTest.causeInvalidECPairingInputs();
    }

    function testErrorFailedRoundEvaluationMismatch() public {
        vm.expectRevert(Errors.RoundEvaluationMismatch.selector);
        ErrorTest.causeRoundEvaluationMismatch();
    }

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

    function testModulusPlusOneIsCorrect() public pure {
        assert(MODULUS_PLUS_ONE == MODULUS + 1);
    }

    function testWordSizesAreCorrect() public pure {
        assert(WORD_SIZE == 32);
        assert(WORDX2_SIZE == 2 * WORD_SIZE);
        assert(WORDX3_SIZE == 3 * WORD_SIZE);
        assert(WORDX4_SIZE == 4 * WORD_SIZE);
        assert(WORDX12_SIZE == 12 * WORD_SIZE);
    }
}
