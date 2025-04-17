// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {ProofExpr} from "../../src/proof_exprs/ProofExpr.pre.sol";

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

    function testEqualsExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = new uint256[](2);
        builder.finalRoundMLEs[1] = 123;
        builder.constraintMultipliers = new uint256[](2);
        builder.constraintMultipliers[0] = 456;
        builder.rowMultipliersEvaluation = 789;
        builder.maxDegree = 3;

        bytes memory expr = abi.encodePacked(
            EQUALS_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 999);

        assert(eval == 123);
        assert(builder.aggregateEvaluation == 0);
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testAddExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(
            ADD_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 3);

        assert(eval == 12); // 2 * 3 + 2 * 3
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testSubtractExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(
            SUBTRACT_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(3)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 3);

        assert(eval == 3); // 3 * 3 - 2 * 3
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testMultiplyExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = new uint256[](1);
        builder.finalRoundMLEs[0] = 405;
        builder.constraintMultipliers = new uint256[](1);
        builder.constraintMultipliers[0] = 20;
        builder.rowMultipliersEvaluation = 2;
        builder.maxDegree = 3;

        bytes memory expr = abi.encodePacked(
            MULTIPLY_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 10);

        assert(eval == 405);
        assert(builder.aggregateEvaluation == 200);
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testAndExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = new uint256[](1);
        builder.finalRoundMLEs[0] = 105;
        builder.constraintMultipliers = new uint256[](1);
        builder.constraintMultipliers[0] = 20;
        builder.rowMultipliersEvaluation = 2;
        builder.maxDegree = 3;

        bytes memory expr = abi.encodePacked(
            AND_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(1)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(1)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 10);

        assert(eval == 105);
        assert(builder.aggregateEvaluation == 200);
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testOrExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        builder.finalRoundMLEs = new uint256[](1);
        builder.finalRoundMLEs[0] = 105;
        builder.constraintMultipliers = new uint256[](1);
        builder.constraintMultipliers[0] = 20;
        builder.rowMultipliersEvaluation = 2;
        builder.maxDegree = 3;

        bytes memory expr = abi.encodePacked(
            OR_EXPR_VARIANT,
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(1)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(1)),
            hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 10);

        assert(eval == addmod(MODULUS, mulmod(MODULUS_MINUS_ONE, 85, MODULUS), MODULUS));
        assert(builder.aggregateEvaluation == 200);
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function testNotExprVariant() public pure {
        VerificationBuilder.Builder memory builder;
        bytes memory expr = abi.encodePacked(
            NOT_EXPR_VARIANT, abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(0)), hex"abcdef"
        );
        bytes memory expectedExprOut = hex"abcdef";

        uint256 eval;
        (expr, builder, eval) = ProofExpr.__proofExprEvaluate(expr, builder, 3);

        assert(eval == 3); // 1 * 3 - 0 * 3
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    /// forge-config: default.allow_internal_expect_revert = true
    function testUnsupportedVariant() public {
        VerificationBuilder.Builder memory builder;
        bytes memory exprIn = abi.encodePacked(uint32(25), hex"abcdef");
        vm.expectRevert(Errors.UnsupportedProofExprVariant.selector);
        ProofExpr.__proofExprEvaluate(exprIn, builder, 0);
    }

    function testVariantsMatchEnum() public pure {
        assert(uint32(ProofExpr.ExprVariant.Column) == COLUMN_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Literal) == LITERAL_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Equals) == EQUALS_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Add) == ADD_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Subtract) == SUBTRACT_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Multiply) == MULTIPLY_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.And) == AND_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Or) == OR_EXPR_VARIANT);
        assert(uint32(ProofExpr.ExprVariant.Not) == NOT_EXPR_VARIANT);
    }
}
