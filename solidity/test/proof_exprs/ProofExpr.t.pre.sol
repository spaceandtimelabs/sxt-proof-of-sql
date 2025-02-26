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

        bytes memory exprIn = abi.encodePacked(COLUMN_EXPR_VARIANT, uint64(1), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";

        (bytes memory exprOut, uint256 eval) = ProofExpr.__proofExprEvaluate(exprIn, builder, 0);
        assert(eval == 0x22222222);
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    function testLiteralExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory exprIn = abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2), hex"abcdef");
        bytes memory expectedExprOut = hex"abcdef";

        (bytes memory exprOut, uint256 eval) = ProofExpr.__proofExprEvaluate(exprIn, builder, 3);
        assert(eval == 6); // 2 * 3
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
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
