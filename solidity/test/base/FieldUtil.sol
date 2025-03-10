// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../../src/base/Constants.sol";

using {F.into, _add as +, _mul as *, _sub as -, _neg as -} for FF global;

type FF is uint256;

function _add(FF a, FF b) pure returns (FF c) {
    c = FF.wrap(addmod(a.into(), b.into(), MODULUS));
}

function _mul(FF a, FF b) pure returns (FF c) {
    c = FF.wrap(mulmod(FF.unwrap(a), FF.unwrap(b), MODULUS));
}

function _sub(FF a, FF b) pure returns (FF c) {
    c = a + (-b);
}

function _neg(FF a) pure returns (FF c) {
    c = FF.wrap(MODULUS_MINUS_ONE) * a;
}

library F {
    FF public constant ZERO = FF.wrap(0);
    FF public constant ONE = FF.wrap(1);
    FF public constant TWO = FF.wrap(2);

    function from(int64 a) internal pure returns (FF c) {
        if (a < 0) {
            c = -FF.wrap(uint256(-int256(a)));
        } else {
            c = FF.wrap(uint256(int256(a)));
        }
    }

    function from(uint256 a) internal pure returns (FF c) {
        c = FF.wrap(a);
    }

    function into(FF a) internal pure returns (uint256 c) {
        c = FF.unwrap(a) % MODULUS;
    }
}
