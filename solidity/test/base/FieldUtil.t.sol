// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {F, FF} from "./FieldUtil.sol";

contract FieldUtilTest is Test {
    function testFromPositive() public pure {
        int64 input = 23;
        FF result = F.from(input);
        assert(result.into() == 23);
    }

    function testFromNegative() public pure {
        int64 input = -23;
        FF result = F.from(input);
        assert(result.into() == MODULUS - 23);
    }

    function testAdd() public pure {
        FF a = FF.wrap(5);
        FF b = FF.wrap(3);
        FF result = a + b;
        assert(result.into() == 8);
    }

    function testAddOverflow() public pure {
        FF a = FF.wrap(MODULUS - 1);
        FF b = FF.wrap(2);
        FF result = a + b;
        assert(result.into() == 1);
    }

    function testMul() public pure {
        FF a = FF.wrap(5);
        FF b = FF.wrap(3);
        FF result = a * b;
        assert(result.into() == 15);
    }

    function testMulOverflow() public pure {
        FF a = FF.wrap(MODULUS - 1);
        FF b = FF.wrap(2);
        FF result = a * b;
        assert(result.into() == MODULUS - 2);
    }

    function testSub() public pure {
        FF a = FF.wrap(5);
        FF b = FF.wrap(3);
        FF result = a - b;
        assert(result.into() == 2);
    }

    function testSubUnderflow() public pure {
        FF a = FF.wrap(3);
        FF b = FF.wrap(5);
        FF result = a - b;
        assert(result.into() == MODULUS - 2);
    }

    function testNeg() public pure {
        FF a = FF.wrap(23);
        FF result = -a;
        assert(result.into() == MODULUS - 23);
    }

    function testNegZero() public pure {
        FF a = FF.wrap(0);
        FF result = -a;
        assert(result.into() == 0);
    }

    function testFuzzField(uint256 a, uint256 b) public pure {
        FF fa = FF.wrap(a);
        FF fb = FF.wrap(b);

        // Test addition
        FF sum = fa + fb;
        assert(sum.into() == addmod(a, b, MODULUS));

        // Test multiplication
        FF product = fa * fb;
        assert(product.into() == mulmod(a, b, MODULUS));

        // Test subtraction
        FF diff = fa - fb;
        assert(diff.into() == addmod(a, mulmod(MODULUS_MINUS_ONE, b, MODULUS), MODULUS));

        // Test negation
        FF neg = -fa;
        assert(neg.into() == mulmod(MODULUS_MINUS_ONE, a, MODULUS));
    }

    function testFuzzFromInt64(int64 value) public pure {
        FF field = F.from(value);
        if (value < 0) {
            assert(field.into() == MODULUS - uint256(-int256(value)));
        } else {
            assert(field.into() == uint256(int256(value)));
        }
    }

    function testFuzzFromUint256(uint256 value) public pure {
        assert(FF.unwrap(F.from(value)) == value);
    }

    function testInto() public pure {
        assert(FF.wrap(23).into() == 23);
        assert(FF.wrap(MODULUS).into() == 0);
        assert(FF.wrap(MODULUS + 5).into() == 5);
    }

    function testFuzzInto(uint256 value) public pure {
        assert(FF.wrap(value).into() == value % MODULUS);
    }

    function testConstants() public pure {
        assert(F.ZERO.into() == 0);
        assert(F.ONE.into() == 1);
        assert(F.TWO.into() == 2);
    }
}
