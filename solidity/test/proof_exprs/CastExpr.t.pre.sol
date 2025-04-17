// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {CastExpr} from "../../src/proof_exprs/CastExpr.pre.sol";
import {F} from "../base/FieldUtil.sol";

contract CastExprTest is Test {
    function testSimpleCastExpr() public pure {
        bytes memory expr =
            abi.encodePacked(abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(7)), hex"abcdef");
        VerificationBuilder.Builder memory builder;

        uint256 eval;
        (expr, builder, eval) = CastExpr.__castExprEvaluate(expr, builder, 10);

        assert(eval == 70);
        bytes memory expectedExprOut = hex"abcdef";
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testFuzzCastExpr(
        VerificationBuilder.Builder memory builder,
        uint256 chiEvaluation,
        int64 inputValue,
        bytes memory trailingExpr
    ) public pure {
        bytes memory expr =
            abi.encodePacked(abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, inputValue), trailingExpr);

        uint256 eval;
        (expr, builder, eval) = CastExpr.__castExprEvaluate(expr, builder, chiEvaluation);

        assert(eval == (F.from(inputValue) * F.from(chiEvaluation)).into());
        assert(expr.length == trailingExpr.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == trailingExpr[i]);
        }
    }
}
