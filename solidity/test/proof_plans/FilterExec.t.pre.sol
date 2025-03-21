// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {FilterExec} from "../../src/proof_plans/FilterExec.pre.sol";
import {VerificationBuilder} from "../../src/proof/VerificationBuilder.pre.sol";
import {FF, F} from "../base/FieldUtil.sol";

contract EqualsExprTest is Test {
    function testSimpleFilterExec() public pure {
        bytes memory plan = abi.encodePacked(
            uint64(0), // table_number
            abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(101)), // where clause
            abi.encodePacked( // select clause
                uint64(3),
                abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(102)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(103)),
                abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(104))
            ),
            hex"abcdef"
        );
        VerificationBuilder.Builder memory builder;
        builder.maxDegree = 3;
        builder.finalRoundMLEs = new uint256[](5);
        builder.finalRoundMLEs[0] = 202;
        builder.finalRoundMLEs[1] = 203;
        builder.finalRoundMLEs[2] = 204;
        builder.finalRoundMLEs[3] = 301;
        builder.finalRoundMLEs[4] = 302;
        builder.constraintMultipliers = new uint256[](3);
        builder.constraintMultipliers[0] = 401;
        builder.constraintMultipliers[1] = 402;
        builder.constraintMultipliers[2] = 403;
        builder.challenges = new uint256[](2);
        builder.challenges[0] = 501;
        builder.challenges[1] = 502;
        builder.aggregateEvaluation = 0;
        builder.rowMultipliersEvaluation = 601;
        builder.chiEvaluations = new uint256[](1);
        builder.chiEvaluations[0] = 701;
        builder.tableChiEvaluations = new uint256[](1);
        builder.tableChiEvaluations[0] = 801;

        uint256[] memory evals;
        (plan, builder, evals) = FilterExec.__filterExecEvaluate(plan, builder);

        FF cFold = FF.wrap(502 * 502) * FF.wrap(102 * 801) + FF.wrap(502) * FF.wrap(103 * 801) + FF.wrap(104 * 801);
        FF dFold = FF.wrap(502 * 502) * FF.wrap(202) + FF.wrap(502) * FF.wrap(203) + FF.wrap(204);

        FF zeroSumConstraint0 = FF.wrap(301) * FF.wrap(101 * 801) - FF.wrap(302);
        FF identityConstraint1 = (F.ONE + FF.wrap(501) * cFold) * FF.wrap(301) - FF.wrap(801);
        FF identityConstraint2 = (F.ONE + FF.wrap(501) * dFold) * FF.wrap(302) - FF.wrap(701);

        FF expectedAggregateEvaluation = zeroSumConstraint0 * FF.wrap(401) + identityConstraint1 * FF.wrap(402 * 601)
            + identityConstraint2 * FF.wrap(403 * 601);

        assert(evals.length == 3);
        assert(evals[0] == 202);
        assert(evals[1] == 203);
        assert(evals[2] == 204);
        assert(builder.aggregateEvaluation == expectedAggregateEvaluation.into());
        assert(builder.finalRoundMLEs.length == 0);
        assert(builder.constraintMultipliers.length == 0);

        bytes memory expectedExprOut = hex"abcdef";
        assert(plan.length == expectedExprOut.length);
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }

    function _computeEqualsExprZeroSumConstraint0(
        VerificationBuilder.Builder memory builder,
        FF whereEvaluation,
        FF[] memory inputEvaluations
    ) internal pure returns (FF zeroSumConstraint0) {
        FF cStar = F.from(builder.finalRoundMLEs[inputEvaluations.length]);
        FF dStar = F.from(builder.finalRoundMLEs[inputEvaluations.length + 1]);
        zeroSumConstraint0 = cStar * whereEvaluation - dStar;
    }

    function _computeEqualsExprIdentityConstraint1(
        VerificationBuilder.Builder memory builder,
        FF[] memory inputEvaluations,
        uint64 tableNumber
    ) internal pure returns (FF identityConstraint1) {
        FF alpha = F.from(builder.challenges[0]);
        FF beta = F.from(builder.challenges[1]);
        FF cFold = F.ZERO;
        uint256 inputEvaluationsLength = inputEvaluations.length;
        for (uint256 i = 0; i < inputEvaluationsLength; ++i) {
            cFold = cFold * beta + inputEvaluations[i];
        }
        FF cStar = F.from(builder.finalRoundMLEs[inputEvaluationsLength]);
        identityConstraint1 = (F.ONE + alpha * cFold) * cStar - F.from(builder.tableChiEvaluations[tableNumber]);
    }

    function _computeEqualsExprIdentityConstraint2(
        VerificationBuilder.Builder memory builder,
        FF[] memory inputEvaluations
    ) internal pure returns (FF identityConstraint2) {
        FF alpha = F.from(builder.challenges[0]);
        FF beta = F.from(builder.challenges[1]);
        FF dFold = F.ZERO;
        uint256 inputEvaluationsLength = inputEvaluations.length;
        for (uint256 i = 0; i < inputEvaluationsLength; ++i) {
            dFold = dFold * beta + F.from(builder.finalRoundMLEs[i]);
        }
        FF dStar = F.from(builder.finalRoundMLEs[inputEvaluationsLength + 1]);
        identityConstraint2 = (F.ONE + alpha * dFold) * dStar - F.from(builder.chiEvaluations[0]);
    }

    function _computeEqualsExprAggregateEvaluation(
        VerificationBuilder.Builder memory builder,
        FF whereEvaluation,
        FF[] memory inputEvaluations,
        uint64 tableNumber
    ) internal pure returns (FF aggregateEvaluation) {
        aggregateEvaluation = F.from(builder.aggregateEvaluation)
            + F.from(builder.constraintMultipliers[0])
                * _computeEqualsExprZeroSumConstraint0(builder, whereEvaluation, inputEvaluations)
            + F.from(builder.constraintMultipliers[1]) * F.from(builder.rowMultipliersEvaluation)
                * _computeEqualsExprIdentityConstraint1(builder, inputEvaluations, tableNumber)
            + F.from(builder.constraintMultipliers[2]) * F.from(builder.rowMultipliersEvaluation)
                * _computeEqualsExprIdentityConstraint2(builder, inputEvaluations);
    }

    function _computeFilterExecResultEvaluations(
        VerificationBuilder.Builder memory builder,
        uint256 inputEvaluationsLength
    ) internal pure returns (uint256[] memory resultEvaluations) {
        resultEvaluations = new uint256[](inputEvaluationsLength);
        for (uint256 i = 0; i < inputEvaluationsLength; ++i) {
            resultEvaluations[i] = builder.finalRoundMLEs[i];
        }
    }

    function testFuzzFilterExec(
        VerificationBuilder.Builder memory builder,
        int64 where,
        int64[] memory inputs,
        uint64 tableNumber
    ) public pure {
        uint64 inputsLength = uint64(inputs.length);
        bytes memory plan = abi.encodePacked(
            tableNumber, abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, where), inputsLength
        );
        for (uint256 i = 0; i < inputsLength; ++i) {
            plan = abi.encodePacked(plan, abi.encodePacked(LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, inputs[i]));
        }
        plan = abi.encodePacked(plan, hex"abcdef");

        vm.assume(builder.maxDegree > 2);
        vm.assume(builder.finalRoundMLEs.length > inputsLength + 1);
        vm.assume(builder.constraintMultipliers.length > 2);
        vm.assume(builder.challenges.length > 1);
        vm.assume(builder.chiEvaluations.length > 0);
        vm.assume(builder.tableChiEvaluations.length > tableNumber);

        FF[] memory inputEvaluations = new FF[](inputsLength);
        for (uint256 i = 0; i < inputsLength; ++i) {
            inputEvaluations[i] = F.from(inputs[i]) * F.from(builder.tableChiEvaluations[tableNumber]);
        }

        uint256 expectedAggregateEvaluation = _computeEqualsExprAggregateEvaluation(
            builder, F.from(where) * F.from(builder.tableChiEvaluations[tableNumber]), inputEvaluations, tableNumber
        ).into();
        uint256[] memory expectedResultEvaluations = _computeFilterExecResultEvaluations(builder, inputsLength);

        uint256[] memory evals;
        (plan, builder, evals) = FilterExec.__filterExecEvaluate(plan, builder);

        uint256 evalsLength = evals.length;
        assert(evalsLength == expectedResultEvaluations.length);
        for (uint256 i = 0; i < evalsLength; ++i) {
            assert(evals[i] == expectedResultEvaluations[i]);
        }
        assert(builder.aggregateEvaluation == expectedAggregateEvaluation);

        bytes memory expectedExprOut = hex"abcdef";
        assert(plan.length == expectedExprOut.length);
        uint256 exprOutLength = plan.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(plan[i] == expectedExprOut[i]);
        }
    }
}
