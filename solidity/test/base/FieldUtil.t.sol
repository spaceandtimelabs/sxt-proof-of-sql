// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {F, FieldUtil} from "./FieldUtil.sol";

contract FieldUtilTest is Test {
    function testFromPositive() public pure {
        int64 input = 42;
        F result = FieldUtil.from(input);
        assert(result.into() == 42);
    }

    function testFromNegative() public pure {
        int64 input = -42;
        F result = FieldUtil.from(input);
        assert(result.into() == MODULUS - 42);
    }

    function testAdd() public pure {
        F a = F.wrap(5);
        F b = F.wrap(3);
        F result = a + b;
        assert(result.into() == 8);
    }

    function testAddOverflow() public pure {
        F a = F.wrap(MODULUS - 1);
        F b = F.wrap(2);
        F result = a + b;
        assert(result.into() == 1);
    }

    function testMul() public pure {
        F a = F.wrap(5);
        F b = F.wrap(3);
        F result = a * b;
        assert(result.into() == 15);
    }

    function testMulOverflow() public pure {
        F a = F.wrap(MODULUS - 1);
        F b = F.wrap(2);
        F result = a * b;
        assert(result.into() == MODULUS - 2);
    }

    function testSub() public pure {
        F a = F.wrap(5);
        F b = F.wrap(3);
        F result = a - b;
        assert(result.into() == 2);
    }

    function testSubUnderflow() public pure {
        F a = F.wrap(3);
        F b = F.wrap(5);
        F result = a - b;
        assert(result.into() == MODULUS - 2);
    }

    function testNeg() public pure {
        F a = F.wrap(42);
        F result = -a;
        assert(result.into() == MODULUS - 42);
    }

    function testNegZero() public pure {
        F a = F.wrap(0);
        F result = -a;
        assert(result.into() == 0);
    }

    function testFuzzField(uint256 a, uint256 b) public pure {
        F fa = F.wrap(a);
        F fb = F.wrap(b);

        // Test addition
        F sum = fa + fb;
        assert(sum.into() == addmod(a, b, MODULUS));

        // Test multiplication
        F product = fa * fb;
        assert(product.into() == mulmod(a, b, MODULUS));

        // Test subtraction
        F diff = fa - fb;
        assert(diff.into() == addmod(a, mulmod(MODULUS_MINUS_ONE, b, MODULUS), MODULUS));

        // Test negation
        F neg = -fa;
        assert(neg.into() == mulmod(MODULUS_MINUS_ONE, a, MODULUS));
    }

    function testFuzzFromInt(int64 value) public pure {
        F field = FieldUtil.from(value);
        if (value < 0) {
            assert(field.into() == MODULUS - uint256(-int256(value)));
        } else {
            assert(field.into() == uint256(int256(value)));
        }
    }
}
