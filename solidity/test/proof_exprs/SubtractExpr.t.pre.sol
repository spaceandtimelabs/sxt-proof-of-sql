// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {SubtractExpr} from "../../src/proof_exprs/SubtractExpr.pre.sol";
import {F} from "../base/FieldUtil.sol";

contract SubtractExprTest is Test {
    function testSimpleSubtractExpr() public pure {
        bytes memory expr = abi.encodePacked(
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(7)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(5)),
            hex"abcdef"
        );
        VerificationBuilder.Builder memory builder;

        uint256 eval;
        (expr, builder, eval) = SubtractExpr.__subtractExprEvaluate(expr, builder, 10);

        assert(eval == 20);
        bytes memory expectedExprOut = hex"abcdef";
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testFuzzSubtractExpr(
        VerificationBuilder.Builder memory builder,
        uint256 chiEvaluation,
        int64 lhsValue,
        int64 rhsValue,
        bytes memory trailingExpr
    ) public pure {
        bytes memory expr = abi.encodePacked(
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, lhsValue),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, rhsValue),
            trailingExpr
        );

        uint256 eval;
        (expr, builder, eval) = SubtractExpr.__subtractExprEvaluate(expr, builder, chiEvaluation);

        assert(eval == ((F.from(lhsValue) - F.from(rhsValue)) * F.from(chiEvaluation)).into());
        assert(expr.length == trailingExpr.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == trailingExpr[i]);
        }
    }
}
