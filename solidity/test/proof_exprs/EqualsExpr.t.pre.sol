// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {EqualsExpr} from "../../src/proof_exprs/EqualsExpr.pre.sol";
import {VerificationBuilder} from "../../src/proof/VerificationBuilder.pre.sol";
import {FieldUtil, F} from "../base/FieldUtil.sol";

contract EqualsExprTest is Test {
    function computeEqualsExprAggregateEvaluation(
        VerificationBuilder.Builder memory builder,
        F chiEvaluation,
        F lhsEval,
        F rhsEval
    ) public pure returns (F aggregateEvaluation) {
        F diffEval = lhsEval - rhsEval;
        F diffStarEval = F.wrap(builder.finalRoundMLEs[0]);
        F eval = F.wrap(builder.finalRoundMLEs[1]);
        F identityConstraint0Eval = eval * diffEval;
        F identityConstraint1Eval = chiEvaluation - (diffEval * diffStarEval + eval);

        F rowMultipliersEval = F.wrap(builder.rowMultipliersEvaluation);

        aggregateEvaluation = F.wrap(builder.aggregateEvaluation);
        aggregateEvaluation = aggregateEvaluation
            + F.wrap(builder.constraintMultipliers[0]) * rowMultipliersEval * identityConstraint0Eval;
        aggregateEvaluation = aggregateEvaluation
            + F.wrap(builder.constraintMultipliers[1]) * rowMultipliersEval * identityConstraint1Eval;
    }

    function computeEqualsExprResultEvaluation(VerificationBuilder.Builder memory builder)
        public
        pure
        returns (F resultEvaluation)
    {
        resultEvaluation = F.wrap(builder.finalRoundMLEs[1]);
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

        F expectedAggregateEvaluation = computeEqualsExprAggregateEvaluation(
            builder,
            F.wrap(chiEvaluation),
            FieldUtil.from(lhsValue) * F.wrap(chiEvaluation),
            FieldUtil.from(rhsValue) * F.wrap(chiEvaluation)
        );
        F expectedEval = computeEqualsExprResultEvaluation(builder);

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
