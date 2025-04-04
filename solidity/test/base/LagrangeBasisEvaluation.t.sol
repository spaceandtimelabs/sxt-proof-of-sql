// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {LagrangeBasisEvaluation} from "./../../src/base/LagrangeBasisEvaluation.sol";
import {F, FF} from "./FieldUtil.sol";

/// A library for efficiently computing sums over Lagrange basis polynomials evaluated at points.
contract LagrangeBasisEvaluationTest is Test {
    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith0Variables() public pure {
        uint256[] memory point = new uint256[](0);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(1, point) == 1);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(0, point) == 0);
    }

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith1Variables() public pure {
        uint256[] memory point = new uint256[](1);
        point[0] = 2;
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(2, point) == 1);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(1, point) == MODULUS - 1);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(0, point) == 0);
    }

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith2Variables() public pure {
        uint256[] memory point = new uint256[](2);
        point[0] = 2;
        point[1] = 5;
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(4, point) == 1);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(3, point) == MODULUS - 9);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(2, point) == MODULUS - 4);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(1, point) == 4);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(0, point) == 0);
    }

    function testComputeTruncatedLagrangeBasisSumGivesCorrectValuesWith3Variables() public pure {
        uint256[] memory point = new uint256[](3);
        point[0] = 2;
        point[1] = 5;
        point[2] = 7;
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(8, point) == 1);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(7, point) == MODULUS - 69);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(6, point) == MODULUS - 34);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(5, point) == 22);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(4, point) == MODULUS - 6);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(3, point) == 54);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(2, point) == 24);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(1, point) == MODULUS - 24);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(0, point) == 0);
    }

    function testComputeTruncatedLagrangeBasisInnerProductGivesCorrectValuesWith0Variables() public pure {
        uint256[] memory a = new uint256[](0);
        uint256[] memory b = new uint256[](0);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(1, a, b) == 1);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(0, a, b) == 0);
    }

    function testComputeTruncatedLagrangeBasisInnerProductGivesCorrectValuesWith1Variable() public pure {
        uint256[] memory a = new uint256[](1);
        uint256[] memory b = new uint256[](1);
        a[0] = 2;
        b[0] = 3;
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(2, a, b) == 8);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(1, a, b) == 2);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(0, a, b) == 0);
    }

    function testComputeTruncatedLagrangeBasisInnerProductGivesCorrectValuesWith3Variables() public pure {
        uint256[] memory a = new uint256[](3);
        uint256[] memory b = new uint256[](3);
        a[0] = 2;
        a[1] = 5;
        a[2] = 7;
        b[0] = 3;
        b[1] = 11;
        b[2] = 13;

        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(8, a, b) == 123880);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(7, a, b) == 93850);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(6, a, b) == 83840);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(5, a, b) == 62000);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(4, a, b) == 54720);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(3, a, b) == 30960);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(2, a, b) == 23040);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(1, a, b) == 5760);
        assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(0, a, b) == 0);
    }

    uint256 private constant MAX_FUZZ_POINT_LENGTH = 5;
    uint256 private constant EXTRA_FUZZ_LENGTH = 10;

    function testFuzzComputeTruncatedLagrangeBasisSum(uint256[] memory rand) public pure {
        uint256 numVars = rand.length;
        if (numVars > MAX_FUZZ_POINT_LENGTH) {
            numVars = MAX_FUZZ_POINT_LENGTH;
        }
        uint256[] memory point = new uint256[](numVars);
        for (uint256 i = 0; i < numVars; ++i) {
            point[i] = rand[i];
        }

        FF sum = F.ZERO;
        for (uint256 i = 0; i < (1 << numVars) + EXTRA_FUZZ_LENGTH; ++i) {
            FF product = F.ONE;
            for (uint256 j = 0; j < numVars; ++j) {
                FF term = F.from(point[j]);
                if ((i >> j) & 1 == 0) {
                    term = F.ONE - term;
                }
                product = product * term;
            }
            if (i >> numVars != 0) {
                product = F.ZERO;
            }
            assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisSum(i, point) == sum.into());
            sum = sum + product;
        }
    }

    function testFuzzComputeTruncatedLagrangeBasisInnerProduct(uint256[] memory rand) public pure {
        uint256 numVars = rand.length / 2;
        if (numVars > MAX_FUZZ_POINT_LENGTH) {
            numVars = MAX_FUZZ_POINT_LENGTH;
        }
        uint256[] memory a = new uint256[](numVars);
        uint256[] memory b = new uint256[](numVars);
        for (uint256 i = 0; i < numVars; ++i) {
            a[i] = rand[i];
            b[i] = rand[i + numVars];
        }

        FF sum = F.ZERO;
        for (uint256 i = 0; i < (1 << numVars) + EXTRA_FUZZ_LENGTH; ++i) {
            FF product = F.ONE;
            for (uint256 j = 0; j < numVars; ++j) {
                FF aTerm = F.from(a[j]);
                FF bTerm = F.from(b[j]);
                if ((i >> j) & 1 == 0) {
                    aTerm = F.ONE - aTerm;
                    bTerm = F.ONE - bTerm;
                }
                product = product * aTerm * bTerm;
            }
            if (i >> numVars != 0) {
                product = F.ZERO;
            }

            assert(LagrangeBasisEvaluation.__computeTruncatedLagrangeBasisInnerProduct(i, a, b) == sum.into());
            sum = sum + product;
        }
    }

    function testSimpleComputeEvaluationVec() public pure {
        uint256[] memory point = new uint256[](3);
        point[0] = 2;
        point[1] = 3;
        point[2] = 5;
        FF a = F.from(point[0]);
        FF b = F.from(point[1]);
        FF c = F.from(point[2]);
        uint256[] memory expectedEvaluations = new uint256[](8);
        expectedEvaluations[0] = ((F.ONE - a) * (F.ONE - b) * (F.ONE - c)).into();
        expectedEvaluations[1] = (a * (F.ONE - b) * (F.ONE - c)).into();
        expectedEvaluations[2] = ((F.ONE - a) * b * (F.ONE - c)).into();
        expectedEvaluations[3] = (a * b * (F.ONE - c)).into();
        expectedEvaluations[4] = ((F.ONE - a) * (F.ONE - b) * c).into();
        expectedEvaluations[5] = (a * (F.ONE - b) * c).into();
        expectedEvaluations[6] = ((F.ONE - a) * b * c).into();
        expectedEvaluations[7] = (a * b * c).into();

        for (uint256 length = 0; length < 8; ++length) {
            uint256[] memory evaluations = LagrangeBasisEvaluation.__computeEvaluationVec(length, point);
            assert(evaluations.length == length);
            for (uint256 i = 0; i < length; ++i) {
                assert(evaluations[i] == expectedEvaluations[i]);
            }
        }
    }

    function testFuzzComputeEvaluationVec(uint256[] memory point) public pure {
        uint256 numVars = point.length;
        // If the point is too long, we will run out of memory
        vm.assume(numVars < 10);
        uint256 maxLength = 1 << numVars;
        uint256[] memory expectedEvaluations = new uint256[](maxLength);
        for (uint256 i = 0; i < maxLength; ++i) {
            FF product = F.ONE;
            for (uint256 j = 0; j < numVars; ++j) {
                FF term = F.from(point[j]);
                if ((i >> j) & 1 == 0) {
                    term = F.ONE - term;
                }
                product = product * term;
            }
            expectedEvaluations[i] = product.into();
        }

        for (uint256 length = 0; length < maxLength; ++length) {
            uint256[] memory evaluations = LagrangeBasisEvaluation.__computeEvaluationVec(length, point);
            assert(evaluations.length == length);
            for (uint256 i = 0; i < length; ++i) {
                assert(evaluations[i] == expectedEvaluations[i]);
            }
        }
    }
}
