// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {MultiplyExpr} from "../../src/proof_exprs/MultiplyExpr.pre.sol";
import {FF, F} from "../base/FieldUtil.sol";

contract MultiplyExprTest is Test {
    function testSimpleMultiplyExpr() public pure {
        bytes memory expr = abi.encodePacked(
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(5)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2)),
            hex"abcdef"
        );
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.finalRoundMLEs = new uint256[](1);
        builder.finalRoundMLEs[0] = addmod(MODULUS, mulmod(MODULUS_MINUS_ONE, 500, MODULUS), MODULUS);
        builder.constraintMultipliers = new uint256[](1);
        builder.constraintMultipliers[0] = 5;
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = addmod(MODULUS, mulmod(MODULUS_MINUS_ONE, 2, MODULUS), MODULUS);

        uint256 eval;
        (expr, builder, eval) = MultiplyExpr.__multiplyExprEvaluate(expr, builder, 10);

        assert(eval == addmod(MODULUS, mulmod(MODULUS_MINUS_ONE, 500, MODULUS), MODULUS));
        assert(builder.aggregateEvaluation == 15000);
        assert(builder.finalRoundMLEs.length == 0);
        assert(builder.constraintMultipliers.length == 0);

        bytes memory expectedExprOut = hex"abcdef";
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function computeMultiplyExprAggregateEvaluation(VerificationBuilder.Builder memory builder, FF lhsEval, FF rhsEval)
        public
        pure
        returns (FF aggregateEvaluation)
    {
        FF eval = F.from(builder.finalRoundMLEs[0]);

        FF identityConstraint0Eval = eval - lhsEval * rhsEval;

        aggregateEvaluation = F.from(builder.aggregateEvaluation);
        aggregateEvaluation = aggregateEvaluation
            + F.from(builder.constraintMultipliers[0]) * F.from(builder.rowMultipliersEvaluation) * identityConstraint0Eval;
    }

    function computeMultiplyExprResultEvaluation(VerificationBuilder.Builder memory builder)
        public
        pure
        returns (FF resultEvaluation)
    {
        resultEvaluation = F.from(builder.finalRoundMLEs[0]);
    }

    function testFuzzMultiplyExpr(
        VerificationBuilder.Builder memory builder,
        uint256 chiEvaluation,
        int64 lhsValue,
        int64 rhsValue,
        bytes memory trailingExpr
    ) public pure {
        vm.assume(builder.finalRoundMLEs.length > 0);
        vm.assume(builder.constraintMultipliers.length > 0);
        vm.assume(builder.maxDegree > 2);

        FF expectedAggregateEvaluation = computeMultiplyExprAggregateEvaluation(
            builder, F.from(lhsValue) * F.from(chiEvaluation), F.from(rhsValue) * F.from(chiEvaluation)
        );
        FF expectedEval = computeMultiplyExprResultEvaluation(builder);

        bytes memory expr = abi.encodePacked(
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, lhsValue),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, rhsValue),
            trailingExpr
        );

        uint256 eval;
        (expr, builder, eval) = MultiplyExpr.__multiplyExprEvaluate(expr, builder, chiEvaluation);

        assert(eval == expectedEval.into());
        assert(builder.aggregateEvaluation == expectedAggregateEvaluation.into());

        uint256 exprLength = expr.length;
        assert(exprLength == trailingExpr.length);
        for (uint256 i = 0; i < exprLength; ++i) {
            assert(expr[i] == trailingExpr[i]);
        }
    }
}
