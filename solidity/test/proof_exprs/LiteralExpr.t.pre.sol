// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import "../../src/base/Errors.sol";
import {LiteralExpr} from "../../src/proof_exprs/LiteralExpr.pre.sol";

contract LiteralExprTest is Test {
    function testLiteralExpr() public pure {
        bytes memory exprIn = abi.encodePacked(LITERAL_BIGINT_VARIANT, int64(2), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";
        (bytes memory exprOut, uint256 eval) = LiteralExpr.__literalExprEvaluate(exprIn, 3);
        assert(eval == 6);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testFuzzBigIntLiteralExpr(int64 literalValue, uint256 chiInEval, bytes memory trailingExpr) public pure {
        bytes memory exprIn = abi.encodePacked(LITERAL_BIGINT_VARIANT, literalValue, trailingExpr);
        (bytes memory exprOut, uint256 eval) = LiteralExpr.__literalExprEvaluate(exprIn, chiInEval);
        uint256 literalValueAsScalar;
        if (literalValue < 0) {
            literalValueAsScalar = MODULUS - uint256(-int256(literalValue));
        } else {
            literalValueAsScalar = uint256(int256(literalValue));
        }
        assert(eval == mulmod(literalValueAsScalar, chiInEval, MODULUS));
        assert(exprOut.length == trailingExpr.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == trailingExpr[i]);
        }
    }

    function testFuzzInvalidLiteralVariant(uint32 variant) public {
        vm.assume(variant > 0);
        bytes memory exprIn = abi.encodePacked(variant, int64(2), hex"abcdef");
        vm.expectRevert(Errors.UnsupportedLiteralVariant.selector);
        LiteralExpr.__literalExprEvaluate(exprIn, 3);
    }

    function testVariantsMatchEnum() public pure {
        assert(uint32(LiteralExpr.LiteralVariant.BigInt) == LITERAL_BIGINT_VARIANT);
    }
}
