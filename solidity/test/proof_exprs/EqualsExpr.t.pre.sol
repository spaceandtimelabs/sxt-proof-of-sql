// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import {EqualsExpr} from "../../src/proof_exprs/EqualsExpr.pre.sol";
import {VerificationBuilder} from "../../src/proof/VerificationBuilder.pre.sol";

contract EqualsExprTest is Test {
    function testEqualsExpr() public pure {
        VerificationBuilder.Builder memory builder;

        // Setup column evaluations for first expression
        uint256[] memory columnValues = new uint256[](2);
        columnValues[0] = 0x11111111;
        columnValues[1] = 0x22222222;
        builder.columnEvaluations = columnValues;

        // Setup final round MLEs
        uint256[] memory finalRoundMLEs = new uint256[](2);
        finalRoundMLEs[0] = 0x33333333; // diff_star_eval
        finalRoundMLEs[1] = 0x44444444; // eval
        builder.finalRoundMLEs = finalRoundMLEs;

        // Setup constraint multipliers
        uint256[] memory constraintMultipliers = new uint256[](2);
        constraintMultipliers[0] = 1;
        constraintMultipliers[1] = 1;
        builder.constraintMultipliers = constraintMultipliers;
        builder.rowMultipliersEvaluation = 1;
        builder.maxDegree = 3;

        // Build expression: equals(column(1), literal(2))
        bytes memory exprIn = abi.encodePacked(
            COLUMN_EXPR_VARIANT, uint64(1), LITERAL_EXPR_VARIANT, LITERAL_BIGINT_VARIANT, int64(2), hex"abcdef"
        );

        bytes memory expectedExprOut = hex"abcdef";

        (bytes memory exprOut, uint256 eval) = EqualsExpr.__equalsExprEvaluate(exprIn, builder, 3);

        // The eval should be the second final round MLE
        assert(eval == 0x44444444);

        // Check remaining expression matches expected
        assert(exprOut.length == expectedExprOut.length);
        uint256 exprOutLength = exprOut.length;
        for (uint256 i = 0; i < exprOutLength; ++i) {
            assert(exprOut[i] == expectedExprOut[i]);
        }
    }

    // function testSetupEqualsExpr(
    //     uint256 chiEval,
    //     uint256 diffStarEval,
    //     uint256 eval,
    //     uint256 lhsEval,
    //     uint256 rhsEval,
    //     bytes memory trailingExpr
    // ) public {
    //     vm.assume(chiEval != 0);

    //     VerificationBuilder.Builder memory builder;

    //     // Setup final round MLEs
    //     uint256[] memory finalRoundMLEs = new uint256[](2);
    //     finalRoundMLEs[0] = diffStarEval;
    //     finalRoundMLEs[1] = eval;
    //     builder.finalRoundMLEs = finalRoundMLEs;

    //     // Setup constraint multipliers
    //     uint256[] memory constraintMultipliers = new uint256[](2);
    //     constraintMultipliers[0] = 1;
    //     constraintMultipliers[1] = 1;
    //     builder.constraintMultipliers = constraintMultipliers;
    //     builder.rowMultipliersEvaluation = 1;
    //     builder.maxDegree = 3;

    //     // Setup column evaluations
    //     uint256[] memory columnValues = new uint256[](2);
    //     columnValues[0] = lhsEval;
    //     columnValues[1] = rhsEval;
    //     builder.columnEvaluations = columnValues;

    //     bytes memory exprIn = abi.encodePacked(uint64(0), uint64(1), trailingExpr);

    //     (bytes memory exprOut, uint256 resultEval) = EqualsExpr.__equalsExprEvaluate(exprIn, builder, chiEval);

    //     assert(resultEval == eval);
    //     assert(exprOut.length == trailingExpr.length);
    // }
}
