// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {VerificationBuilder} from "../../src/builder/VerificationBuilder.pre.sol";
import {EqualsExpr} from "../../src/proof_exprs/EqualsExpr.pre.sol";
import {FF, F} from "../base/FieldUtil.sol";

contract EqualsExprTest is Test {
    function testSimpleEqualsExpr() public pure {
        bytes memory expr = abi.encodePacked(
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(3)),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2)),
            hex"abcdef"
        );
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.finalRoundMLEs = new uint256[](2);
        builder.finalRoundMLEs[0] = 0;
        builder.finalRoundMLEs[1] = 5;
        builder.constraintMultipliers = new uint256[](2);
        builder.constraintMultipliers[0] = 1;
        builder.constraintMultipliers[1] = 100;
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = 1;

        uint256 eval;
        (expr, builder, eval) = EqualsExpr.__equalsExprEvaluate(expr, builder, 10);

        assert(eval == 5);
        assert(builder.aggregateEvaluation == 550);
        assert(builder.finalRoundMLEs.length == 0);
        assert(builder.constraintMultipliers.length == 0);

        bytes memory expectedExprOut = hex"abcdef";
        assert(expr.length == expectedExprOut.length);
        uint256 exprOutLength = expr.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(expr[i] == expectedExprOut[i]);
        }
    }

    function computeEqualsExprAggregateEvaluation(
        VerificationBuilder.Builder memory builder,
        FF chiEvaluation,
        FF lhsEval,
        FF rhsEval
    ) public pure returns (FF aggregateEvaluation) {
        FF diffEval = lhsEval - rhsEval;
        FF diffStarEval = F.from(builder.finalRoundMLEs[0]);
        FF eval = F.from(builder.finalRoundMLEs[1]);

        FF identityConstraint0Eval = eval * diffEval;
        FF identityConstraint1Eval = chiEvaluation - (diffEval * diffStarEval + eval);

        aggregateEvaluation = F.from(builder.aggregateEvaluation);
        aggregateEvaluation = aggregateEvaluation
            + F.from(builder.constraintMultipliers[0]) * F.from(builder.rowMultipliersEvaluation) * identityConstraint0Eval;
        aggregateEvaluation = aggregateEvaluation
            + F.from(builder.constraintMultipliers[1]) * F.from(builder.rowMultipliersEvaluation) * identityConstraint1Eval;
    }

    function computeEqualsExprResultEvaluation(VerificationBuilder.Builder memory builder)
        public
        pure
        returns (FF resultEvaluation)
    {
        resultEvaluation = F.from(builder.finalRoundMLEs[1]);
    }

    function testFuzzEqualsExpr(
        VerificationBuilder.Builder memory builder,
        uint256 chiEvaluation,
        int64 lhsValue,
        int64 rhsValue,
        bytes memory trailingExpr
    ) public pure {
        vm.assume(builder.finalRoundMLEs.length > 1);
        vm.assume(builder.constraintMultipliers.length > 1);
        vm.assume(builder.maxDegree > 2);

        FF expectedAggregateEvaluation = computeEqualsExprAggregateEvaluation(
            builder,
            F.from(chiEvaluation),
            F.from(lhsValue) * F.from(chiEvaluation),
            F.from(rhsValue) * F.from(chiEvaluation)
        );
        FF expectedEval = computeEqualsExprResultEvaluation(builder);

        bytes memory expr = abi.encodePacked(
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, lhsValue),
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, rhsValue),
            trailingExpr
        );

        uint256 eval;
        (expr, builder, eval) = EqualsExpr.__equalsExprEvaluate(expr, builder, chiEvaluation);

        assert(eval == expectedEval.into());
        assert(builder.aggregateEvaluation == expectedAggregateEvaluation.into());

        uint256 exprLength = expr.length;
        assert(exprLength == trailingExpr.length);
        for (uint256 i = 0; i < exprLength; ++i) {
            assert(expr[i] == trailingExpr[i]);
        }
    }
}
