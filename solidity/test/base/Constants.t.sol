// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";

import {
    WORD_SIZE,
    WORDX2_SIZE,
    WORDX3_SIZE,
    WORDX4_SIZE,
    WORDX12_SIZE,
    MODULUS,
    MODULUS_PLUS_ONE,
    MODULUS_MASK,
    Errors
} from "../../src/base/Constants.sol";
// solhint-disable-next-line no-unused-import
import {INVALID_EC_ADD_INPUTS, INVALID_EC_MUL_INPUTS, INVALID_EC_PAIRING_INPUTS} from "../../src/base/Constants.sol";

contract ConstantsTest is Test {
    function testErrorFailedInvalidECAddInputs() public {
        vm.expectPartialRevert(Errors.InvalidECAddInputs.selector);
        assembly {
            mstore(0, INVALID_EC_ADD_INPUTS)
            revert(0, 4)
        }
    }

    function testErrorFailedInvalidECMulInputs() public {
        vm.expectPartialRevert(Errors.InvalidECMulInputs.selector);
        assembly {
            mstore(0, INVALID_EC_MUL_INPUTS)
            revert(0, 4)
        }
    }

    function testErrorFailedInvalidECPairingInputs() public {
        vm.expectPartialRevert(Errors.InvalidECPairingInputs.selector);
        assembly {
            mstore(0, INVALID_EC_PAIRING_INPUTS)
            revert(0, 4)
        }
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

    function testWordSizesAreCrrect() public pure {
        assert(WORD_SIZE == 32);
        assert(WORDX2_SIZE == 2 * WORD_SIZE);
        assert(WORDX3_SIZE == 3 * WORD_SIZE);
        assert(WORDX4_SIZE == 4 * WORD_SIZE);
        assert(WORDX12_SIZE == 12 * WORD_SIZE);
    }
}
