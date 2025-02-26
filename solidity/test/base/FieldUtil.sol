// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import "../../src/base/Constants.sol";

using {FieldUtil.into, _add as +, _mul as *, _sub as -, _neg as -} for F global;

type F is uint256;

function _add(F a, F b) pure returns (F c) {
    c = F.wrap(addmod(a.into(), b.into(), MODULUS));
}

function _mul(F a, F b) pure returns (F c) {
    c = F.wrap(mulmod(F.unwrap(a), F.unwrap(b), MODULUS));
}

function _sub(F a, F b) pure returns (F c) {
    c = a + (-b);
}

function _neg(F a) pure returns (F c) {
    c = F.wrap(MODULUS_MINUS_ONE) * a;
}

library FieldUtil {
    function from(int64 a) internal pure returns (F c) {
        if (a < 0) {
            c = -F.wrap(uint256(-int256(a)));
        } else {
            c = F.wrap(uint256(int256(a)));
        }
    }

    function into(F a) internal pure returns (uint256 c) {
        c = F.unwrap(a) % MODULUS;
    }
}
