// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {ProofExpr} from "../../src/proof_exprs/ProofExpr.pre.sol";
import {VerificationBuilder} from "../../src/proof/VerificationBuilder.pre.sol";

contract ProofExprTest is Test {
    function testColumnExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        uint256[] memory values = new uint256[](2);
        values[0] = 0x11111111;
        values[1] = 0x22222222;
        builder.columnEvaluations = values;

        bytes memory expr = abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(1), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 0);

        assert(eval == 0x22222222);
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testLiteralExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 3);

        assert(eval == 6); // 2 * 3
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testUnsupportedVariant() public {
        VerificationBuilder.Builder memory builder;
        bytes memory exprIn = abi.encodePacked(uint32(2), hex"abcdef");
        vm.expectRevert(Errors.UnsupportedProofExprVariant.selector);
        ProofExpr.__proofExprEvaluate(exprIn, builder, 0);
    }

    function testVariantsMatchEnum() public pure {
        assert(uint32(ProofExpr.ExprVariant.Column) == COLUMN_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Literal) == LITERAL_EXPR_VARIANT);
    }
}
