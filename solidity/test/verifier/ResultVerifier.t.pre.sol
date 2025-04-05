// SPDX-License-Identifier: UNLICENSED
// This is licensed under the Cryptographic Open Software License 1.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";
import "../../src/base/Constants.sol";
import {Errors} from "../../src/base/Errors.sol";
import "../../src/base/LagrangeBasisEvaluation.pre.sol";
import {ResultVerifier} from "../../src/verifier/ResultVerifier.pre.sol";
import {FF, F} from "../base/FieldUtil.sol";

contract ResultVerifierTest is Test {
    /// Utility for generating a serailized result
    function generateResult(int64[][] memory table, bytes[] memory columnNames, uint8 quoteType, uint32 columnVariant)
        public
        pure
        returns (bytes memory result)
    {
        uint64 numberOfColumns = uint64(table.length);
        result = abi.encodePacked(numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            uint64 numberOfRows = uint64(table[i].length);
            result = bytes.concat(
                result,
                abi.encodePacked(uint64(columnNames[i].length), columnNames[i], quoteType, columnVariant, numberOfRows)
            );
            for (uint256 j = 0; j < numberOfRows; ++j) {
                result = bytes.concat(result, bytes8(uint64(table[i][j])));
            }
        }
    }

    /// Utility for evaluating a table at a given point. This is incredibly inefficient.
    function evaluateTable(int64[][] memory table, uint256[] memory evaluationPoint)
        public
        pure
        returns (uint256[] memory evaluations)
    {
        uint64 numberOfColumns = uint64(table.length);
        evaluations = new uint256[](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            uint64 numberOfRows = uint64(table[i].length);
            uint256[] memory evaluationVector =
                LagrangeBasisEvaluation.__computeEvaluationVec(numberOfRows, evaluationPoint);
            FF evaluation = F.ZERO;
            for (uint256 j = 0; j < numberOfRows; ++j) {
                evaluation = evaluation + F.from(table[i][j]) * F.from(evaluationVector[j]);
            }
            evaluations[i] = evaluation.into();
        }
    }

    function testCorrectResult() public pure {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1][0] = -4;
        table[1][1] = 5;
        table[1][2] = -6;

        bytes memory result = generateResult(table, columnNames, 0, COLUMN_BIGINT_VARIANT);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);

        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }

    function testIncorrectResult() public {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1][0] = -4;
        table[1][1] = 5;
        table[1][2] = -6;

        bytes memory result = generateResult(table, columnNames, 0, COLUMN_BIGINT_VARIANT);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);
        ++evaluations[0];

        vm.expectRevert(Errors.IncorrectResult.selector);
        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }

    function testResultColumnMismatch() public {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1][0] = -4;
        table[1][1] = 5;
        table[1][2] = -6;

        bytes memory result = generateResult(table, columnNames, 0, COLUMN_BIGINT_VARIANT);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);
        uint256[] memory wrongEvaluations = new uint256[](1);
        wrongEvaluations[0] = evaluations[0];

        vm.expectRevert(Errors.ResultColumnCountMismatch.selector);
        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, wrongEvaluations);
    }

    function testInvalidColumnNameInResult() public {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1][0] = -4;
        table[1][1] = 5;
        table[1][2] = -6;

        bytes memory result = generateResult(table, columnNames, 1, COLUMN_BIGINT_VARIANT);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);

        vm.expectRevert(Errors.InvalidResultColumnName.selector);
        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }

    function testInvalidColumnTypeInResult() public {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1][0] = -4;
        table[1][1] = 5;
        table[1][2] = -6;

        bytes memory result = generateResult(table, columnNames, 0, 1);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);

        vm.expectRevert(Errors.UnsupportedLiteralVariant.selector);
        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }

    function testInconsistentResultColumnLengths() public {
        uint64 numberOfColumns = 2;
        uint64 numberOfRows = 3;

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            table[i] = new int64[](numberOfRows);
        }

        columnNames[0] = "A";
        table[0][0] = 1;
        table[0][1] = 2;
        table[0][2] = 3;

        columnNames[1] = "B2";
        table[1] = new int64[](2);
        table[1][0] = -4;
        table[1][1] = 5;

        bytes memory result = generateResult(table, columnNames, 0, COLUMN_BIGINT_VARIANT);

        uint256[] memory evaluationPoint = new uint256[](2);
        evaluationPoint[0] = 101;
        evaluationPoint[1] = 102;

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);

        vm.expectRevert(Errors.InconsistentResultColumnLengths.selector);
        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }

    function testFuzzCorrectResult(
        uint64 numberOfColumns,
        uint64 numberOfRows,
        bytes[] memory nameData,
        int64[] memory data,
        uint256[] memory evaluationPoint
    ) public pure {
        // solhint-disable-next-line gas-strict-inequalities
        vm.assume(nameData.length >= numberOfColumns);
        // solhint-disable-next-line gas-strict-inequalities
        vm.assume(data.length >= uint256(numberOfColumns) * uint256(numberOfRows));

        bytes[] memory columnNames = new bytes[](numberOfColumns);
        int64[][] memory table = new int64[][](numberOfColumns);
        for (uint256 i = 0; i < numberOfColumns; ++i) {
            columnNames[i] = nameData[i];
            table[i] = new int64[](numberOfRows);
            for (uint256 j = 0; j < numberOfRows; ++j) {
                table[i][j] = data[i * uint256(numberOfRows) + j];
            }
        }

        bytes memory result = generateResult(table, columnNames, 0, COLUMN_BIGINT_VARIANT);

        uint256[] memory evaluations = evaluateTable(table, evaluationPoint);

        ResultVerifier.__verifyResultEvaluations(result, evaluationPoint, evaluations);
    }
}
